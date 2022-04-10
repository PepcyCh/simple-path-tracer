use crate::core::color::Color;

use super::util;

#[enum_dispatch::enum_dispatch(Fresnel)]
pub trait FresnelT {
    fn fresnel(&self, i: glam::Vec3A, n: glam::Vec3A) -> Color;

    fn ior(&self) -> f32;
}

#[enum_dispatch::enum_dispatch]
pub enum Fresnel {
    DielectricFresnel,
    ConductorFresnel,
    SchlickFresnel,
}

pub struct DielectricFresnel {
    ior: f32,
}

impl DielectricFresnel {
    pub fn new(ior: f32) -> Self {
        Self { ior }
    }
}

impl FresnelT for DielectricFresnel {
    fn fresnel(&self, i: glam::Vec3A, n: glam::Vec3A) -> Color {
        Color::gray(util::fresnel_n(self.ior, i, n))
    }

    fn ior(&self) -> f32 {
        self.ior
    }
}

pub struct ConductorFresnel {
    eta: Color,
    k: Color,
}

impl ConductorFresnel {
    pub fn new(eta: Color, k: Color) -> Self {
        Self { eta, k }
    }
}

impl FresnelT for ConductorFresnel {
    fn fresnel(&self, i: glam::Vec3A, n: glam::Vec3A) -> Color {
        util::fresnel_conductor_n(self.eta, self.k, i, n)
    }

    fn ior(&self) -> f32 {
        // unreachable
        1.0
    }
}

pub struct SchlickFresnel {
    r0: Color,
}

impl SchlickFresnel {
    pub fn new(r0: Color) -> Self {
        Self { r0 }
    }
}

impl FresnelT for SchlickFresnel {
    fn fresnel(&self, i: glam::Vec3A, n: glam::Vec3A) -> Color {
        util::schlick_fresnel_with_r0(self.r0, i.dot(n))
    }

    fn ior(&self) -> f32 {
        let sqrt_r0 = self.r0.luminance().sqrt();
        (1.0 - sqrt_r0) / (1.0 + sqrt_r0)
    }
}
