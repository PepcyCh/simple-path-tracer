use crate::core::color::Color;
use crate::core::light::Light;
use cgmath::{InnerSpace, Point3, Vector3};

pub struct PointLight {
    position: Point3<f32>,
    strength: Color,
}

impl PointLight {
    pub fn new(position: Point3<f32>, strength: Color) -> Self {
        Self { position, strength }
    }
}

impl Light for PointLight {
    fn sample(&self, position: Point3<f32>) -> (Vector3<f32>, f32, Color, f32) {
        let sample = self.position - position;
        let dist_sqr = sample.magnitude2();
        let dist = dist_sqr.sqrt();
        let sample = sample / dist;
        (sample, 1.0, self.strength / dist_sqr, dist)
    }

    fn is_delta(&self) -> bool {
        true
    }
}
