use crate::core::{color::Color, scatter::ScatterType};
use crate::core::sampler::Sampler;
use crate::core::scatter::{Scatter, Transmit};
use cgmath::{InnerSpace, Point3, Vector3};

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
        _po: Point3<f32>,
        wo: Vector3<f32>,
        _pi: Point3<f32>,
        _sampler: &mut dyn Sampler,
    ) -> (Vector3<f32>, f32, Color, ScatterType) {
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
                ScatterType::specular_transmit()
            )
        } else {
            (wo, 1.0, Color::BLACK, ScatterType::specular_transmit())
        }
    }

    fn pdf(&self, _po: Point3<f32>, _wo: Vector3<f32>, _pi: Point3<f32>, _wi: Vector3<f32>) -> f32 {
        1.0
    }

    fn bxdf(
        &self,
        _po: Point3<f32>,
        wo: Vector3<f32>,
        _pi: Point3<f32>,
        wi: Vector3<f32>,
    ) -> Color {
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
