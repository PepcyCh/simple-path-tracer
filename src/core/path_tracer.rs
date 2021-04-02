use crate::core::camera::Camera;
use crate::core::color::Color;
use crate::core::film::Film;
use crate::core::filter::Filter;
use crate::core::intersection::Intersection;
use crate::core::light::Light;
use crate::core::primitive::Aggregate;
use crate::core::ray::Ray;
use crate::core::sampler::Sampler;
use cgmath::{InnerSpace, Matrix};
use image::RgbImage;
use rand::Rng;

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
    const CUTOFF_LUMINANCE: f32 = 0.005;

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
        let mut film = Film::new(width, height);
        let aspect = width as f32 / height as f32;
        let mut rr_rng = rand::thread_rng();
        for j in 0..height {
            for i in 0..width {
                let samples = self.sampler.pixel_samples(self.spp);
                for (offset_x, offset_y) in samples {
                    let x = (i as f32 + offset_x) / width as f32 * aspect - 0.5;
                    let y = ((height - j - 1) as f32 + offset_y) / height as f32 - 0.5;
                    let ray = self.camera.generate_ray((x, y));
                    let color = self.trace_ray(ray, 0, &mut rr_rng);
                    film.add_sample(i, j, (x, y), color);
                }
            }
        }
        film.filter_to_image(self.filter.as_ref())
    }

    fn trace_ray(&self, ray: Ray, depth: u32, rr_rng: &mut rand::rngs::ThreadRng) -> Color {
        if depth >= self.max_depth {
            return Color::BLACK;
        }

        let mut inter = Intersection::default();
        if !self.objects.intersect(&ray, &mut inter) {
            return Color::BLACK;
        }

        let p = ray.point_at(inter.t);
        let mat = inter.primitive.unwrap().material().unwrap();

        let normal_to_world = normal_to_world(inter.normal);
        let world_to_normal = normal_to_world.transpose();
        let wo = world_to_normal * -ray.direction;

        let mut li = if depth == 0 {
            mat.emissive()
        } else {
            Color::BLACK
        };

        for light in &self.lights {
            let (light_dir, pdf, light_strength, dist) = light.sample(p);
            let wi = world_to_normal * light_dir;
            if wi.z < 0.0 {
                continue;
            }
            let bsdf = mat.bsdf(wo, wi);
            let mat_pdf = mat.pdf(wo, wi);
            let shadow_ray = Ray::new(p, light_dir);
            if pdf != 0.0 && light_strength.luminance() > Self::CUTOFF_LUMINANCE
                && bsdf.luminance() > Self::CUTOFF_LUMINANCE
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
                let light_dir = normal_to_world * wi;
                let (light_strength, dist, light_pdf) = light.strength_dist_pdf(p, light_dir);
                let shadow_ray = Ray::new(p, light_dir);
                if pdf != 0.0 && light_strength.luminance() > Self::CUTOFF_LUMINANCE
                    && bsdf.luminance() > Self::CUTOFF_LUMINANCE
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

        let rr_rand: f32 = rr_rng.gen();
        let rr_prop = li.luminance().clamp(0.2, 1.0);
        // let rr_prop = 0.8;
        if rr_rand > rr_prop {
            return li;
        }

        let (wi, pdf, bsdf) = mat.sample(wo);
        if bsdf.luminance() > Self::CUTOFF_LUMINANCE {
            let next_ray = Ray::new(p, normal_to_world * wi);
            let li_next = self.trace_ray(next_ray, depth + 1, rr_rng);
            li += li_next * bsdf * wi.z.abs() / pdf / rr_prop;
        }

        li
    }
}

fn normal_to_world(normal: cgmath::Vector3<f32>) -> cgmath::Matrix3<f32> {
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
