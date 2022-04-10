mod util;

mod lambert;
mod pseudo;

mod fresnel;
mod microfacet;
mod microfacet_conductor;
mod microfacet_dielectric;
mod microfacet_plastic;
mod pndf_bvh;
mod substrate;

mod specular_conductor;
mod specular_dielectric;
mod specular_plastic;

pub use lambert::*;
pub use pseudo::*;

pub use fresnel::*;
pub use microfacet::*;
pub use microfacet_conductor::*;
pub use microfacet_dielectric::*;
pub use microfacet_plastic::*;
pub use pndf_bvh::*;
pub use substrate::*;

pub use specular_conductor::*;
pub use specular_dielectric::*;
pub use specular_plastic::*;

use crate::{
    core::{color::Color, coord::Coordinate, rng::Rng},
    primitive::Primitive,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BxdfLobeType {
    Diffuse,
    Glossy,
    Specular,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BxdfDirType {
    Reflect,
    Transmit,
}

#[derive(Debug, Clone, Copy)]
pub struct BxdfSampleType {
    pub lobe: BxdfLobeType,
    pub dir: BxdfDirType,
    pub subsurface: bool,
}

pub struct BxdfInputs<'a> {
    pub po: glam::Vec3A,
    pub coord_po: Coordinate,
    pub wo: glam::Vec3A,
    pub scene: &'a Primitive,
}

pub struct BxdfSubsurfaceSample {
    pub pi: glam::Vec3A,
    pub coord_pi: Coordinate,
    pub sp: Color,
    pub pdf_pi: f32,
}

pub struct BxdfSample {
    pub wi: glam::Vec3A,
    pub ty: BxdfSampleType,
    pub bxdf: Color,
    pub pdf: f32,
    pub subsurface: Option<BxdfSubsurfaceSample>,
}

#[enum_dispatch::enum_dispatch(Bxdf)]
pub trait BxdfT {
    fn sample<'a>(&self, inputs: &'a BxdfInputs, rng: &mut Rng) -> BxdfSample;

    /// only wo -> wi, no po -> wi
    fn pdf(&self, wo: glam::Vec3A, wi: glam::Vec3A) -> f32;

    fn bxdf(&self, wo: glam::Vec3A, wi: glam::Vec3A) -> Color;

    fn is_delta(&self) -> bool;
}

#[enum_dispatch::enum_dispatch]
pub enum Bxdf {
    Lambert,
    Pseudo,
    MicrofacetConductor,
    MicrofacetDielectric,
    MicrofacetPlastic,
    SpecularConductor,
    SpecularDielectric,
    SpecularPlastic,
}
