use crate::core::{color::Color, rng::Rng};

use super::{Reflect, ScatterT, ScatterType};

pub struct MixScatter<S1, S2> {
    factor1: f32,
    scatter1: S1,
    factor2: f32,
    scatter2: S2,
    ratio: f32,
}

impl<S1: Reflect, S2: Reflect> MixScatter<S1, S2> {
    pub fn new(factor1: f32, scatter1: S1, factor2: f32, scatter2: S2) -> Self {
        let ratio = factor1 / (factor1 + factor2);
        Self {
            factor1,
            scatter1,
            factor2,
            scatter2,
            ratio,
        }
    }
}

impl<S1: Reflect, S2: Reflect> ScatterT for MixScatter<S1, S2> {
    fn sample_wi(
        &self,
        po: glam::Vec3A,
        wo: glam::Vec3A,
        pi: glam::Vec3A,
        rng: &mut Rng,
    ) -> (glam::Vec3A, f32, Color, ScatterType) {
        if rng.uniform_1d() < self.ratio {
            let (wi, pdf1, bxdf1, ty) = self.scatter1.sample_wi(po, wo, pi, rng);
            let pdf2 = self.scatter2.pdf(po, wo, pi, wi);
            let bxdf2 = self.scatter2.bxdf(po, wo, pi, wi);
            let pdf = self.ratio * pdf1 + (1.0 - self.ratio) * pdf2;
            let bxdf = self.factor1 * bxdf1 + self.factor2 * bxdf2;
            (wi, pdf, bxdf, ty)
        } else {
            let (wi, pdf2, bxdf2, ty) = self.scatter2.sample_wi(po, wo, pi, rng);
            let pdf1 = self.scatter1.pdf(po, wo, pi, wi);
            let bxdf1 = self.scatter1.bxdf(po, wo, pi, wi);
            let pdf = self.ratio * pdf1 + (1.0 - self.ratio) * pdf2;
            let bxdf = self.factor1 * bxdf1 + self.factor2 * bxdf2;
            (wi, pdf, bxdf, ty)
        }
    }

    fn pdf(&self, po: glam::Vec3A, wo: glam::Vec3A, pi: glam::Vec3A, wi: glam::Vec3A) -> f32 {
        if wo.z * wi.z >= 0.0 {
            let pdf1 = self.scatter1.pdf(po, wo, pi, wi);
            let pdf2 = self.scatter1.pdf(po, wo, pi, wi);
            self.ratio * pdf1 + (1.0 - self.ratio) * pdf2
        } else {
            1.0
        }
    }

    fn bxdf(&self, po: glam::Vec3A, wo: glam::Vec3A, pi: glam::Vec3A, wi: glam::Vec3A) -> Color {
        if wo.z * wi.z >= 0.0 {
            let bxdf1 = self.scatter1.bxdf(po, wo, pi, wi);
            let bxdf2 = self.scatter1.bxdf(po, wo, pi, wi);
            self.factor1 * bxdf1 + self.factor2 * bxdf2
        } else {
            Color::BLACK
        }
    }

    fn is_delta(&self) -> bool {
        self.scatter1.is_delta() && self.scatter2.is_delta()
    }
}
