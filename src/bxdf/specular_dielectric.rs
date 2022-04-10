use crate::core::{color::Color, rng::Rng};

use super::{
    util, BxdfDirType, BxdfInputs, BxdfLobeType, BxdfSample, BxdfSampleType, BxdfT, Fresnel,
    FresnelT,
};

pub struct SpecularDielectric {
    fresnel: Fresnel,
}

impl SpecularDielectric {
    pub fn new(fresnel: Fresnel) -> Self {
        Self { fresnel }
    }
}

impl BxdfT for SpecularDielectric {
    fn sample<'a>(&self, inputs: &'a BxdfInputs, rng: &mut Rng) -> BxdfSample {
        let fresnel = self.fresnel.fresnel(inputs.wo, glam::Vec3A::Z);
        let sample_reflect_pdf = fresnel.luminance();
        if rng.uniform_1d() < sample_reflect_pdf {
            let wi = util::reflect(inputs.wo);
            let bxdf = fresnel / wi.z.abs();
            let pdf = sample_reflect_pdf;

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
        } else if let Some(wi) = util::refract(inputs.wo, self.fresnel.ior()) {
            let ior_ratio = if inputs.wo.z >= 0.0 {
                1.0 / self.fresnel.ior()
            } else {
                self.fresnel.ior()
            };
            let bxdf = ior_ratio * ior_ratio * (Color::WHITE - fresnel) / wi.z.abs();
            let pdf = 1.0 - sample_reflect_pdf;

            BxdfSample {
                wi,
                ty: BxdfSampleType {
                    lobe: BxdfLobeType::Specular,
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
                    lobe: BxdfLobeType::Specular,
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
        let fresnel = self.fresnel.fresnel(wo, glam::Vec3A::Z);
        let sample_reflect_pdf = fresnel.luminance();
        if wo.z * wi.z >= 0.0 {
            sample_reflect_pdf
        } else {
            1.0 - sample_reflect_pdf
        }
    }

    fn bxdf(&self, wo: glam::Vec3A, wi: glam::Vec3A) -> Color {
        let fresnel = self.fresnel.fresnel(wo, glam::Vec3A::Z);
        if wo.z * wi.z >= 0.0 {
            let expected_wi = util::reflect(wo);
            if wi.dot(expected_wi) > 0.999 {
                fresnel / wi.z.abs()
            } else {
                Color::BLACK
            }
        } else if let Some(expected_wi) = util::refract(wo, self.fresnel.ior()) {
            if wi.dot(expected_wi) > 0.999 {
                let ior_ratio = if wo.z >= 0.0 {
                    1.0 / self.fresnel.ior()
                } else {
                    self.fresnel.ior()
                };
                ior_ratio * ior_ratio * (Color::WHITE - fresnel) / wi.z.abs()
            } else {
                Color::BLACK
            }
        } else {
            Color::BLACK
        }
    }

    fn is_delta(&self) -> bool {
        true
    }
}
