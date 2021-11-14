use crate::core::{
    color::Color,
    sampler::Sampler,
    scatter::{Scatter, ScatterType, Transmit},
};

pub struct MicrofacetTransmit {
    transmittance: Color,
    ior: f32,
    roughness_x: f32,
    roughness_y: f32,
}

impl MicrofacetTransmit {
    pub fn new(transmittance: Color, ior: f32, roughness_x: f32, roughness_y: f32) -> Self {
        Self {
            transmittance,
            ior,
            roughness_x,
            roughness_y,
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
        // let (rand_x, rand_y) = sampler.uniform_2d();
        // let cos_theta_sqr = crate::scatter::util::ggx_ndf_cdf_inverse(self.roughness_x * self.roughness_y, rand_x);
        // let cos_theta = cos_theta_sqr.sqrt();
        // let sin_theta = (1.0 - cos_theta_sqr).sqrt();
        // let phi = 2.0 * std::f32::consts::PI * rand_y;
        // let half = glam::Vec3A::new(sin_theta * phi.cos(), sin_theta * phi.sin(), cos_theta);
        let (half, pdf) = crate::scatter::util::ggx_smith_vndf_sample(
            wo,
            self.roughness_x,
            self.roughness_y,
            sampler.uniform_2d(),
        );

        if let Some(wi) = crate::scatter::util::refract_n(wo, half, self.ior) {
            if wi.z * wo.z <= 0.0 {
                let ndf =
                    crate::scatter::util::ggx_ndf_aniso(half, self.roughness_x, self.roughness_y);
                let visible = crate::scatter::util::smith_separable_visible_aniso(
                    wo,
                    wi,
                    self.roughness_x,
                    self.roughness_y,
                );
                // let ndf = crate::scatter::util::ggx_ndf(half.z, self.roughness_x * self.roughness_y);
                // let visible = crate::scatter::util::smith_separable_visible(
                //     wo.z.abs(),
                //     wi.z.abs(),
                //     self.roughness_x * self.roughness_y,
                // );
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
                let pdf = pdf * num / denom;
                // let pdf = ndf * half.z * num / denom;
                return (wi, pdf, bsdf, ScatterType::glossy_transmit());
            }
        }
        (wo, 1.0, Color::BLACK, ScatterType::glossy_transmit())
    }

    fn pdf(&self, _po: glam::Vec3A, wo: glam::Vec3A, _pi: glam::Vec3A, wi: glam::Vec3A) -> f32 {
        if wo.z * wi.z <= 0.0 {
            let half = crate::scatter::util::half_from_refract(wo, wi, self.ior);
            let pdf = crate::scatter::util::ggx_smith_vndf_pdf(
                half,
                wo,
                self.roughness_x,
                self.roughness_y,
            );
            let ior_ratio = if wo.z >= 0.0 {
                1.0 / self.ior
            } else {
                self.ior
            };

            let denom = ior_ratio * wo.dot(half) + wi.dot(half);
            let denom = denom * denom;
            let num = wi.dot(half).abs();
            pdf * num / denom
            // crate::scatter::util::ggx_ndf(half.z, self.roughness_x * self.roughness_y) * half.z * num / denom
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

            let ndf = crate::scatter::util::ggx_ndf_aniso(half, self.roughness_x, self.roughness_y);
            let visible = crate::scatter::util::smith_separable_visible_aniso(
                wo,
                wi,
                self.roughness_x,
                self.roughness_y,
            );
            // let ndf = crate::scatter::util::ggx_ndf(half.z, self.roughness_x * self.roughness_y);
            // let visible = crate::scatter::util::smith_separable_visible(
            //     wo.z.abs(),
            //     wi.z.abs(),
            //     self.roughness_x * self.roughness_y,
            // );

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
