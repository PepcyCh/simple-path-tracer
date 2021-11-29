use crate::core::{color::Color, rng::Rng};

use super::{util, Reflect, ScatterT, ScatterType};

pub struct FresnelConductor<R> {
    ior: Color,
    ior_k: Color,
    /// reflect - one of lambert/microfacet/specular reflect
    reflect: R,
}

impl<R: Reflect> FresnelConductor<R> {
    pub fn new(ior: Color, ior_k: Color, reflect: R) -> Self {
        Self {
            ior,
            ior_k,
            reflect,
        }
    }
}

impl<R: Reflect> ScatterT for FresnelConductor<R> {
    fn sample_wi(
        &self,
        po: glam::Vec3A,
        wo: glam::Vec3A,
        pi: glam::Vec3A,
        rng: &mut Rng,
    ) -> (glam::Vec3A, f32, Color, ScatterType) {
        let fresnel = util::fresnel_conductor(self.ior, self.ior_k, wo);
        // let fresnel = util::schlick_fresnel_with_r0(self.ior, wo.z.abs());
        let (wi, pdf, bxdf, ty) = self.reflect.sample_wi(po, wo, pi, rng);
        (wi, pdf, fresnel * bxdf, ty)
    }

    fn pdf(&self, po: glam::Vec3A, wo: glam::Vec3A, pi: glam::Vec3A, wi: glam::Vec3A) -> f32 {
        if wo.z * wi.z >= 0.0 {
            self.reflect.pdf(po, wo, pi, wi)
        } else {
            1.0
        }
    }

    fn bxdf(&self, po: glam::Vec3A, wo: glam::Vec3A, pi: glam::Vec3A, wi: glam::Vec3A) -> Color {
        let fresnel = util::fresnel_conductor(self.ior, self.ior_k, wo);
        // let fresnel = util::schlick_fresnel_with_r0(self.ior, wo.z.abs());
        if wo.z * wi.z >= 0.0 {
            fresnel * self.reflect.bxdf(po, wo, pi, wi)
        } else {
            Color::BLACK
        }
    }

    fn is_delta(&self) -> bool {
        self.reflect.is_delta()
    }
}
