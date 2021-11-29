use crate::{
    core::{color::Color, coord::Coordinate, rng::Rng},
    primitive::Primitive,
};

use super::{util, Reflect, ScatterT, ScatterType, SsReflect, Transmit};

struct FresnelDielectric<R, T> {
    ior: f32,
    /// reflect - one of lambert/microfacet/specular reflect
    reflect: R,
    /// transmit - one of lambert/microfacet/specular transmit, lambert/subsurface reflect
    transmit: T,
}

pub struct FresnelDielectricRT<R: Reflect, T: Transmit>(FresnelDielectric<R, T>);

impl<R: Reflect, T: Transmit> FresnelDielectricRT<R, T> {
    pub fn new(ior: f32, reflect: R, transmit: T) -> Self {
        Self(FresnelDielectric {
            ior,
            reflect,
            transmit,
        })
    }
}

pub struct FresnelDielectricRR<R: Reflect, T: Reflect>(FresnelDielectric<R, T>);

impl<R: Reflect, T: Reflect> FresnelDielectricRR<R, T> {
    pub fn new(ior: f32, reflect: R, transmit: T) -> Self {
        Self(FresnelDielectric {
            ior,
            reflect,
            transmit,
        })
    }
}

pub struct FresnelDielectricRSsr<R: Reflect, T: SsReflect>(FresnelDielectric<R, T>);

impl<R: Reflect, T: SsReflect> FresnelDielectricRSsr<R, T> {
    pub fn new(ior: f32, reflect: R, transmit: T) -> Self {
        Self(FresnelDielectric {
            ior,
            reflect,
            transmit,
        })
    }
}

impl<R: Reflect, T: Transmit> ScatterT for FresnelDielectricRT<R, T> {
    fn sample_wi(
        &self,
        po: glam::Vec3A,
        wo: glam::Vec3A,
        pi: glam::Vec3A,
        rng: &mut Rng,
    ) -> (glam::Vec3A, f32, Color, ScatterType) {
        // TODO - half vector fresnel ?
        let fresnel = util::fresnel(self.0.ior, wo);
        let rand = rng.uniform_1d();
        if rand <= fresnel {
            let (wi, pdf, bxdf, ty) = self.0.reflect.sample_wi(po, wo, pi, rng);
            (wi, fresnel * pdf, fresnel * bxdf, ty)
        } else {
            let (wi, pdf, bxdf, ty) = self.0.transmit.sample_wi(po, wo, pi, rng);
            (wi, (1.0 - fresnel) * pdf, (1.0 - fresnel) * bxdf, ty)
        }
    }

    fn pdf(&self, po: glam::Vec3A, wo: glam::Vec3A, pi: glam::Vec3A, wi: glam::Vec3A) -> f32 {
        let fresnel = util::fresnel(self.0.ior, wo);
        if wo.z * wi.z >= 0.0 {
            fresnel * self.0.reflect.pdf(po, wo, pi, wi)
        } else {
            (1.0 - fresnel) * self.0.transmit.pdf(po, wo, pi, wi)
        }
    }

    fn bxdf(&self, po: glam::Vec3A, wo: glam::Vec3A, pi: glam::Vec3A, wi: glam::Vec3A) -> Color {
        let fresnel = util::fresnel(self.0.ior, wo);
        if wo.z * wi.z >= 0.0 {
            fresnel * self.0.reflect.bxdf(po, wo, pi, wi)
        } else {
            (1.0 - fresnel) * self.0.transmit.bxdf(po, wo, pi, wi)
        }
    }

    fn is_delta(&self) -> bool {
        self.0.reflect.is_delta() && self.0.transmit.is_delta()
    }
}

impl<R: Reflect, T: Reflect> ScatterT for FresnelDielectricRR<R, T> {
    fn sample_wi(
        &self,
        po: glam::Vec3A,
        wo: glam::Vec3A,
        pi: glam::Vec3A,
        rng: &mut Rng,
    ) -> (glam::Vec3A, f32, Color, ScatterType) {
        let fresnel = util::fresnel(self.0.ior, wo);
        let rand = rng.uniform_1d();
        if rand <= fresnel {
            let (wi, reflect_pdf, reflect_bxdf, ty) = self.0.reflect.sample_wi(po, wo, pi, rng);
            let transmit_pdf = self.0.transmit.pdf(po, wo, pi, wi);
            let transmit_bxdf = self.0.transmit.bxdf(po, wo, pi, wi);
            let pdf = fresnel * reflect_pdf + (1.0 - fresnel) * transmit_pdf;
            let bxdf = fresnel * reflect_bxdf + (1.0 - fresnel) * transmit_bxdf;
            (wi, pdf, bxdf, ty)
        } else {
            let (wi, transmit_pdf, transmit_bxdf, ty) = self.0.transmit.sample_wi(po, wo, pi, rng);
            let reflect_pdf = self.0.reflect.pdf(po, wo, pi, wi);
            let reflect_bxdf = self.0.reflect.bxdf(po, wo, pi, wi);
            let pdf = fresnel * reflect_pdf + (1.0 - fresnel) * transmit_pdf;
            let bxdf = fresnel * reflect_bxdf + (1.0 - fresnel) * transmit_bxdf;
            (wi, pdf, bxdf, ty)
        }
    }

    fn pdf(&self, po: glam::Vec3A, wo: glam::Vec3A, pi: glam::Vec3A, wi: glam::Vec3A) -> f32 {
        if wo.z * wi.z >= 0.0 {
            let fresnel = util::fresnel(self.0.ior, wo);
            let reflect_pdf = self.0.reflect.pdf(po, wo, pi, wi);
            let transmit_pdf = self.0.transmit.pdf(po, wo, pi, wi);
            fresnel * reflect_pdf + (1.0 - fresnel) * transmit_pdf
        } else {
            1.0
        }
    }

    fn bxdf(&self, po: glam::Vec3A, wo: glam::Vec3A, pi: glam::Vec3A, wi: glam::Vec3A) -> Color {
        if wo.z * wi.z >= 0.0 {
            let fresnel = util::fresnel(self.0.ior, wo);
            let reflect_bxdf = self.0.reflect.bxdf(po, wo, pi, wi);
            let transmit_bxdf = self.0.transmit.bxdf(po, wo, pi, wi);
            fresnel * reflect_bxdf + (1.0 - fresnel) * transmit_bxdf
        } else {
            Color::BLACK
        }
    }

    fn is_delta(&self) -> bool {
        self.0.reflect.is_delta() && self.0.transmit.is_delta()
    }
}

impl<R: Reflect, T: SsReflect> ScatterT for FresnelDielectricRSsr<R, T> {
    fn sample_pi(
        &self,
        po: glam::Vec3A,
        wo: glam::Vec3A,
        coord_po: Coordinate,
        rng: &mut Rng,
        scene: &Primitive,
    ) -> (glam::Vec3A, Coordinate, f32, Color) {
        let fresnel = util::fresnel(self.0.ior, wo);
        let rand = rng.uniform_1d();
        if rand <= fresnel {
            (po, coord_po, 1.0, Color::WHITE)
        } else {
            self.0.transmit.sample_pi(po, wo, coord_po, rng, scene)
        }
    }

    fn sample_wi(
        &self,
        po: glam::Vec3A,
        wo: glam::Vec3A,
        pi: glam::Vec3A,
        rng: &mut Rng,
    ) -> (glam::Vec3A, f32, Color, ScatterType) {
        let fresnel = util::fresnel(self.0.ior, wo);
        if po == pi {
            let (wi, pdf, bxdf, ty) = self.0.reflect.sample_wi(po, wo, pi, rng);
            (wi, fresnel * pdf, fresnel * bxdf, ty)
        } else {
            let (wi, pdf, bxdf, ty) = self.0.transmit.sample_wi(po, wo, pi, rng);
            (wi, (1.0 - fresnel) * pdf, (1.0 - fresnel) * bxdf, ty)
        }
    }

    fn pdf(&self, po: glam::Vec3A, wo: glam::Vec3A, pi: glam::Vec3A, wi: glam::Vec3A) -> f32 {
        let fresnel = util::fresnel(self.0.ior, wo);
        if po == pi {
            fresnel * self.0.reflect.pdf(po, wo, pi, wi)
        } else {
            (1.0 - fresnel) * self.0.transmit.pdf(po, wo, pi, wi)
        }
    }

    fn bxdf(&self, po: glam::Vec3A, wo: glam::Vec3A, pi: glam::Vec3A, wi: glam::Vec3A) -> Color {
        let fresnel = util::fresnel(self.0.ior, wo);
        if po == pi {
            fresnel * self.0.reflect.bxdf(po, wo, pi, wi)
        } else {
            (1.0 - fresnel) * self.0.transmit.bxdf(po, wo, pi, wi)
        }
    }

    fn is_delta(&self) -> bool {
        self.0.reflect.is_delta() && self.0.transmit.is_delta()
    }
}
