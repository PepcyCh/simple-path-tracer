use crate::core::color::Color;
use crate::core::coord::Coordinate;
use crate::core::primitive::Aggregate;
use crate::core::sampler::Sampler;
use cgmath::{Point3, Vector3};

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
        po: Point3<f32>,
        _wo: Vector3<f32>,
        coord_po: Coordinate,
        _sampler: &mut dyn Sampler,
        _scene: &dyn Aggregate,
    ) -> (Point3<f32>, Coordinate, f32, Color) {
        (po, coord_po, 1.0, Color::WHITE)
    }

    /// sample wi at given pi
    /// return (wi, pdf, bxdf, scatter type)
    fn sample_wi(
        &self,
        po: Point3<f32>,
        wo: Vector3<f32>,
        pi: Point3<f32>,
        sampler: &mut dyn Sampler,
    ) -> (Vector3<f32>, f32, Color, ScatterType);

    /// only wo -> wi, no po -> wi
    fn pdf(&self, po: Point3<f32>, wo: Vector3<f32>, pi: Point3<f32>, wi: Vector3<f32>) -> f32;

    /// only wo -> wi, no po -> wi
    fn bxdf(&self, po: Point3<f32>, wo: Vector3<f32>, pi: Point3<f32>, wi: Vector3<f32>) -> Color;

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
