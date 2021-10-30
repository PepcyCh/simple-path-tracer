use std::sync::Arc;

use crate::{core::{color::Color, intersection::Intersection, light::Light, primitive::Primitive, ray::Ray, sampler::Sampler, scene::Instance, transform::Transform}};

pub struct ShapeLight {
    shape: Arc<Instance>,
}

impl ShapeLight {
    pub fn new(
        shape: Arc<Instance>,
    ) -> Self {
        Self {
            shape,
        }
    }
}

impl Light for ShapeLight {
    fn sample(
        &self,
        position: glam::Vec3A,
        sampler: &mut dyn Sampler,
    ) -> (glam::Vec3A, f32, Color, f32) {
        let (inter, pdf) = self.shape.sample(Transform::IDENTITY, sampler);
        let emissive = inter.surface.unwrap().emissive(&inter);

        let light_vec = inter.position - position;
        let light_dist_sqr = light_vec.length_squared();
        let light_dist = light_dist_sqr.sqrt();
        let light_dir = light_vec / light_dist;

        let (cos, emissive) = if inter.surface.unwrap().double_sided() {
            (light_dir.dot(inter.normal).abs(), emissive)
        } else {
            let cos = light_dir.dot(-inter.normal);
            if cos > 0.0 {
                (cos, emissive)
            } else {
                (1.0, Color::BLACK)
            }
        };
        let pdf = pdf * light_dist_sqr / cos.max(0.001);
        
        (light_dir, pdf, emissive, light_dist)
    }

    fn strength_dist_pdf(&self, position: glam::Vec3A, wi: glam::Vec3A) -> (Color, f32, f32) {
        let ray = Ray::new(position, wi);
        let mut inter = Intersection::default();
        if self.shape.intersect(&ray, &mut inter) {
            let emissive = inter.surface.unwrap().emissive(&inter);

            let pdf = self.shape.pdf(Transform::IDENTITY, &inter);

            let light_dist_sqr = inter.t * inter.t;
            let light_dist = inter.t;

            let (cos, emissive) = if inter.surface.unwrap().double_sided() {
                (wi.dot(inter.normal).abs(), emissive)
            } else {
                let cos = wi.dot(-inter.normal);
                if cos > 0.0 {
                    (cos, emissive)
                } else {
                    (1.0, Color::BLACK)
                }
            };
            let pdf = pdf * light_dist_sqr / cos.max(0.001);
            
            (emissive, light_dist, pdf)
        } else {
            (Color::BLACK, 0.0, 1.0)
        }
    }

    fn is_delta(&self) -> bool {
        false
    }
}