use crate::core::{color::Color, rng::Rng};

use super::{BxdfDirType, BxdfInputs, BxdfLobeType, BxdfSample, BxdfSampleType, BxdfT};

pub struct Lambert {
    reflectance: Color,
}

impl Lambert {
    pub fn new(reflectance: Color) -> Self {
        Self { reflectance }
    }

    pub(crate) fn reflectance(&self) -> Color {
        self.reflectance
    }
}

impl BxdfT for Lambert {
    fn sample<'a>(&self, inputs: &'a BxdfInputs, rng: &mut Rng) -> BxdfSample {
        let mut wi = rng.cosine_weighted_on_hemisphere();
        if inputs.wo.z < 0.0 {
            wi.z = -wi.z;
        }
        BxdfSample {
            wi,
            ty: BxdfSampleType {
                lobe: BxdfLobeType::Diffuse,
                dir: BxdfDirType::Reflect,
                subsurface: false,
            },
            bxdf: self.reflectance * std::f32::consts::FRAC_1_PI,
            pdf: wi.z.abs() * std::f32::consts::FRAC_1_PI,
            subsurface: None,
        }
    }

    fn pdf(&self, wo: glam::Vec3A, wi: glam::Vec3A) -> f32 {
        if wo.z * wi.z >= 0.0 {
            wi.z.abs() * std::f32::consts::FRAC_1_PI
        } else {
            1.0
        }
    }

    fn bxdf(&self, wo: glam::Vec3A, wi: glam::Vec3A) -> Color {
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
