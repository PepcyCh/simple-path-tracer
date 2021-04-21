use crate::core::color::Color;
use crate::core::sampler::Sampler;
use crate::core::scatter::{Reflect, Scatter};
use cgmath::{InnerSpace, Point3, Vector3};

pub struct MicrofacetReflect {
    reflectance: Color,
    _roughness: f32,
    roughness_sqr: f32,
}

impl MicrofacetReflect {
    pub fn new(reflectance: Color, roughness: f32) -> Self {
        let roughness_sqr = roughness * roughness;
        Self {
            reflectance,
            _roughness: roughness,
            roughness_sqr,
        }
    }
}

impl Scatter for MicrofacetReflect {
    fn sample_wi(
        &self,
        _po: Point3<f32>,
        wo: Vector3<f32>,
        _pi: Point3<f32>,
        sampler: &mut dyn Sampler,
    ) -> (Vector3<f32>, f32, Color) {
        let (rand_x, rand_y) = sampler.uniform_2d();
        let cos_theta_sqr = crate::scatter::util::ggx_ndf_cdf_inverse(self.roughness_sqr, rand_x);
        let cos_theta = cos_theta_sqr.sqrt();
        let sin_theta = (1.0 - cos_theta_sqr).sqrt();
        let phi = 2.0 * std::f32::consts::PI * rand_y;
        let half = Vector3::new(sin_theta * phi.cos(), sin_theta * phi.sin(), cos_theta);

        let wi = crate::scatter::util::reflect_n(wo, half);
        if wi.z * wo.z >= 0.0 {
            let ndf = crate::scatter::util::ggx_ndf(half.z, self.roughness_sqr);
            let visible = crate::scatter::util::smith_separable_visible(
                wo.z.abs(),
                wi.z.abs(),
                self.roughness_sqr,
            );
            let pdf = ndf * half.z / (4.0 * wo.dot(half).abs());
            let bxdf = self.reflectance * ndf * visible;
            (wi, pdf, bxdf)
        } else {
            (wo, 1.0, Color::BLACK)
        }
    }

    fn pdf(&self, _po: Point3<f32>, wo: Vector3<f32>, _pi: Point3<f32>, wi: Vector3<f32>) -> f32 {
        if wo.z * wi.z >= 0.0 {
            let half = crate::scatter::util::half_from_reflect(wo, wi);
            let ndf = crate::scatter::util::ggx_ndf(half.z, self.roughness_sqr);
            ndf * half.z / (4.0 * wo.dot(half).abs())
        } else {
            1.0
        }
    }

    fn bxdf(
        &self,
        _po: Point3<f32>,
        wo: Vector3<f32>,
        _pi: Point3<f32>,
        wi: Vector3<f32>,
    ) -> Color {
        if wo.z * wi.z >= 0.0 {
            let half = crate::scatter::util::half_from_reflect(wo, wi);

            let ndf = crate::scatter::util::ggx_ndf(half.z, self.roughness_sqr);
            let visible = crate::scatter::util::smith_separable_visible(
                wo.z.abs(),
                wi.z.abs(),
                self.roughness_sqr,
            );

            self.reflectance * ndf * visible
        } else {
            Color::BLACK
        }
    }

    fn is_delta(&self) -> bool {
        false
    }
}

impl Reflect for MicrofacetReflect {}
