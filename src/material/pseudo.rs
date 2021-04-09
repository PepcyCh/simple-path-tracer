use crate::core::color::Color;
use crate::core::material::Material;
use cgmath::{InnerSpace, Vector3};

pub struct PseudoMaterial {}

impl PseudoMaterial {
    pub fn new() -> Self {
        Self {}
    }
}

impl Material for PseudoMaterial {
    fn sample(&self, wo: Vector3<f32>) -> (Vector3<f32>, f32, Color) {
        (-wo, wo.z.abs(), Color::WHITE)
    }

    fn bsdf(&self, wo: Vector3<f32>, wi: Vector3<f32>) -> Color {
        if wo.dot(wi) < -0.99 {
            Color::WHITE
        } else {
            Color::BLACK
        }
    }

    fn pdf(&self, wo: Vector3<f32>, wi: Vector3<f32>) -> f32 {
        if wo.dot(wi) < -0.99 {
            wi.z.abs()
        } else {
            0.0
        }
    }

    fn is_delta(&self) -> bool {
        true
    }

    fn emissive(&self) -> Color {
        Color::BLACK
    }
}
