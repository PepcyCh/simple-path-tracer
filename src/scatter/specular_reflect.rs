use crate::core::{color::Color, rng::Rng};

use super::{Reflect, ScatterT, ScatterType};

pub struct SpecularReflect {
    reflectance: Color,
}

impl SpecularReflect {
    pub fn new(reflectance: Color) -> Self {
        Self { reflectance }
    }
}

impl ScatterT for SpecularReflect {
    fn sample_wi(
        &self,
        _po: glam::Vec3A,
        wo: glam::Vec3A,
        _pi: glam::Vec3A,
        _rng: &mut Rng,
    ) -> (glam::Vec3A, f32, Color, ScatterType) {
        let wi = crate::scatter::util::reflect(wo);
        (
            wi,
            1.0,
            self.reflectance / wi.z.abs(),
            ScatterType::specular_reflect(),
        )
    }

    fn pdf(&self, _po: glam::Vec3A, _wo: glam::Vec3A, _pi: glam::Vec3A, _wi: glam::Vec3A) -> f32 {
        1.0
    }

    fn bxdf(&self, _po: glam::Vec3A, wo: glam::Vec3A, _pi: glam::Vec3A, wi: glam::Vec3A) -> Color {
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
