use crate::core::{color::Color, coord::Coordinate, primitive::Aggregate, sampler::Sampler};

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

pub trait Scatter {
    /// sample pi
    /// return (pi, coord_pi, pdf, sp)
    fn sample_pi(
        &self,
        po: glam::Vec3A,
        _wo: glam::Vec3A,
        coord_po: Coordinate,
        _sampler: &mut dyn Sampler,
        _scene: &dyn Aggregate,
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
        sampler: &mut dyn Sampler,
    ) -> (glam::Vec3A, f32, Color, ScatterType);

    /// only wo -> wi, no po -> wi
    fn pdf(&self, po: glam::Vec3A, wo: glam::Vec3A, pi: glam::Vec3A, wi: glam::Vec3A) -> f32;

    /// only wo -> wi, no po -> wi
    fn bxdf(&self, po: glam::Vec3A, wo: glam::Vec3A, pi: glam::Vec3A, wi: glam::Vec3A) -> Color;

    fn is_delta(&self) -> bool;
}

pub trait Reflect: Scatter {}

pub trait Transmit: Scatter {}

pub trait SsReflect: Scatter {}

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
