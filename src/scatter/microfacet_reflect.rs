use crate::core::{color::Color, rng::Rng};

use super::{util, Reflect, ScatterT, ScatterType};

pub struct MicrofacetReflect {
    reflectance: Color,
    roughness_x: f32,
    roughness_y: f32,
}

impl MicrofacetReflect {
    pub fn new(reflectance: Color, roughness_x: f32, roughness_y: f32) -> Self {
        Self {
            reflectance,
            roughness_x,
            roughness_y,
        }
    }
}

impl ScatterT for MicrofacetReflect {
    fn sample_wi(
        &self,
        _po: glam::Vec3A,
        wo: glam::Vec3A,
        _pi: glam::Vec3A,
        rng: &mut Rng,
    ) -> (glam::Vec3A, f32, Color, ScatterType) {
        let (half, pdf) =
            util::ggx_smith_vndf_sample(wo, self.roughness_x, self.roughness_y, rng.uniform_2d());

        let wi = util::reflect_n(wo, half);
        if wi.z * wo.z >= 0.0 {
            let ndf = util::ggx_ndf_aniso(half, self.roughness_x, self.roughness_y);
            let visible =
                util::smith_separable_visible_aniso(wo, wi, self.roughness_x, self.roughness_y);
            let pdf = pdf / (4.0 * wo.dot(half).abs());
            let bxdf = self.reflectance * ndf * visible;
            (wi, pdf, bxdf, ScatterType::glossy_reflect())
        } else {
            (wo, 1.0, Color::BLACK, ScatterType::glossy_reflect())
        }
    }

    fn pdf(&self, _po: glam::Vec3A, wo: glam::Vec3A, _pi: glam::Vec3A, wi: glam::Vec3A) -> f32 {
        if wo.z * wi.z >= 0.0 {
            let half = util::half_from_reflect(wo, wi);
            let pdf = util::ggx_smith_vndf_pdf(half, wo, self.roughness_x, self.roughness_y);
            pdf / (4.0 * wo.dot(half).abs())
        } else {
            1.0
        }
    }

    fn bxdf(&self, _po: glam::Vec3A, wo: glam::Vec3A, _pi: glam::Vec3A, wi: glam::Vec3A) -> Color {
        if wo.z * wi.z >= 0.0 {
            let half = util::half_from_reflect(wo, wi);

            let ndf = util::ggx_ndf_aniso(half, self.roughness_x, self.roughness_y);
            let visible =
                util::smith_separable_visible_aniso(wo, wi, self.roughness_x, self.roughness_y);

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
