use crate::core::color::Color;
use crate::core::coord::Coordinate;
use crate::core::primitive::Aggregate;
use crate::core::sampler::Sampler;
use cgmath::{Point3, Vector3};

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
    /// return (pi, wi, pdf, bxdf)
    fn sample_wi(
        &self,
        po: Point3<f32>,
        wo: Vector3<f32>,
        pi: Point3<f32>,
        sampler: &mut dyn Sampler,
    ) -> (Vector3<f32>, f32, Color);

    /// only wo -> wi, no po -> wi
    fn pdf(&self, po: Point3<f32>, wo: Vector3<f32>, pi: Point3<f32>, wi: Vector3<f32>) -> f32;

    /// only wo -> wi, no po -> wi
    fn bxdf(&self, po: Point3<f32>, wo: Vector3<f32>, pi: Point3<f32>, wi: Vector3<f32>) -> Color;

    fn is_delta(&self) -> bool;
}

pub trait Reflect: Scatter {}

pub trait Transmit: Scatter {}

pub trait SsReflect: Scatter {}
