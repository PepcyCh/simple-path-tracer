use crate::core::{color::Color, rng::Rng};

use super::{Reflect, ScatterT, ScatterType};

pub struct LambertReflect {
    reflectance: Color,
}

impl LambertReflect {
    pub fn new(reflectance: Color) -> Self {
        Self { reflectance }
    }
}

impl ScatterT for LambertReflect {
    fn sample_wi(
        &self,
        _po: glam::Vec3A,
        wo: glam::Vec3A,
        _pi: glam::Vec3A,
        sampler: &mut Rng,
    ) -> (glam::Vec3A, f32, Color, ScatterType) {
        let mut wi = sampler.cosine_weighted_on_hemisphere();
        if wo.z < 0.0 {
            wi.z = -wi.z;
        }
        (
            wi,
            wi.z.abs() * std::f32::consts::FRAC_1_PI,
            self.reflectance * std::f32::consts::FRAC_1_PI,
            ScatterType::lambert_reflect(),
        )
    }

    fn pdf(&self, _po: glam::Vec3A, wo: glam::Vec3A, _pi: glam::Vec3A, wi: glam::Vec3A) -> f32 {
        if wo.z * wi.z >= 0.0 {
            wi.z.abs() * std::f32::consts::FRAC_1_PI
        } else {
            1.0
        }
    }

    fn bxdf(&self, _po: glam::Vec3A, wo: glam::Vec3A, _pi: glam::Vec3A, wi: glam::Vec3A) -> Color {
        if wo.z * wi.z >= 0.0 {
            self.reflectance * std::f32::consts::FRAC_1_PI
        } else {
            Color::BLACK
        }
    }

    fn is_delta(&self) -> bool {
        false
    }
}

impl Reflect for LambertReflect {}
