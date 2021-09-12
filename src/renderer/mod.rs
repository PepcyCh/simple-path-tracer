use crate::core::camera::Camera;
use crate::core::color::Color;
use crate::core::coord::Coordinate;
use crate::core::film::Film;
use crate::core::filter::Filter;
use crate::core::intersection::Intersection;
use crate::core::light::Light;
use crate::core::medium::Medium;
use crate::core::primitive::{Aggregate, Primitive};
use crate::core::ray::Ray;
use crate::core::sampler::Sampler;
use crate::light::EnvLight;
use crate::sampler::sampler_from;
use cgmath::{InnerSpace, Point3, Vector3};
use image::RgbImage;
use std::sync::{Arc, Mutex};

pub struct PathTracer {
    camera: Box<dyn Camera>,
    objects: Box<dyn Aggregate>,
    lights: Vec<Arc<dyn Light>>,
    environment: Option<Arc<EnvLight>>,
    spp: u32,
    sampler_type: &'static str,
    max_depth: u32,
    filter: Box<dyn Filter>,
}

impl PathTracer {
    const CUTOFF_LUMINANCE: f32 = 0.001;

    pub fn new(
        camera: Box<dyn Camera>,
        objects: Box<dyn Aggregate>,
        lights: Vec<Arc<dyn Light>>,
        environment: Option<Arc<EnvLight>>,
        spp: u32,
        sampler_type: &'static str,
        max_depth: u32,
        filter: Box<dyn Filter>,
    ) -> Self {
        Self {
            camera,
            objects,
            lights,
            environment,
            spp,
            sampler_type,
            max_depth,
            filter,
        }
    }

    pub fn render(&mut self, width: u32, height: u32) -> RgbImage {
        let film = Arc::new(Mutex::new(Film::new(width, height)));
        let aspect = width as f32 / height as f32;

        let progress_bar = indicatif::ProgressBar::new((width * height) as u64);
        progress_bar.set_style(
            indicatif::ProgressStyle::default_bar()
                .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} (eta: {eta})")
                .progress_chars("#>-"),
        );

        #[derive(Copy, Clone)]
        struct ImageRange {
            from: u32,
            to: u32,
        }
        let num_cpus = num_cpus::get() as u32 * 2;
        let height_per_cpu = height / num_cpus;
        let mut ranges = Vec::with_capacity(num_cpus as usize);
        for t in 0..num_cpus {
            let from = t * height_per_cpu;
            let to = if t + 1 == num_cpus {
                height
            } else {
                (t + 1) * height_per_cpu
            };
            ranges.push(ImageRange { from, to });
        }

        crossbeam::scope(|scope| {
            for t in 0..num_cpus as usize {
                let width_inv = 1.0 / width as f32;
                let height_inv = 1.0 / height as f32;
                let spp = self.spp;
                let spp_sqrt_inv = 1.0 / (spp as f32).sqrt();
                let sampler_type = self.sampler_type;
                let film = film.clone();
                let progress_bar = progress_bar.clone();
                let path_tracer = &self;
                let ImageRange { from, to } = ranges[t];

                scope.spawn(move |_| {
                    let mut sampler = sampler_from(sampler_type, spp);
                    for j in from..to {
                        for i in 0..width {
                            let samples = sampler.pixel_samples(spp);
                            for (offset_x, offset_y) in samples {
                                let x = ((i as f32 + offset_x) * width_inv - 0.5) * aspect;
                                let y = ((height - j - 1) as f32 + offset_y) * height_inv - 0.5;
                                // let ray = path_tracer.camera.generate_ray((x, y));
                                let ray = path_tracer.camera.generate_ray_with_aux_ray(
                                    (x, y),
                                    (aspect * width_inv * spp_sqrt_inv, height_inv * spp_sqrt_inv),
                                );
                                let color = path_tracer.trace_ray(ray, sampler.as_mut());
                                let mut film = film.lock().unwrap();
                                film.add_sample(i, j, (offset_x - 0.5, offset_y - 0.5), color);
                            }
                            progress_bar.inc(1);
                        }
                    }
                });
            }
        })
        .unwrap();

        let film = film.lock().unwrap();
        // TODO - filter_to_image can also be multi-threaded
        film.filter_to_image(self.filter.as_ref())
    }

    fn trace_ray(&self, mut ray: Ray, sampler: &mut dyn Sampler) -> Color {
        let mut final_color = Color::BLACK;
        let mut color_coe = Color::WHITE;
        let mut curr_depth = 0;
        let mut curr_medium: Option<&dyn Medium> = None;
        let mut curr_primitive: Option<&dyn Primitive>;

        while curr_depth < self.max_depth {
            let mut inter = Intersection::default();
            let does_hit = self.objects.intersect(&ray, &mut inter);
            if does_hit {
                inter.calc_differential(&ray);
                inter.apply_normal_map();
            }
            curr_primitive = inter.primitive;

            if let Some(medium) = curr_medium {
                let wo = -ray.direction;
                let (pi, still_in_medium, attenuation) =
                    medium.sample_pi(ray.origin, wo, inter.t, sampler);
                color_coe *= attenuation;

                if !still_in_medium {
                    curr_medium = None;
                    continue;
                } else {
                    // I found that sampling to lights for medium make less influence to the result...
                    let mut li = Color::BLACK;
                    for light in &self.lights {
                        let (light_dir, pdf, light_strength, dist) = light.sample(pi, sampler);
                        let phase = medium.phase(wo, light_dir);

                        let (shadow_ray, transported_dist) = self.shadow_ray_from_medium(
                            pi,
                            light_dir,
                            dist,
                            curr_primitive.unwrap(),
                        );
                        let atten = medium.transport_attenuation(transported_dist);
                        if pdf != 0.0
                            && pdf.is_finite()
                            && !self.objects.intersect_test(&shadow_ray, dist - 0.001)
                        {
                            if light.is_delta() {
                                li += atten * phase * light_strength / pdf;
                            } else {
                                let weight = power_heuristic(1, pdf, 1, phase);
                                li += atten * phase * light_strength * weight / pdf;
                            }
                        }

                        if !light.is_delta() {
                            let (wi, phase) = medium.sample_wi(wo, sampler);
                            let (light_strength, dist, light_pdf) = light.strength_dist_pdf(pi, wi);

                            let (shadow_ray, transported_dist) = self.shadow_ray_from_medium(
                                pi,
                                light_dir,
                                dist,
                                curr_primitive.unwrap(),
                            );
                            let atten = medium.transport_attenuation(transported_dist);
                            if phase != 0.0
                                && phase.is_finite()
                                && !self.objects.intersect_test(&shadow_ray, dist - 0.001)
                            {
                                let weight = power_heuristic(1, phase, 1, light_pdf);
                                li += atten * phase * light_strength * weight / phase;
                            }
                        }
                    }
                    final_color += color_coe * li;
                }

                let (wi, _) = medium.sample_wi(wo, sampler);
                ray = Ray::new(pi, wi);
            } else {
                if !does_hit {
                    if let Some(env) = &self.environment {
                        if curr_depth == 0 {
                            let (env, _, _) = env.strength_dist_pdf(ray.origin, ray.direction);
                            final_color += color_coe * env;
                        }
                    }
                    break;
                }

                if cfg!(feature = "debug_normal") {
                    let normal_color = Color::new(inter.normal.x, inter.normal.y, inter.normal.z);
                    let normal_color = normal_color * 0.5 + Color::gray(0.5);
                    final_color = normal_color;
                    break;
                }

                let po = ray.point_at(inter.t);
                let mat = inter.primitive.unwrap().material().unwrap();
                let scatter = mat.scatter(&inter);

                let coord_po = Coordinate::from_z(
                    inter.shade_normal,
                    if ray.direction.dot(inter.normal) > 0.0 {
                        -inter.normal
                    } else {
                        inter.normal
                    },
                );
                let wo = coord_po.to_local(-ray.direction);

                let (pi, coord_pi, sp_pdf, sp) =
                    scatter.sample_pi(po, wo, coord_po, sampler, &*self.objects);
                color_coe *= sp / sp_pdf;
                if !color_coe.is_finite() || color_coe.luminance() < Self::CUTOFF_LUMINANCE {
                    break;
                }

                let mut li = if curr_depth == 0 {
                    mat.emissive(&inter)
                } else {
                    Color::BLACK
                };

                for light in &self.lights {
                    let (light_dir, pdf, light_strength, dist) = light.sample(pi, sampler);
                    let wi = coord_pi.to_local(light_dir);
                    let bxdf = scatter.bxdf(po, wo, pi, wi);
                    let mat_pdf = scatter.pdf(po, wo, pi, wi);
                    let mut shadow_ray = Ray::new(pi, light_dir);
                    shadow_ray.t_min = Ray::T_MIN_EPS / wi.z.abs().max(0.00001);
                    if pdf != 0.0
                        && pdf.is_finite()
                        && !self.objects.intersect_test(&shadow_ray, dist - 0.001)
                    {
                        if light.is_delta() {
                            li += light_strength * bxdf * wi.z / pdf.max(0.00001);
                        } else {
                            let weight = power_heuristic(1, pdf, 1, mat_pdf);
                            li += light_strength * bxdf * wi.z * weight / pdf.max(0.00001);
                        }
                    }

                    if !light.is_delta() {
                        let (wi, pdf, bxdf, ty) = scatter.sample_wi(po, wo, pi, sampler);
                        let light_dir = coord_pi.to_world(wi);
                        if !coord_pi.in_expected_hemisphere(light_dir, ty.dir) {
                            continue;
                        }
                        let (light_strength, dist, light_pdf) =
                            light.strength_dist_pdf(pi, light_dir);
                        let shadow_ray = Ray::new(pi, light_dir);
                        if pdf != 0.0
                            && pdf.is_finite()
                            && !self.objects.intersect_test(&shadow_ray, dist - 0.001)
                        {
                            if scatter.is_delta() {
                                li += light_strength * bxdf * wi.z / pdf.max(0.00001);
                            } else {
                                let weight = power_heuristic(1, pdf, 1, light_pdf);
                                li += light_strength * bxdf * wi.z * weight / pdf.max(0.00001);
                            }
                        }
                    }
                }
                final_color += color_coe * li;

                let (wi, pdf, bxdf, ty) = scatter.sample_wi(po, wo, pi, sampler);
                let wi_world = coord_pi.to_world(wi);
                ray = Ray::new(pi, wi_world);
                ray.t_min = Ray::T_MIN_EPS / wi.z.abs().max(0.00001);
                color_coe *= bxdf * wi.z.abs() / pdf.max(0.00001);
                if !color_coe.is_finite() || color_coe.luminance() < Self::CUTOFF_LUMINANCE {
                    break;
                }
                if !coord_pi.in_expected_hemisphere(wi_world, ty.dir) {
                    break;
                }

                if wi.z < 0.0 {
                    curr_medium = inter.primitive.unwrap().inside_medium();
                }
            }

            let rr_rand = sampler.uniform_1d();
            let rr_prop = color_coe.luminance().clamp(Self::CUTOFF_LUMINANCE, 1.0);
            if rr_rand > rr_prop {
                break;
            }
            color_coe /= rr_prop;

            curr_depth += 1;
        }

        final_color
    }

    fn shadow_ray_from_medium(
        &self,
        p: Point3<f32>,
        light_dir: Vector3<f32>,
        light_dist: f32,
        medium_primitive: &dyn Primitive,
    ) -> (Ray, f32) {
        let mut shadow_ray = Ray::new(p, light_dir);

        let mut temp_inter = Intersection::with_t_max(light_dist - 0.001);

        let transported_dist;
        if medium_primitive.intersect(&shadow_ray, &mut temp_inter) {
            transported_dist = temp_inter.t;
            shadow_ray.t_min += temp_inter.t;
        } else {
            transported_dist = light_dist;
            shadow_ray.t_min += light_dist - 0.001;
        }

        (shadow_ray, transported_dist)
    }
}

fn power_heuristic(n0: u32, p0: f32, n1: u32, p1: f32) -> f32 {
    let prod0 = n0 as f32 * p0;
    let prod1 = n1 as f32 * p1;
    prod0 * prod0 / (prod0 * prod0 + prod1 * prod1)
}
