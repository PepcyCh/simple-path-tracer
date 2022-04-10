use crate::core::{color::Color, rng::Rng};

use super::{
    util, BxdfDirType, BxdfInputs, BxdfLobeType, BxdfSample, BxdfSampleType, BxdfT, Fresnel,
    FresnelT,
};

pub struct SpecularConductor {
    fresnel: Fresnel,
}

impl SpecularConductor {
    pub fn new(fresnel: Fresnel) -> Self {
        Self { fresnel }
    }
}

impl BxdfT for SpecularConductor {
    fn sample<'a>(&self, inputs: &'a BxdfInputs, rng: &mut Rng) -> BxdfSample {
        let fresnel = self.fresnel.fresnel(inputs.wo, glam::Vec3A::Z);
        let wi = util::reflect(inputs.wo);
        let bxdf = fresnel / wi.z.abs();
        let pdf = 1.0;

        BxdfSample {
            wi,
            ty: BxdfSampleType {
                lobe: BxdfLobeType::Specular,
                dir: BxdfDirType::Reflect,
                subsurface: false,
            },
            bxdf,
            pdf,
            subsurface: None,
        }
    }

    fn pdf(&self, wo: glam::Vec3A, wi: glam::Vec3A) -> f32 {
        1.0
    }

    fn bxdf(&self, wo: glam::Vec3A, wi: glam::Vec3A) -> Color {
        let expected_wi = util::reflect(wo);
        if wi.dot(expected_wi) > 0.999 {
            let fresnel = self.fresnel.fresnel(wo, glam::Vec3A::Z);
            fresnel / wi.z.abs()
        } else {
            Color::BLACK
        }
    }

    fn is_delta(&self) -> bool {
        true
    }
}
