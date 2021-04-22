use crate::core::color::Color;
use crate::core::sampler::Sampler;
use crate::core::scatter::{Reflect, Scatter};
use cgmath::{Point3, Vector3};

pub struct LambertReflect {
    reflectance: Color,
}

impl LambertReflect {
    pub fn new(reflectance: Color) -> Self {
        Self { reflectance }
    }
}

impl Scatter for LambertReflect {
    fn sample_wi(
        &self,
        _po: Point3<f32>,
        wo: Vector3<f32>,
        _pi: Point3<f32>,
        sampler: &mut dyn Sampler,
    ) -> (Vector3<f32>, f32, Color) {
        let mut wi = sampler.cosine_weighted_on_hemisphere();
        if wo.z < 0.0 {
            wi.z = -wi.z;
        }
        (
            wi,
            wi.z.abs() / std::f32::consts::PI,
            self.reflectance / std::f32::consts::PI,
        )
    }

    fn pdf(&self, _po: Point3<f32>, wo: Vector3<f32>, _pi: Point3<f32>, wi: Vector3<f32>) -> f32 {
        if wo.z * wi.z >= 0.0 {
            wi.z.abs() / std::f32::consts::PI
        } else {
            1.0
        }
    }

    fn bxdf(
        &self,
        _po: Point3<f32>,
        wo: Vector3<f32>,
        _pi: Point3<f32>,
        wi: Vector3<f32>,
    ) -> Color {
        if wo.z * wi.z >= 0.0 {
            self.reflectance / std::f32::consts::PI
        } else {
            Color::BLACK
        }
    }

    fn is_delta(&self) -> bool {
        false
    }
}

impl Reflect for LambertReflect {}
