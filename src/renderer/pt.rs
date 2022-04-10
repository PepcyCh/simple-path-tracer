use std::{cell::UnsafeCell, mem::MaybeUninit};

use crate::{
    bxdf::{BxdfInputs, BxdfT},
    camera::CameraT,
    core::{
        color::Color,
        film::{Film, UnsafeFilm},
        intersection::Intersection,
        ray::Ray,
        rng::Rng,
        scene::Scene,
    },
    filter::Filter,
    light::LightT,
    light_sampler::{LightSamplerInputs, LightSamplerT},
    medium::{Medium, MediumT},
    pixel_sampler::{PixelSampler, PixelSamplerT},
    primitive::{BasicPrimitiveRef, PrimitiveT},
};

use super::{util, OutputConfig, RendererT};

pub struct PathTracer {
    max_depth: u32,
    pixel_sampler: PixelSampler,
    filter: Filter,
}

impl PathTracer {
    pub fn new(max_depth: u32, pixel_sampler: PixelSampler, filter: Filter) -> Self {
        Self {
            max_depth,
            pixel_sampler,
            filter,
        }
    }

    fn trace_ray(&self, scene: &Scene, mut ray: Ray, rng: &mut Rng) -> Color {
        let mut final_color = Color::BLACK;
        let mut throuput = Color::WHITE;
        let mut curr_depth = 0;
        let mut curr_medium: Option<&Medium> = None;
        let mut curr_primitive: Option<BasicPrimitiveRef<'_>>;
        let mut light_sampler_inputs = MaybeUninit::<LightSamplerInputs>::uninit();
        let mut last_sample_pdf = 0.0;

        while curr_depth < self.max_depth {
            let mut inter = Intersection::default();
            let does_hit = scene.aggregate().intersect(&ray, &mut inter);
            if does_hit {
                inter.calc_differential(&ray);
            }
            curr_primitive = inter.primitive;

            if let Some(medium) = curr_medium {
                let wo = -ray.direction;
                let (pi, still_in_medium, attenuation) =
                    medium.sample_pi(ray.origin, wo, inter.t, rng);
                throuput *= attenuation;

                if !still_in_medium {
                    curr_medium = None;
                    continue;
                } else {
                    let mut li = Color::BLACK;
                    light_sampler_inputs.write(LightSamplerInputs {
                        position: pi,
                        normal: glam::Vec3A::ZERO,
                    });
                    let (light_dir, pdf, light_strength, dist, light_is_delta) = scene
                        .light_sampler()
                        .sample_light(unsafe { light_sampler_inputs.assume_init_ref() }, rng);
                    let phase = medium.phase(wo, light_dir);

                    let (shadow_ray, transported_dist) =
                        self.shadow_ray_from_medium(pi, light_dir, dist, curr_primitive.unwrap());
                    let atten = medium.transport_attenuation(transported_dist);
                    if pdf != 0.0
                        && pdf.is_finite()
                        && !scene.aggregate().intersect_test(&shadow_ray, dist - 0.001)
                    {
                        if light_is_delta {
                            li = atten * phase * light_strength / pdf;
                        } else {
                            let weight = power_heuristic(1, pdf, 1, phase);
                            li = atten * phase * light_strength * weight / pdf;
                        }
                    }

                    final_color += throuput * li;
                }

                let (wi, pdf) = medium.sample_wi(wo, rng);
                last_sample_pdf = pdf;
                ray = Ray::new(pi, wi);
            } else if !does_hit {
                if let Some(env) = scene.environment() {
                    let (env, _, env_pdf) = env.strength_dist_pdf(ray.origin, ray.direction);
                    let weight = if curr_depth == 0 {
                        1.0
                    } else {
                        let pdf = scene
                            .light_sampler()
                            .pdf_env_light(unsafe { light_sampler_inputs.assume_init_ref() })
                            * env_pdf;
                        power_heuristic(1, last_sample_pdf, 1, pdf)
                    };
                    final_color += throuput * env * weight;
                }
                break;
            } else {
                if cfg!(feature = "debug_normal") {
                    let normal_color = Color::new(inter.normal.x, inter.normal.y, inter.normal.z);
                    let normal_color = normal_color * 0.5 + Color::gray(0.5);
                    final_color = normal_color;
                    break;
                }

                let mut po = ray.point_at(inter.t);
                let surf = inter.surface.unwrap();
                let (bxdf_context, mut coord_po) = surf.scatter_and_coord(&ray, &inter);

                let li_emissive = surf.emissive(&inter);
                if li_emissive.luminance() > 0.0 {
                    let weight = if curr_depth == 0 {
                        1.0
                    } else {
                        let pdf = scene.light_sampler().pdf_shape_light(
                            unsafe { light_sampler_inputs.assume_init_ref() },
                            inter.instance.unwrap(),
                            &inter,
                        );
                        power_heuristic(1, last_sample_pdf, 1, pdf)
                    };
                    final_color += throuput * li_emissive * weight;
                }

                let wo = coord_po.to_local(-ray.direction);
                let bxdf_inputs = BxdfInputs {
                    po,
                    coord_po,
                    wo,
                    scene: scene.aggregate(),
                };
                let samp = bxdf_context.sample(&bxdf_inputs, rng);
                if let Some(subsurface) = samp.subsurface {
                    po = subsurface.pi;
                    coord_po = subsurface.coord_pi;
                    throuput *= subsurface.sp / subsurface.pdf_pi;
                }

                let mut li = Color::BLACK;
                light_sampler_inputs.write(LightSamplerInputs {
                    position: po,
                    normal: inter.normal,
                });
                if !bxdf_context.is_delta() {
                    let (light_dir, pdf, light_strength, dist, light_is_delta) = scene
                        .light_sampler()
                        .sample_light(unsafe { light_sampler_inputs.assume_init_ref() }, rng);
                    let wi = coord_po.to_local(light_dir);
                    let bxdf = bxdf_context.bxdf(wo, wi);
                    let mat_pdf = bxdf_context.pdf(wo, wi);
                    let mut shadow_ray = Ray::new(po, light_dir);
                    shadow_ray.t_min = Ray::T_MIN_EPS / wi.z.abs().max(0.00001);
                    if pdf != 0.0
                        && pdf.is_finite()
                        && !scene.aggregate().intersect_test(&shadow_ray, dist - 0.001)
                    {
                        if light_is_delta {
                            li = light_strength * bxdf * wi.z.abs() / pdf.max(0.00001);
                        } else {
                            let weight = power_heuristic(1, pdf, 1, mat_pdf);
                            li = light_strength * bxdf * wi.z.abs() * weight / pdf.max(0.00001);
                        }
                    }
                    final_color += throuput * li;
                }

                last_sample_pdf = samp.pdf;
                let wi_world = coord_po.to_world(samp.wi);
                ray = Ray::new(po, wi_world);
                ray.t_min = Ray::T_MIN_EPS / samp.wi.z.abs().max(0.00001);
                throuput *= samp.bxdf * samp.wi.z.abs() / samp.pdf.max(0.00001);
                if !coord_po.in_expected_hemisphere(wi_world, samp.ty.dir) {
                    break;
                }

                if wi_world.dot(inter.normal) < 0.0 {
                    curr_medium = surf.inside_medium();
                }
            }

            if !throuput.is_finite() {
                break;
            }

            let rr_rand = rng.uniform_1d();
            let rr_prop = throuput.luminance().clamp(0.001, 0.95);
            if rr_rand > rr_prop {
                break;
            }
            throuput /= rr_prop;

            curr_depth += 1;
        }

        final_color
    }

    fn shadow_ray_from_medium(
        &self,
        p: glam::Vec3A,
        light_dir: glam::Vec3A,
        light_dist: f32,
        medium_primitive: BasicPrimitiveRef<'_>,
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

impl RendererT for PathTracer {
    fn render(&self, scene: &Scene, config: &OutputConfig) {
        let film = UnsafeCell::new(Film::new(config.width, config.height));
        let aspect = config.width as f32 / config.height as f32;

        let progress_bar = util::render_prograss_bar(config.width, config.height);

        let num_cpus = num_cpus::get() as u32 * 2;
        let ranges = util::create_image_ranges(num_cpus, config.height);

        let used_camera = scene.get_camera(&config.used_camera_name);

        crossbeam::scope(|scope| {
            for t in 0..num_cpus as usize {
                let width_inv = 1.0 / config.width as f32;
                let height_inv = 1.0 / config.height as f32;
                let mut pixel_sampler = self.pixel_sampler;
                let spp = pixel_sampler.spp();
                let spp_sqrt_inv = 1.0 / (spp as f32).sqrt();
                let film = UnsafeFilm::new(&film);
                let camera = used_camera.clone();
                let progress_bar = progress_bar.clone();
                let path_tracer = &self;
                let util::ImageRange { from, to } = ranges[t];

                scope.spawn(move |_| {
                    let mut rng = Rng::new();
                    for j in from..to {
                        for i in 0..config.width {
                            pixel_sampler.start_pixel();
                            while let Some((offset_x, offset_y)) =
                                pixel_sampler.next_sample(&mut rng)
                            {
                                let x = ((i as f32 + offset_x) * width_inv - 0.5) * aspect;
                                let y =
                                    ((config.height - j - 1) as f32 + offset_y) * height_inv - 0.5;
                                let ray = camera.generate_ray_with_aux_ray(
                                    (x, y),
                                    (aspect * width_inv * spp_sqrt_inv, height_inv * spp_sqrt_inv),
                                );
                                let color = path_tracer.trace_ray(scene, ray, &mut rng);
                                unsafe {
                                    film.add_sample(i, j, (offset_x - 0.5, offset_y - 0.5), color);
                                }
                            }
                            progress_bar.inc(1);
                        }
                    }
                });
            }
        })
        .unwrap();

        // TODO - filter_to_image can also be multi-threaded
        let film = film.into_inner();
        let image = film.filter_to_image(&self.filter);
        if let Err(err) = image.save(&config.output_filename) {
            println!("Failed to save image, err: {}", err);
        }
    }
}

fn power_heuristic(n0: u32, p0: f32, n1: u32, p1: f32) -> f32 {
    let prod0 = n0 as f32 * p0;
    let prod1 = n1 as f32 * p1;
    prod0 * prod0 / (prod0 * prod0 + prod1 * prod1)
}
