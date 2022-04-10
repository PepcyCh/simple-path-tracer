use crate::core::{color::Color, rng::Rng};

use super::{
    util, BxdfDirType, BxdfInputs, BxdfLobeType, BxdfSample, BxdfSampleType, BxdfT, Fresnel,
    FresnelT, Microfacet, MicrofacetT,
};

pub struct MicrofacetDielectric {
    microfacet: Microfacet,
    fresnel: Fresnel,
}

impl MicrofacetDielectric {
    pub fn new(microfacet: Microfacet, fresnel: Fresnel) -> Self {
        Self {
            microfacet,
            fresnel,
        }
    }
}

impl BxdfT for MicrofacetDielectric {
    fn sample<'a>(&self, inputs: &'a BxdfInputs, rng: &mut Rng) -> BxdfSample {
        let (half, half_pdf) = self.microfacet.sample_half(inputs, rng);
        let fresnel = self.fresnel.fresnel(inputs.wo, half);
        let sample_reflect_pdf = fresnel.luminance();
        if rng.uniform_1d() < sample_reflect_pdf {
            let wi = util::reflect_n(inputs.wo, half);
            let bxdf = fresnel * self.microfacet.ndf_visible(inputs.wo, wi, half);
            let pdf = sample_reflect_pdf * half_pdf / (4.0 * inputs.wo.dot(half).abs());

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
        } else if let Some(wi) = util::refract_n(inputs.wo, half, self.fresnel.ior()) {
            let ior_ratio = if inputs.wo.z >= 0.0 {
                1.0 / self.fresnel.ior()
            } else {
                self.fresnel.ior()
            };

            let denom = ior_ratio * inputs.wo.dot(half) + wi.dot(half);
            let denom = denom * denom;
            let num = wi.dot(half).abs();
            let pdf = (1.0 - sample_reflect_pdf) * half_pdf * num / denom;

            let num = 4.0 * inputs.wo.dot(half).abs() * wi.dot(half).abs();
            let bxdf =
                (Color::WHITE - fresnel) * self.microfacet.ndf_visible(inputs.wo, wi, half) * num
                    / denom;

            BxdfSample {
                wi,
                ty: BxdfSampleType {
                    lobe: BxdfLobeType::Glossy,
                    dir: BxdfDirType::Transmit,
                    subsurface: false,
                },
                bxdf,
                pdf,
                subsurface: None,
            }
        } else {
            BxdfSample {
                wi: glam::Vec3A::ZERO,
                ty: BxdfSampleType {
                    lobe: BxdfLobeType::Glossy,
                    dir: BxdfDirType::Transmit,
                    subsurface: false,
                },
                bxdf: Color::BLACK,
                pdf: 1.0,
                subsurface: None,
            }
        }
    }

    fn pdf(&self, wo: glam::Vec3A, wi: glam::Vec3A) -> f32 {
        if wo.z * wi.z >= 0.0 {
            let half = util::half_from_reflect(wo, wi);
            let half_pdf = self.microfacet.half_pdf(wo, half);
            let fresnel = self.fresnel.fresnel(wo, half);
            let sample_reflect_pdf = fresnel.luminance();
            let pdf = sample_reflect_pdf * half_pdf / (4.0 * wo.dot(half).abs());
            pdf
        } else {
            let half = util::half_from_refract(wo, wi, self.fresnel.ior());
            let half_pdf = self.microfacet.half_pdf(wo, half);
            let fresnel = self.fresnel.fresnel(wo, half);
            let sample_reflect_pdf = fresnel.luminance();

            let ior_ratio = if wo.z >= 0.0 {
                1.0 / self.fresnel.ior()
            } else {
                self.fresnel.ior()
            };
            let denom = ior_ratio * wo.dot(half) + wi.dot(half);
            let denom = denom * denom;
            let num = wi.dot(half).abs();
            let pdf = (1.0 - sample_reflect_pdf) * half_pdf * num / denom;
            pdf
        }
    }

    fn bxdf(&self, wo: glam::Vec3A, wi: glam::Vec3A) -> Color {
        if wo.z * wi.z >= 0.0 {
            let half = util::half_from_reflect(wo, wi);
            let fresnel = self.fresnel.fresnel(wo, half);
            let bxdf = fresnel * self.microfacet.ndf_visible(wo, wi, half);
            bxdf
        } else {
            let half = util::half_from_refract(wo, wi, self.fresnel.ior());
            let half_pdf = self.microfacet.half_pdf(wo, half);
            let fresnel = self.fresnel.fresnel(wo, half);
            let sample_reflect_pdf = fresnel.luminance();

            let ior_ratio = if wo.z >= 0.0 {
                1.0 / self.fresnel.ior()
            } else {
                self.fresnel.ior()
            };
            let denom = ior_ratio * wo.dot(half) + wi.dot(half);
            let denom = denom * denom;
            let num = 4.0 * wo.dot(half).abs() * wi.dot(half).abs();
            let bxdf =
                (Color::WHITE - fresnel) * self.microfacet.ndf_visible(wo, wi, half) * num / denom;
            bxdf
        }
    }

    fn is_delta(&self) -> bool {
        false
    }
}
