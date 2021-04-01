use crate::core::color::Color;
use crate::core::light::Light;
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
    fn sample_light(&self, _position: Point3<f32>) -> (Vector3<f32>, f32, Color, f32) {
        (-self.direction, 1.0, self.strength, f32::MAX)
    }
}
