use crate::core::{color::Color, rng::Rng};

use super::{util, Reflect, ScatterT, ScatterType};

pub struct SchlickFresnelMetal<S> {
    f0: Color,
    specular: S,
}

impl<S: Reflect> SchlickFresnelMetal<S> {
    pub fn new(f0: Color, specular: S) -> Self {
        Self { f0, specular }
    }
}

impl<S: Reflect> ScatterT for SchlickFresnelMetal<S> {
    fn sample_wi(
        &self,
        po: glam::Vec3A,
        wo: glam::Vec3A,
        pi: glam::Vec3A,
        rng: &mut Rng,
    ) -> (glam::Vec3A, f32, Color, ScatterType) {
        let fresnel = util::schlick_fresnel_with_r0(self.f0, wo.z.abs());
        let (wi, pdf, bxdf, ty) = self.specular.sample_wi(po, wo, pi, rng);
        (wi, pdf, bxdf * fresnel, ty)
    }

    fn pdf(&self, po: glam::Vec3A, wo: glam::Vec3A, pi: glam::Vec3A, wi: glam::Vec3A) -> f32 {
        if wo.z * wi.z >= 0.0 {
            self.specular.pdf(po, wo, pi, wi)
        } else {
            1.0
        }
    }

    fn bxdf(&self, po: glam::Vec3A, wo: glam::Vec3A, pi: glam::Vec3A, wi: glam::Vec3A) -> Color {
        let fresnel = util::schlick_fresnel_with_r0(self.f0, wo.z.abs());
        if wo.z * wi.z >= 0.0 {
            fresnel * self.specular.bxdf(po, wo, pi, wi)
        } else {
            Color::BLACK
        }
    }

    fn is_delta(&self) -> bool {
        self.specular.is_delta()
    }
}

impl<S: Reflect> Reflect for SchlickFresnelMetal<S> {}
