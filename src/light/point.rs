use crate::core::{color::Color, light::Light, sampler::Sampler};

pub struct PointLight {
    position: glam::Vec3A,
    strength: Color,
}

impl PointLight {
    pub fn new(position: glam::Vec3A, strength: Color) -> Self {
        Self { position, strength }
    }
}

impl Light for PointLight {
    fn sample(
        &self,
        position: glam::Vec3A,
        _sampler: &mut dyn Sampler,
    ) -> (glam::Vec3A, f32, Color, f32) {
        let sample = self.position - position;
        let dist_sqr = sample.length_squared();
        let dist = dist_sqr.sqrt();
        let sample = sample / dist;
        (sample, 1.0, self.strength / dist_sqr, dist)
    }

    fn strength_dist_pdf(&self, position: glam::Vec3A, wi: glam::Vec3A) -> (Color, f32, f32) {
        let dir = self.position - position;
        let dist_sqr = dir.length_squared();
        let dist = dist_sqr.sqrt();
        let dir = dir / dist;
        if dir.dot(wi) >= 0.99 {
            (self.strength / dist_sqr, dist, 1.0)
        } else {
            (Color::BLACK, f32::MAX, 0.0)
        }
    }

    fn is_delta(&self) -> bool {
        true
    }
}
