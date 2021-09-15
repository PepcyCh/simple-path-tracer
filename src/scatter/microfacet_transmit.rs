use crate::core::{
    color::Color,
    sampler::Sampler,
    scatter::{Scatter, ScatterType, Transmit},
};

pub struct MicrofacetTransmit {
    transmittance: Color,
    ior: f32,
    _roughness: f32,
    roughness_sqr: f32,
}

impl MicrofacetTransmit {
    pub fn new(transmittance: Color, ior: f32, roughness: f32) -> Self {
        let roughness_sqr = roughness * roughness;
        Self {
            transmittance,
            ior,
            _roughness: roughness,
            roughness_sqr,
        }
    }
}

impl Scatter for MicrofacetTransmit {
    fn sample_wi(
        &self,
        _po: glam::Vec3A,
        wo: glam::Vec3A,
        _pi: glam::Vec3A,
        sampler: &mut dyn Sampler,
    ) -> (glam::Vec3A, f32, Color, ScatterType) {
        let (rand_x, rand_y) = sampler.uniform_2d();
        let cos_theta_sqr = crate::scatter::util::ggx_ndf_cdf_inverse(self.roughness_sqr, rand_x);
        let cos_theta = cos_theta_sqr.sqrt();
        let sin_theta = (1.0 - cos_theta_sqr).sqrt();
        let phi = 2.0 * std::f32::consts::PI * rand_y;
        let half = glam::Vec3A::new(sin_theta * phi.cos(), sin_theta * phi.sin(), cos_theta);

        if let Some(wi) = crate::scatter::util::refract_n(wo, half, self.ior) {
            if wi.z * wo.z <= 0.0 {
                let ndf = crate::scatter::util::ggx_ndf(half.z, self.roughness_sqr);
                let visible = crate::scatter::util::smith_separable_visible(
                    wo.z.abs(),
                    wi.z.abs(),
                    self.roughness_sqr,
                );
                let ior_ratio = if wo.z >= 0.0 {
                    1.0 / self.ior
                } else {
                    self.ior
                };

                let denom = ior_ratio * wo.dot(half) + wi.dot(half);
                let denom = denom * denom;
                let num = 4.0 * wo.dot(half).abs() * wi.dot(half).abs();
                let bsdf = self.transmittance * ndf * visible * num / denom;

                let num = wi.dot(half).abs();
                let pdf = ndf * half.z * num / denom;
                return (wi, pdf, bsdf, ScatterType::glossy_transmit());
            }
        }
        (wo, 1.0, Color::BLACK, ScatterType::glossy_transmit())
    }

    fn pdf(&self, _po: glam::Vec3A, wo: glam::Vec3A, _pi: glam::Vec3A, wi: glam::Vec3A) -> f32 {
        if wo.z * wi.z <= 0.0 {
            let half = crate::scatter::util::half_from_refract(wo, wi, self.ior);
            let ior_ratio = if wo.z >= 0.0 {
                1.0 / self.ior
            } else {
                self.ior
            };

            let denom = ior_ratio * wo.dot(half) + wi.dot(half);
            let denom = denom * denom;
            let num = wi.dot(half).abs();
            crate::scatter::util::ggx_ndf(half.z, self.roughness_sqr) * half.z * num / denom
        } else {
            1.0
        }
    }

    fn bxdf(&self, _po: glam::Vec3A, wo: glam::Vec3A, _pi: glam::Vec3A, wi: glam::Vec3A) -> Color {
        if wo.z * wi.z <= 0.0 {
            let half = crate::scatter::util::half_from_refract(wo, wi, self.ior);
            if wo.dot(half) * wi.dot(half) >= 0.0 {
                return Color::BLACK;
            }

            let ior_ratio = if wo.z >= 0.0 {
                1.0 / self.ior
            } else {
                self.ior
            };

            let ndf = crate::scatter::util::ggx_ndf(half.z, self.roughness_sqr);
            let visible = crate::scatter::util::smith_separable_visible(
                wo.z.abs(),
                wi.z.abs(),
                self.roughness_sqr,
            );

            let denom = ior_ratio * wo.dot(half) + wi.dot(half);
            let denom = denom * denom;
            let num = 4.0 * wo.dot(half).abs() * wi.dot(half).abs();
            self.transmittance * ndf * visible * num / denom
        } else {
            Color::BLACK
        }
    }

    fn is_delta(&self) -> bool {
        false
    }
}

impl Transmit for MicrofacetTransmit {}
