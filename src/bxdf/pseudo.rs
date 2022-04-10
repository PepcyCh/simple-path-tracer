use crate::core::{color::Color, rng::Rng};

use super::{BxdfDirType, BxdfInputs, BxdfLobeType, BxdfSample, BxdfSampleType, BxdfT};

pub struct Pseudo {}

impl Pseudo {
    pub fn new() -> Self {
        Self {}
    }
}

impl BxdfT for Pseudo {
    fn sample<'a>(&self, inputs: &'a BxdfInputs, rng: &mut Rng) -> BxdfSample {
        BxdfSample {
            wi: -inputs.wo,
            ty: BxdfSampleType {
                lobe: BxdfLobeType::Specular,
                dir: BxdfDirType::Transmit,
                subsurface: false,
            },
            bxdf: Color::WHITE / inputs.wo.z.abs(),
            pdf: 1.0,
            subsurface: None,
        }
    }

    fn pdf(&self, wo: glam::Vec3A, wi: glam::Vec3A) -> f32 {
        1.0
    }

    fn bxdf(&self, wo: glam::Vec3A, wi: glam::Vec3A) -> Color {
        if wo.dot(wi) < -0.999 {
            Color::WHITE / wi.z.abs()
        } else {
            Color::BLACK
        }
    }

    fn is_delta(&self) -> bool {
        true
    }
}
