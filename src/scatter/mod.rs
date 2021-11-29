mod util;

mod fresnel_conductor;
mod fresnel_dielectric;
mod lambert_reflect;
mod lambert_transmit;
mod microfacet_reflect;
mod microfacet_transmit;
mod specular_reflect;
mod specular_transmit;
mod subsurface_reflect;

pub use fresnel_conductor::*;
pub use fresnel_dielectric::*;
pub use lambert_reflect::*;
pub use lambert_transmit::*;
pub use microfacet_reflect::*;
pub use microfacet_transmit::*;
pub use specular_reflect::*;
pub use specular_transmit::*;
pub use subsurface_reflect::*;

use crate::{
    core::{color::Color, coord::Coordinate, rng::Rng},
    primitive::Primitive,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScatterLobeType {
    Lambert,
    Glossy,
    Specular,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScatterDirType {
    Reflect,
    Transmit,
}

#[derive(Debug, Clone, Copy)]
pub struct ScatterType {
    pub lobe: ScatterLobeType,
    pub dir: ScatterDirType,
}

#[enum_dispatch::enum_dispatch(Scatter)]
pub trait ScatterT {
    /// sample pi
    /// return (pi, coord_pi, pdf, sp)
    fn sample_pi(
        &self,
        po: glam::Vec3A,
        _wo: glam::Vec3A,
        coord_po: Coordinate,
        _rng: &mut Rng,
        _scene: &Primitive,
    ) -> (glam::Vec3A, Coordinate, f32, Color) {
        (po, coord_po, 1.0, Color::WHITE)
    }

    /// sample wi at given pi
    /// return (wi, pdf, bxdf, scatter type)
    fn sample_wi(
        &self,
        po: glam::Vec3A,
        wo: glam::Vec3A,
        pi: glam::Vec3A,
        rng: &mut Rng,
    ) -> (glam::Vec3A, f32, Color, ScatterType);

    /// only wo -> wi, no po -> wi
    fn pdf(&self, po: glam::Vec3A, wo: glam::Vec3A, pi: glam::Vec3A, wi: glam::Vec3A) -> f32;

    /// only wo -> wi, no po -> wi
    fn bxdf(&self, po: glam::Vec3A, wo: glam::Vec3A, pi: glam::Vec3A, wi: glam::Vec3A) -> Color;

    fn is_delta(&self) -> bool;
}

pub trait Reflect: ScatterT {}

pub trait Transmit: ScatterT {}

pub trait SsReflect: ScatterT {}

impl ScatterType {
    pub fn specular_reflect() -> Self {
        Self {
            lobe: ScatterLobeType::Specular,
            dir: ScatterDirType::Reflect,
        }
    }

    pub fn glossy_reflect() -> Self {
        Self {
            lobe: ScatterLobeType::Glossy,
            dir: ScatterDirType::Reflect,
        }
    }

    pub fn lambert_reflect() -> Self {
        Self {
            lobe: ScatterLobeType::Lambert,
            dir: ScatterDirType::Reflect,
        }
    }

    pub fn specular_transmit() -> Self {
        Self {
            lobe: ScatterLobeType::Specular,
            dir: ScatterDirType::Transmit,
        }
    }

    pub fn glossy_transmit() -> Self {
        Self {
            lobe: ScatterLobeType::Glossy,
            dir: ScatterDirType::Transmit,
        }
    }

    pub fn lambert_transmit() -> Self {
        Self {
            lobe: ScatterLobeType::Lambert,
            dir: ScatterDirType::Transmit,
        }
    }
}

#[enum_dispatch::enum_dispatch]
pub enum Scatter {
    MicrofacetConductor(FresnelConductor<MicrofacetReflect>),
    SpecularConductor(FresnelConductor<SpecularReflect>),
    MicrofacetLambertDielectric(FresnelDielectricRR<MicrofacetReflect, LambertReflect>),
    SpecularLambertDielectric(FresnelDielectricRR<SpecularReflect, LambertReflect>),
    MicrofacetTransmittiveDielectric(FresnelDielectricRT<MicrofacetReflect, MicrofacetTransmit>),
    SpecularTransmittiveDielectric(FresnelDielectricRT<SpecularReflect, SpecularTransmit>),
    MicrofacetSubsurfaceDielectric(FresnelDielectricRSsr<MicrofacetReflect, SubsurfaceReflect>),
    SpecularSubsurfaceDielectric(FresnelDielectricRSsr<SpecularReflect, SubsurfaceReflect>),
    LambertReflect,
    LambertTransmit,
    MicrofacetReflect,
    MicrofacetTransmit,
    SpecularReflect,
    SpecularTransmit,
    SubsurfaceReflect,
}
