use crate::core::camera::Camera;
use crate::core::color::Color;
use crate::core::film::Film;
use crate::core::filter::Filter;
use crate::core::intersection::Intersection;
use crate::core::light::Light;
use crate::core::medium::Medium;
use crate::core::primitive::{Aggregate, Primitive};
use crate::core::ray::Ray;
use crate::core::sampler::Sampler;
use cgmath::{InnerSpace, Matrix, Point3, Vector3};
use image::RgbImage;
use rand::Rng;
use std::sync::{Arc, Mutex};

pub struct PathTracer {
    camera: Box<dyn Camera>,
    objects: Box<dyn Aggregate>,
    lights: Vec<Box<dyn Light>>,
    spp: u32,
    sampler: Box<dyn Sampler>,
    max_depth: u32,
    filter: Box<dyn Filter>,
}

impl PathTracer {
    const CUTOFF_LUMINANCE: f32 = 0.001;

    pub fn new(
        camera: Box<dyn Camera>,
        objects: Box<dyn Aggregate>,
        lights: Vec<Box<dyn Light>>,
        spp: u32,
        sampler: Box<dyn Sampler>,
        max_depth: u32,
        filter: Box<dyn Filter>,
    ) -> Self {
        Self {
            camera,
            objects,
            lights,
            spp,
            sampler,
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
                .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len}")
                .progress_chars("#>-"),
        );

        crossbeam::scope(|scope| {
            let mut samples = vec![];
            for _ in 0..height {
                for _ in 0..width {
                    let samples_pixel = self.sampler.pixel_samples(self.spp);
                    samples.push(samples_pixel);
                }
            }

            for j in 0..height {
                for i in 0..width {
                    let samples = std::mem::take(&mut samples[(j * width + i) as usize]);
                    let film = film.clone();
                    let progress_bar = progress_bar.clone();
                    let path_tracer = &self;
                    scope.spawn(move |_| {
                        for (offset_x, offset_y) in samples {
                            let x = (i as f32 + offset_x) / width as f32 * aspect - 0.5;
                            let y = ((height - j - 1) as f32 + offset_y) / height as f32 - 0.5;
                            let ray = path_tracer.camera.generate_ray((x, y));
                            let color = path_tracer.trace_ray(ray);
                            let mut film = film.lock().unwrap();
                            film.add_sample(i, j, (offset_x - 0.5, offset_y - 0.5), color);
                        }
                        progress_bar.inc(1);
                    });
                }
            }
        })
        .unwrap();

        let film = film.lock().unwrap();
        film.filter_to_image(self.filter.as_ref())
    }

    fn trace_ray(&self, mut ray: Ray) -> Color {
        let mut final_color = Color::BLACK;
        let mut color_coe = Color::WHITE;
        let mut curr_depth = 0;
        let mut rr_rng = rand::thread_rng();
        let mut curr_medium: Option<&dyn Medium> = None;
        let mut curr_primitive: Option<&dyn Primitive>;

        while curr_depth < self.max_depth {
            let mut inter = Intersection::default();
            let does_hit = self.objects.intersect(&ray, &mut inter);
            curr_primitive = inter.primitive;

            if let Some(medium) = curr_medium {
                let wo = -ray.direction;
                let (p, still_in_medium, attenuation) =
                    medium.sample_transport(ray.origin, wo, inter.t);
                color_coe *= attenuation;

                if !still_in_medium {
                    curr_medium = None;
                    continue;
                } else {
                    // I found that sampling to lights for medium make less influence to the result...
                    let mut li = Color::BLACK;
                    for light in &self.lights {
                        let (light_dir, pdf, light_strength, dist) = light.sample(p);
                        let phase = medium.phase(wo, light_dir);

                        let (shadow_ray, transported_dist) = self.shadow_ray_from_medium(
                            p,
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
                            let (wi, phase) = medium.sample_phase(wo);
                            let (light_strength, dist, light_pdf) = light.strength_dist_pdf(p, wi);

                            let (shadow_ray, transported_dist) = self.shadow_ray_from_medium(
                                p,
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

                let (wi, _) = medium.sample_phase(wo);
                ray = Ray::new(p, wi);
            } else {
                if !does_hit {
                    break;
                }

                let po = ray.point_at(inter.t);
                let mat = inter.primitive.unwrap().material().unwrap();

                let normal_to_world = normal_to_world_from_normal(inter.normal);
                let world_to_normal = normal_to_world.transpose();
                let wo = world_to_normal * -ray.direction;

                let (pi, ni, sp) = mat.sample_sp(po, wo, normal_to_world, &*self.objects);
                color_coe *= sp;
                if !color_coe.is_finite() || color_coe.luminance() < Self::CUTOFF_LUMINANCE {
                    break;
                }

                let normal_to_world = normal_to_world_from_normal(ni);
                let world_to_normal = normal_to_world.transpose();

                let mut li = if curr_depth == 0 {
                    mat.emissive()
                } else {
                    Color::BLACK
                };

                for light in &self.lights {
                    let (light_dir, pdf, light_strength, dist) = light.sample(pi);
                    let wi = world_to_normal * light_dir;
                    if wi.z < 0.0 {
                        continue;
                    }
                    let bsdf = mat.bsdf(wo, wi);
                    let mat_pdf = mat.pdf(wo, wi);
                    let shadow_ray = Ray::new(pi, light_dir);
                    if pdf != 0.0
                        && pdf.is_finite()
                        && !self.objects.intersect_test(&shadow_ray, dist - 0.001)
                    {
                        if light.is_delta() {
                            li += light_strength * bsdf * wi.z / pdf;
                        } else {
                            let weight = power_heuristic(1, pdf, 1, mat_pdf);
                            li += light_strength * bsdf * wi.z * weight / pdf;
                        }
                    }

                    if !light.is_delta() {
                        let (wi, pdf, bsdf) = mat.sample(wo);
                        if wi.z < 0.0 {
                            continue;
                        }
                        let light_dir = normal_to_world * wi;
                        let (light_strength, dist, light_pdf) =
                            light.strength_dist_pdf(pi, light_dir);
                        let shadow_ray = Ray::new(pi, light_dir);
                        if pdf != 0.0
                            && pdf.is_finite()
                            && !self.objects.intersect_test(&shadow_ray, dist - 0.001)
                        {
                            if mat.is_delta() {
                                li += light_strength * bsdf * wi.z / pdf;
                            } else {
                                let weight = power_heuristic(1, pdf, 1, light_pdf);
                                li += light_strength * bsdf * wi.z * weight / pdf;
                            }
                        }
                    }
                }
                final_color += color_coe * li;

                let (wi, pdf, bsdf) = mat.sample(wo);
                ray = Ray::new(pi, normal_to_world * wi);
                color_coe *= bsdf * wi.z.abs() / pdf;
                if !color_coe.is_finite() || color_coe.luminance() < Self::CUTOFF_LUMINANCE {
                    break;
                }

                if wi.z < 0.0 {
                    curr_medium = inter.primitive.unwrap().inside_medium();
                }
            }

            let rr_rand: f32 = rr_rng.gen();
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

fn normal_to_world_from_normal(normal: cgmath::Vector3<f32>) -> cgmath::Matrix3<f32> {
    let bitangent = if normal.y.abs() < 0.99 {
        cgmath::Vector3::unit_y()
    } else {
        cgmath::Vector3::unit_x()
    };
    let tangent = (bitangent.cross(normal)).normalize();
    let bitangent = normal.cross(tangent);
    cgmath::Matrix3::from_cols(tangent, bitangent, normal)
}

fn power_heuristic(n0: u32, p0: f32, n1: u32, p1: f32) -> f32 {
    let prod0 = n0 as f32 * p0;
    let prod1 = n1 as f32 * p1;
    prod0 * prod0 / (prod0 * prod0 + prod1 * prod1)
}
