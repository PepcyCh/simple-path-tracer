use crate::core::color::Color;
use crate::core::material::Material;
use crate::core::sampler::Sampler;
use cgmath::{InnerSpace, Vector3};
use std::sync::Mutex;

pub struct MicrofacetGlass {
    reflectance: Color,
    transmittance: Color,
    ior: f32,
    _roughness: f32,
    roughness_sqr: f32,
    sampler: Box<Mutex<dyn Sampler>>,
}

impl MicrofacetGlass {
    pub fn new(
        reflectance: Color,
        transmittance: Color,
        ior: f32,
        roughness: f32,
        sampler: Box<Mutex<dyn Sampler>>,
    ) -> Self {
        Self {
            reflectance,
            transmittance,
            ior,
            _roughness: roughness,
            roughness_sqr: roughness * roughness,
            sampler,
        }
    }
}

impl Material for MicrofacetGlass {
    fn sample(&self, wo: Vector3<f32>) -> (Vector3<f32>, f32, Color) {
        let (rand_u, rand_x, rand_y) = {
            let mut sampler = self.sampler.lock().unwrap();
            let rand_u = sampler.uniform_1d();
            let (rand_x, rand_y) = sampler.uniform_2d();
            (rand_u, rand_x, rand_y)
        };

        let cos_theta_sqr = crate::material::util::ggx_ndf_cdf_inverse(self.roughness_sqr, rand_x);
        let cos_theta = cos_theta_sqr.sqrt();
        let sin_theta = (1.0 - cos_theta_sqr).sqrt();
        let phi = 2.0 * std::f32::consts::PI * rand_y;
        let sample_half = Vector3::new(sin_theta * phi.cos(), sin_theta * phi.sin(), cos_theta);
        let ndf = crate::material::util::ggx_ndf(sample_half.z, self.roughness_sqr);

        let fresnel = crate::material::util::fresnel_n(self.ior, wo, sample_half);

        if rand_u <= fresnel {
            let sample_wi = crate::material::util::reflect_n(wo, sample_half);
            if sample_wi.z * wo.z >= 0.0 {
                let visible = crate::material::util::smith_separable_visible(
                    wo.z.abs(),
                    sample_wi.z.abs(),
                    self.roughness_sqr,
                );
                let pdf = fresnel * ndf * sample_half.z / (4.0 * wo.dot(sample_half).abs());
                let bsdf = self.reflectance * fresnel * ndf * visible;
                return (sample_wi, pdf, bsdf);
            }
        } else {
            if let Some(sample_wi) = crate::material::util::refract_n(wo, sample_half, self.ior) {
                if sample_wi.z * wo.z <= 0.0 {
                    let visible = crate::material::util::smith_separable_visible(
                        wo.z.abs(),
                        sample_wi.z.abs(),
                        self.roughness_sqr,
                    );
                    let ior_ratio = if wo.z >= 0.0 {
                        1.0 / self.ior
                    } else {
                        self.ior
                    };

                    // let denom = wo.dot(sample_half) + ior_ratio * sample_wi.dot(sample_half);
                    let denom = ior_ratio * wo.dot(sample_half) + sample_wi.dot(sample_half);
                    let denom = denom * denom;
                    let num = 4.0
                        * wo.dot(sample_half).abs()
                        * sample_wi.dot(sample_half).abs();
                    let bsdf = self.transmittance * (1.0 - fresnel) * ndf * visible * num / denom;

                    let num = sample_wi.dot(sample_half).abs();
                    let pdf = (1.0 - fresnel) * ndf * sample_half.z * num / denom;
                    return (sample_wi, pdf, bsdf);
                }
            }
        }

        (wo, 1.0, Color::BLACK)
    }

    fn bsdf(&self, wo: Vector3<f32>, wi: Vector3<f32>) -> Color {
        if wo.z * wi.z >= 0.0 {
            let half = if wo.z >= 0.0 {
                (wo + wi).normalize()
            } else {
                -(wo + wi).normalize()
            };

            let ndoth = half.z;
            let ndotl = wi.z.abs();
            let ndotv = wo.z.abs();

            let ndf = crate::material::util::ggx_ndf(ndoth, self.roughness_sqr);
            let visible =
                crate::material::util::smith_separable_visible(ndotv, ndotl, self.roughness_sqr);
            let fresnel = crate::material::util::fresnel_n(self.ior, wo, half);

            self.reflectance * fresnel * ndf * visible
        } else {
            let half = crate::material::util::half_from_refract(wo, wi, self.ior);
            if wo.dot(half) * wi.dot(half) >= 0.0 {
                return Color::BLACK
            }

            let ior_ratio = if wo.z >= 0.0 {
                1.0 / self.ior
            } else {
                self.ior
            };
            let ndoth = half.z;
            let ndotl = wi.z.abs();
            let ndotv = wo.z.abs();

            let ndf = crate::material::util::ggx_ndf(ndoth, self.roughness_sqr);
            let visible =
                crate::material::util::smith_separable_visible(ndotv, ndotl, self.roughness_sqr);
            let fresnel = crate::material::util::fresnel_n(self.ior, wo, half);

            let denom = ior_ratio * wo.dot(half) + wi.dot(half);
            let denom = denom * denom;
            let num = 4.0 * wo.dot(half).abs() * wi.dot(half).abs();

            self.transmittance * (1.0 - fresnel) * ndf * visible * num / denom
        }
    }

    fn pdf(&self, wo: Vector3<f32>, wi: Vector3<f32>) -> f32 {
        if wo.z * wi.z >= 0.0 {
            let half = if wo.z >= 0.0 {
                (wo + wi).normalize()
            } else {
                -(wo + wi).normalize()
            };

            let fresnel = crate::material::util::fresnel_n(self.ior, wo, half);
            crate::material::util::ggx_ndf(half.z, self.roughness_sqr) * half.z
                / (4.0 * wo.dot(half).abs())
                * fresnel
        } else {
            let half = crate::material::util::half_from_refract(wo, wi, self.ior);
            let ior_ratio = if wo.z >= 0.0 {
                1.0 / self.ior
            } else {
                self.ior
            };

            let fresnel = crate::material::util::fresnel_n(self.ior, wo, half);
            let denom = ior_ratio * wo.dot(half) + wi.dot(half);
            let denom = denom * denom;
            let num = wi.dot(half).abs();
            crate::material::util::ggx_ndf(half.z, self.roughness_sqr)
                * half.z
                * (1.0 - fresnel)
                * num
                / denom
        }
    }

    fn is_delta(&self) -> bool {
        false
    }

    fn emissive(&self) -> Color {
        Color::BLACK
    }
}
