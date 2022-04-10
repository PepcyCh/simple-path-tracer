use crate::core::{color::Color, rng::Rng};

use super::{
    util, BxdfDirType, BxdfInputs, BxdfLobeType, BxdfSample, BxdfSampleType, BxdfT, Fresnel,
    FresnelT, Substrate, SubstrateT,
};

pub struct SpecularPlastic {
    fresnel: Fresnel,
    substrate: Substrate,
}

impl SpecularPlastic {
    pub fn new(fresnel: Fresnel, substrate: Substrate) -> Self {
        Self { fresnel, substrate }
    }
}

impl BxdfT for SpecularPlastic {
    fn sample<'a>(&self, inputs: &'a BxdfInputs, rng: &mut Rng) -> BxdfSample {
        let fresnel = self.fresnel.fresnel(inputs.wo, glam::Vec3A::Z);
        let specular_weight = fresnel.luminance();
        let substrate_weight =
            ((Color::WHITE - fresnel) * self.substrate.reflectance()).luminance();
        let sample_reflect_pdf = specular_weight / (specular_weight + substrate_weight);

        if rng.uniform_1d() < sample_reflect_pdf {
            let wi = util::reflect(inputs.wo);
            let specular_bxdf = fresnel / wi.z.abs();
            let specular_pdf = sample_reflect_pdf;

            let substrate_bxdf = (Color::WHITE - fresnel) * self.substrate.bxdf(inputs.wo, wi);
            let substrate_pdf = (1.0 - sample_reflect_pdf) * self.substrate.pdf(inputs.wo, wi);

            BxdfSample {
                wi,
                ty: BxdfSampleType {
                    lobe: BxdfLobeType::Specular,
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
            let substrate_bxdf = (Color::WHITE - fresnel) * samp.bxdf;

            let specular_pdf = sample_reflect_pdf;
            let specular_bxdf = fresnel / samp.wi.z.abs();

            BxdfSample {
                bxdf: substrate_bxdf + specular_bxdf,
                pdf: substrate_pdf + specular_pdf,
                ..samp
            }
        }
    }

    fn pdf(&self, wo: glam::Vec3A, wi: glam::Vec3A) -> f32 {
        if wo.z * wi.z >= 0.0 {
            let fresnel = self.fresnel.fresnel(wo, glam::Vec3A::Z);
            let specular_weight = fresnel.luminance();
            let substrate_weight =
                ((Color::WHITE - fresnel) * self.substrate.reflectance()).luminance();
            let sample_reflect_pdf = specular_weight / (specular_weight + substrate_weight);

            let specular_pdf = sample_reflect_pdf;
            let substrate_pdf = (1.0 - sample_reflect_pdf) * self.substrate.pdf(wo, wi);

            specular_pdf + substrate_pdf
        } else {
            1.0
        }
    }

    fn bxdf(&self, wo: glam::Vec3A, wi: glam::Vec3A) -> Color {
        if wo.z * wi.z >= 0.0 {
            let fresnel = self.fresnel.fresnel(wo, glam::Vec3A::Z);
            let reflect = fresnel / wi.z.abs();
            let substrate = (Color::WHITE - fresnel) * self.substrate.bxdf(wo, wi);

            reflect + substrate
        } else {
            Color::BLACK
        }
    }

    fn is_delta(&self) -> bool {
        false
    }
}
