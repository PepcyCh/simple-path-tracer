use crate::core::color::Color;
use crate::core::light::Light;
use crate::core::sampler::Sampler;
use cgmath::{InnerSpace, Point3, Vector3};

pub struct DirLight {
    direction: Vector3<f32>,
    strength: Color,
}

impl DirLight {
    pub fn new(direction: Vector3<f32>, strength: Color) -> Self {
        Self {
            direction: direction.normalize(),
            strength,
        }
    }
}

impl Light for DirLight {
    fn sample(
        &self,
        _position: Point3<f32>,
        _sampler: &mut dyn Sampler,
    ) -> (Vector3<f32>, f32, Color, f32) {
        (-self.direction, 1.0, self.strength, f32::MAX)
    }

    fn strength_dist_pdf(&self, _position: Point3<f32>, wi: Vector3<f32>) -> (Color, f32, f32) {
        if wi.dot(self.direction) <= -0.99 {
            (self.strength, f32::MAX, 1.0)
        } else {
            (Color::BLACK, f32::MAX, 0.0)
        }
    }

    fn is_delta(&self) -> bool {
        true
    }
}
