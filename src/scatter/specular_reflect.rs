use crate::core::sampler::Sampler;
use crate::core::scatter::{Reflect, Scatter};
use crate::core::{color::Color, scatter::ScatterType};
use cgmath::{InnerSpace, Point3, Vector3};

pub struct SpecularReflect {
    reflectance: Color,
}

impl SpecularReflect {
    pub fn new(reflectance: Color) -> Self {
        Self { reflectance }
    }
}

impl Scatter for SpecularReflect {
    fn sample_wi(
        &self,
        _po: Point3<f32>,
        wo: Vector3<f32>,
        _pi: Point3<f32>,
        _sampler: &mut dyn Sampler,
    ) -> (Vector3<f32>, f32, Color, ScatterType) {
        let wi = crate::scatter::util::reflect(wo);
        (
            wi,
            1.0,
            self.reflectance / wi.z.abs(),
            ScatterType::specular_reflect(),
        )
    }

    fn pdf(&self, _po: Point3<f32>, _wo: Vector3<f32>, _pi: Point3<f32>, _wi: Vector3<f32>) -> f32 {
        1.0
    }

    fn bxdf(
        &self,
        _po: Point3<f32>,
        wo: Vector3<f32>,
        _pi: Point3<f32>,
        wi: Vector3<f32>,
    ) -> Color {
        let expected_wi = crate::scatter::util::reflect(wo);
        if expected_wi.dot(wi) >= 0.99 {
            self.reflectance / wi.z.abs()
        } else {
            Color::BLACK
        }
    }

    fn is_delta(&self) -> bool {
        true
    }
}

impl Reflect for SpecularReflect {}
