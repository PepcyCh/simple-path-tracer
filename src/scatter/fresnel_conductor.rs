use crate::core::color::Color;
use crate::core::sampler::Sampler;
use crate::core::scatter::{Reflect, Scatter};
use cgmath::{Point3, Vector3};

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

impl<R: Reflect> Scatter for FresnelConductor<R> {
    fn sample_wi(
        &self,
        po: Point3<f32>,
        wo: Vector3<f32>,
        pi: Point3<f32>,
        sampler: &mut dyn Sampler,
    ) -> (Vector3<f32>, f32, Color) {
        // let fresnel = crate::scatter::util::fresnel_conductor(self.ior, self.ior_k, wo);
        let fresnel = crate::scatter::util::schlick_fresnel_with_r0(self.ior, wo.z.abs());
        let (wi, pdf, bxdf) = self.reflect.sample_wi(po, wo, pi, sampler);
        (wi, pdf, fresnel * bxdf)
    }

    fn pdf(&self, po: Point3<f32>, wo: Vector3<f32>, pi: Point3<f32>, wi: Vector3<f32>) -> f32 {
        if wo.z * wi.z >= 0.0 {
            self.reflect.pdf(po, wo, pi, wi)
        } else {
            1.0
        }
    }

    fn bxdf(&self, po: Point3<f32>, wo: Vector3<f32>, pi: Point3<f32>, wi: Vector3<f32>) -> Color {
        // let fresnel = crate::scatter::util::fresnel_conductor(self.ior, self.ior_k, wo);
        let fresnel = crate::scatter::util::schlick_fresnel_with_r0(self.ior, wo.z.abs());
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