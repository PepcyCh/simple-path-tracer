use crate::core::{color::Color, rng::Rng};

use super::{
    util, BxdfDirType, BxdfInputs, BxdfLobeType, BxdfSample, BxdfSampleType, BxdfT, Fresnel,
    FresnelT, Microfacet, MicrofacetT,
};

pub struct MicrofacetConductor {
    microfacet: Microfacet,
    fresnel: Fresnel,
}

impl MicrofacetConductor {
    pub fn new(microfacet: Microfacet, fresnel: Fresnel) -> Self {
        Self {
            microfacet,
            fresnel,
        }
    }
}

impl BxdfT for MicrofacetConductor {
    fn sample<'a>(&self, inputs: &'a BxdfInputs, rng: &mut Rng) -> BxdfSample {
        let (half, half_pdf) = self.microfacet.sample_half(inputs, rng);
        let fresnel = self.fresnel.fresnel(inputs.wo, half);
        let wi = util::reflect_n(inputs.wo, half);
        let bxdf = fresnel * self.microfacet.ndf_visible(inputs.wo, wi, half);
        let pdf = half_pdf / (4.0 * inputs.wo.dot(half).abs());

        BxdfSample {
            wi,
            ty: BxdfSampleType {
                lobe: BxdfLobeType::Glossy,
                dir: BxdfDirType::Reflect,
                subsurface: false,
            },
            bxdf,
            pdf,
            subsurface: None,
        }
    }

    fn pdf(&self, wo: glam::Vec3A, wi: glam::Vec3A) -> f32 {
        if wo.z * wi.z >= 0.0 {
            let half = util::half_from_reflect(wo, wi);
            let half_pdf = self.microfacet.half_pdf(wo, half);
            let pdf = half_pdf / (4.0 * wo.dot(half).abs());
            pdf
        } else {
            1.0
        }
    }

    fn bxdf(&self, wo: glam::Vec3A, wi: glam::Vec3A) -> Color {
        if wo.z * wi.z >= 0.0 {
            let half = util::half_from_reflect(wo, wi);
            let fresnel = self.fresnel.fresnel(wo, half);
            let bxdf = fresnel * self.microfacet.ndf_visible(wo, wi, half);
            bxdf
        } else {
            Color::BLACK
        }
    }

    fn is_delta(&self) -> bool {
        false
    }
}
