use crate::core::{color::Color, rng::Rng};

use super::{
    util, BxdfDirType, BxdfInputs, BxdfLobeType, BxdfSample, BxdfSampleType, BxdfT, Fresnel,
    FresnelT, Microfacet, MicrofacetT, Substrate, SubstrateT,
};

pub struct MicrofacetPlastic {
    microfacet: Microfacet,
    fresnel: Fresnel,
    substrate: Substrate,
}

impl MicrofacetPlastic {
    pub fn new(microfacet: Microfacet, fresnel: Fresnel, substrate: Substrate) -> Self {
        Self {
            microfacet,
            fresnel,
            substrate,
        }
    }
}

impl BxdfT for MicrofacetPlastic {
    fn sample<'a>(&self, inputs: &'a BxdfInputs, rng: &mut Rng) -> BxdfSample {
        let fresnel_macro = self.fresnel.fresnel(inputs.wo, glam::Vec3A::Z);
        let specular_weight = fresnel_macro.luminance();
        let substrate_weight =
            ((Color::WHITE - fresnel_macro) * self.substrate.reflectance()).luminance();
        let sample_reflect_pdf = specular_weight / (specular_weight + substrate_weight);

        if rng.uniform_1d() < sample_reflect_pdf {
            let (half, half_pdf) = self.microfacet.sample_half(inputs, rng);
            let fresnel = self.fresnel.fresnel(inputs.wo, half);
            let wi = util::reflect_n(inputs.wo, half);
            let specular_bxdf = fresnel * self.microfacet.ndf_visible(inputs.wo, wi, half);
            let specular_pdf = sample_reflect_pdf * half_pdf / (4.0 * inputs.wo.dot(half).abs());

            let substrate_bxdf =
                (Color::WHITE - fresnel_macro) * self.substrate.bxdf(inputs.wo, wi);
            let substrate_pdf = (1.0 - sample_reflect_pdf) * self.substrate.pdf(inputs.wo, wi);

            BxdfSample {
                wi,
                ty: BxdfSampleType {
                    lobe: BxdfLobeType::Glossy,
                    dir: BxdfDirType::Reflect,
                    subsurface: false,
                },
                bxdf: specular_bxdf + substrate_bxdf,
                pdf: specular_pdf + substrate_pdf,
                subsurface: None,
            }
        } else {
            let samp = self.substrate.sample(inputs, rng);
            let substrate_pdf = (1.0 - sample_reflect_pdf) * samp.pdf;
            let substrate_bxdf = (Color::WHITE - fresnel_macro) * samp.bxdf;

            let half = util::half_from_reflect(inputs.wo, samp.wi);
            let half_pdf = self.microfacet.half_pdf(inputs.wo, half);
            let specular_pdf = sample_reflect_pdf * half_pdf / (4.0 * inputs.wo.dot(half).abs());

            let fresnel = self.fresnel.fresnel(inputs.wo, half);
            let specular_bxdf = fresnel * self.microfacet.ndf_visible(inputs.wo, samp.wi, half);

            BxdfSample {
                bxdf: substrate_bxdf + specular_bxdf,
                pdf: substrate_pdf + specular_pdf,
                ..samp
            }
        }
    }

    fn pdf(&self, wo: glam::Vec3A, wi: glam::Vec3A) -> f32 {
        if wo.z * wi.z >= 0.0 {
            let fresnel_macro = self.fresnel.fresnel(wo, glam::Vec3A::Z);
            let specular_weight = fresnel_macro.luminance();
            let substrate_weight =
                ((Color::WHITE - fresnel_macro) * self.substrate.reflectance()).luminance();
            let sample_reflect_pdf = specular_weight / (specular_weight + substrate_weight);

            let half = util::half_from_reflect(wo, wi);
            let half_pdf = self.microfacet.half_pdf(wo, half);
            let specular_pdf = sample_reflect_pdf * half_pdf / (4.0 * wo.dot(half).abs());

            let substrate_pdf = (1.0 - sample_reflect_pdf) * self.substrate.pdf(wo, wi);

            specular_pdf + substrate_pdf
        } else {
            1.0
        }
    }

    fn bxdf(&self, wo: glam::Vec3A, wi: glam::Vec3A) -> Color {
        if wo.z * wi.z >= 0.0 {
            let reflect = {
                let half = util::half_from_reflect(wo, wi);
                let fresnel = self.fresnel.fresnel(wo, half);
                fresnel * self.microfacet.ndf_visible(wo, wi, half)
            };
            let substrate = {
                let fresnel_macro = self.fresnel.fresnel(wo, glam::Vec3A::Z);
                (Color::WHITE - fresnel_macro) * self.substrate.bxdf(wo, wi)
            };

            reflect + substrate
        } else {
            Color::BLACK
        }
    }

    fn is_delta(&self) -> bool {
        false
    }
}
