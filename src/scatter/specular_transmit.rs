use crate::core::{
    color::Color,
    sampler::Sampler,
    scatter::{Scatter, ScatterType, Transmit},
};

pub struct SpecularTransmit {
    transmittance: Color,
    ior: f32,
}

impl SpecularTransmit {
    pub fn new(transmittance: Color, ior: f32) -> Self {
        Self { transmittance, ior }
    }
}

impl Scatter for SpecularTransmit {
    fn sample_wi(
        &self,
        _po: glam::Vec3A,
        wo: glam::Vec3A,
        _pi: glam::Vec3A,
        _sampler: &mut dyn Sampler,
    ) -> (glam::Vec3A, f32, Color, ScatterType) {
        if let Some(wi) = crate::scatter::util::refract(wo, self.ior) {
            let ior_ratio = if wo.z >= 0.0 {
                1.0 / self.ior
            } else {
                self.ior
            };
            (
                wi,
                1.0,
                ior_ratio * ior_ratio * self.transmittance / wi.z.abs(),
                ScatterType::specular_transmit(),
            )
        } else {
            (wo, 1.0, Color::BLACK, ScatterType::specular_transmit())
        }
    }

    fn pdf(&self, _po: glam::Vec3A, _wo: glam::Vec3A, _pi: glam::Vec3A, _wi: glam::Vec3A) -> f32 {
        1.0
    }

    fn bxdf(&self, _po: glam::Vec3A, wo: glam::Vec3A, _pi: glam::Vec3A, wi: glam::Vec3A) -> Color {
        if let Some(expected_wi) = crate::scatter::util::refract(wo, self.ior) {
            if expected_wi.dot(wi) >= 0.99 {
                let ior_ratio = if wo.z >= 0.0 {
                    1.0 / self.ior
                } else {
                    self.ior
                };
                return ior_ratio * ior_ratio * self.transmittance / wi.z.abs();
            }
        }
        Color::BLACK
    }

    fn is_delta(&self) -> bool {
        true
    }
}

impl Transmit for SpecularTransmit {}
