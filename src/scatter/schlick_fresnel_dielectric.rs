use crate::core::{color::Color, rng::Rng};

use super::{util, Reflect, ScatterT, ScatterType};

pub struct SchlickFresnelDielectric<S, D> {
    f0: Color,
    specular: S,
    diffuse: D,
}

impl<S: Reflect, D: Reflect> SchlickFresnelDielectric<S, D> {
    pub fn new(f0: Color, specular: S, diffuse: D) -> Self {
        Self {
            f0,
            specular,
            diffuse,
        }
    }
}

impl<S: Reflect, D: Reflect> ScatterT for SchlickFresnelDielectric<S, D> {
    fn sample_wi(
        &self,
        po: glam::Vec3A,
        wo: glam::Vec3A,
        pi: glam::Vec3A,
        rng: &mut Rng,
    ) -> (glam::Vec3A, f32, Color, ScatterType) {
        let fresnel = util::schlick_fresnel_with_r0(self.f0, wo.z.abs());
        let fresnel_lum = fresnel.luminance();
        if rng.uniform_1d() < fresnel_lum {
            let (wi, specular_pdf, specular_bxdf, ty) = self.specular.sample_wi(po, wo, pi, rng);
            let diffuse_pdf = self.diffuse.pdf(po, wo, pi, wi);
            let diffuse_bxdf = self.diffuse.bxdf(po, wo, pi, wi);
            let pdf = fresnel_lum * specular_pdf + (1.0 - fresnel_lum) * diffuse_pdf;
            let bxdf = fresnel * specular_bxdf + (Color::WHITE - fresnel) * diffuse_bxdf;
            (wi, pdf, bxdf, ty)
        } else {
            let (wi, diffuse_pdf, diffuse_bxdf, ty) = self.diffuse.sample_wi(po, wo, pi, rng);
            let specular_pdf = self.specular.pdf(po, wo, pi, wi);
            let specular_bxdf = self.specular.bxdf(po, wo, pi, wi);
            let pdf = fresnel_lum * specular_pdf + (1.0 - fresnel_lum) * diffuse_pdf;
            let bxdf = fresnel * specular_bxdf + (Color::WHITE - fresnel) * diffuse_bxdf;
            (wi, pdf, bxdf, ty)
        }
    }

    fn pdf(&self, po: glam::Vec3A, wo: glam::Vec3A, pi: glam::Vec3A, wi: glam::Vec3A) -> f32 {
        if wo.z * wi.z >= 0.0 {
            let fresnel = util::schlick_fresnel_with_r0(self.f0, wo.z.abs()).luminance();
            let specular_pdf = self.specular.pdf(po, wo, pi, wi);
            let diffuse_pdf = self.diffuse.pdf(po, wo, pi, wi);
            fresnel * specular_pdf + (1.0 - fresnel) * diffuse_pdf
        } else {
            1.0
        }
    }

    fn bxdf(&self, po: glam::Vec3A, wo: glam::Vec3A, pi: glam::Vec3A, wi: glam::Vec3A) -> Color {
        if wo.z * wi.z >= 0.0 {
            let fresnel = util::schlick_fresnel_with_r0(self.f0, wo.z.abs());
            let specular_bxdf = self.specular.bxdf(po, wo, pi, wi);
            let diffuse_bxdf = self.diffuse.bxdf(po, wo, pi, wi);
            fresnel * specular_bxdf + (Color::WHITE - fresnel) * diffuse_bxdf
        } else {
            Color::BLACK
        }
    }

    fn is_delta(&self) -> bool {
        self.specular.is_delta() && self.diffuse.is_delta()
    }
}

impl<S: Reflect, D: Reflect> Reflect for SchlickFresnelDielectric<S, D> {}
