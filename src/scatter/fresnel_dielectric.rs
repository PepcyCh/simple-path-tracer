use crate::core::color::Color;
use crate::core::coord::Coordinate;
use crate::core::primitive::Aggregate;
use crate::core::sampler::Sampler;
use crate::core::scatter::{Reflect, Scatter, SsReflect, Transmit};
use cgmath::{Point3, Vector3};

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

impl<R: Reflect, T: Transmit> Scatter for FresnelDielectricRT<R, T> {
    fn sample_wi(
        &self,
        po: Point3<f32>,
        wo: Vector3<f32>,
        pi: Point3<f32>,
        sampler: &mut dyn Sampler,
    ) -> (Vector3<f32>, f32, Color) {
        // TODO - half vector fresnel ?
        let fresnel = crate::scatter::util::fresnel(self.0.ior, wo);
        let rand = sampler.uniform_1d();
        if rand <= fresnel {
            let (wi, pdf, bxdf) = self.0.reflect.sample_wi(po, wo, pi, sampler);
            (wi, fresnel * pdf, fresnel * bxdf)
        } else {
            let (wi, pdf, bxdf) = self.0.transmit.sample_wi(po, wo, pi, sampler);
            (wi, (1.0 - fresnel) * pdf, (1.0 - fresnel) * bxdf)
        }
    }

    fn pdf(&self, po: Point3<f32>, wo: Vector3<f32>, pi: Point3<f32>, wi: Vector3<f32>) -> f32 {
        let fresnel = crate::scatter::util::fresnel(self.0.ior, wo);
        if wo.z * wi.z >= 0.0 {
            fresnel * self.0.reflect.pdf(po, wo, pi, wi)
        } else {
            (1.0 - fresnel) * self.0.transmit.pdf(po, wo, pi, wi)
        }
    }

    fn bxdf(&self, po: Point3<f32>, wo: Vector3<f32>, pi: Point3<f32>, wi: Vector3<f32>) -> Color {
        let fresnel = crate::scatter::util::fresnel(self.0.ior, wo);
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

impl<R: Reflect, T: Reflect> Scatter for FresnelDielectricRR<R, T> {
    fn sample_wi(
        &self,
        po: Point3<f32>,
        wo: Vector3<f32>,
        pi: Point3<f32>,
        sampler: &mut dyn Sampler,
    ) -> (Vector3<f32>, f32, Color) {
        let fresnel = crate::scatter::util::fresnel(self.0.ior, wo);
        let rand = sampler.uniform_1d();
        if rand <= fresnel {
            let (wi, reflect_pdf, reflect_bxdf) = self.0.reflect.sample_wi(po, wo, pi, sampler);
            let transmit_pdf = self.0.transmit.pdf(po, wo, pi, wi);
            let transmit_bxdf = self.0.transmit.bxdf(po, wo, pi, wi);
            let pdf = fresnel * reflect_pdf + (1.0 - fresnel) * transmit_pdf;
            let bxdf = fresnel * reflect_bxdf + (1.0 - fresnel) * transmit_bxdf;
            (wi, pdf, bxdf)
        } else {
            let (wi, transmit_pdf, transmit_bxdf) = self.0.transmit.sample_wi(po, wo, pi, sampler);
            let reflect_pdf = self.0.reflect.pdf(po, wo, pi, wi);
            let reflect_bxdf = self.0.reflect.bxdf(po, wo, pi, wi);
            let pdf = fresnel * reflect_pdf + (1.0 - fresnel) * transmit_pdf;
            let bxdf = fresnel * reflect_bxdf + (1.0 - fresnel) * transmit_bxdf;
            (wi, pdf, bxdf)
        }
    }

    fn pdf(&self, po: Point3<f32>, wo: Vector3<f32>, pi: Point3<f32>, wi: Vector3<f32>) -> f32 {
        if wo.z * wi.z >= 0.0 {
            let fresnel = crate::scatter::util::fresnel(self.0.ior, wo);
            let reflect_pdf = self.0.reflect.pdf(po, wo, pi, wi);
            let transmit_pdf = self.0.transmit.pdf(po, wo, pi, wi);
            fresnel * reflect_pdf + (1.0 - fresnel) * transmit_pdf
        } else {
            1.0
        }
    }

    fn bxdf(&self, po: Point3<f32>, wo: Vector3<f32>, pi: Point3<f32>, wi: Vector3<f32>) -> Color {
        if wo.z * wi.z >= 0.0 {
            let fresnel = crate::scatter::util::fresnel(self.0.ior, wo);
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

impl<R: Reflect, T: SsReflect> Scatter for FresnelDielectricRSsr<R, T> {
    fn sample_pi(
        &self,
        po: Point3<f32>,
        wo: Vector3<f32>,
        coord_po: Coordinate,
        sampler: &mut dyn Sampler,
        scene: &dyn Aggregate,
    ) -> (Point3<f32>, Coordinate, f32, Color) {
        let fresnel = crate::scatter::util::fresnel(self.0.ior, wo);
        let rand = sampler.uniform_1d();
        if rand <= fresnel {
            (po, coord_po, 1.0, Color::WHITE)
        } else {
            self.0.transmit.sample_pi(po, wo, coord_po, sampler, scene)
        }
    }

    fn sample_wi(
        &self,
        po: Point3<f32>,
        wo: Vector3<f32>,
        pi: Point3<f32>,
        sampler: &mut dyn Sampler,
    ) -> (Vector3<f32>, f32, Color) {
        let fresnel = crate::scatter::util::fresnel(self.0.ior, wo);
        if po == pi {
            let (wi, pdf, bxdf) = self.0.reflect.sample_wi(po, wo, pi, sampler);
            (wi, fresnel * pdf, bxdf)
        } else {
            let (wi, pdf, bxdf) = self.0.transmit.sample_wi(po, wo, pi, sampler);
            (wi, (1.0 - fresnel) * pdf, bxdf)
        }
    }

    fn pdf(&self, po: Point3<f32>, wo: Vector3<f32>, pi: Point3<f32>, wi: Vector3<f32>) -> f32 {
        let fresnel = crate::scatter::util::fresnel(self.0.ior, wo);
        if po == pi {
            fresnel * self.0.reflect.pdf(po, wo, pi, wi)
        } else {
            (1.0 - fresnel) * self.0.transmit.pdf(po, wo, pi, wi)
        }
    }

    fn bxdf(&self, po: Point3<f32>, wo: Vector3<f32>, pi: Point3<f32>, wi: Vector3<f32>) -> Color {
        let fresnel = crate::scatter::util::fresnel(self.0.ior, wo);
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
