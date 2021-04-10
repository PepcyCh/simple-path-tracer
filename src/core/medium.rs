use crate::core::color::Color;
use cgmath::{Point3, Vector3};

pub trait Medium: Send + Sync {
    /// return (
    ///   sample position,
    ///   still in medium or not,
    ///   transport attenuation / transport pdf
    /// )
    fn sample_transport(
        &self,
        position: Point3<f32>,
        wo: Vector3<f32>,
        t_max: f32,
    ) -> (Point3<f32>, bool, Color);

    /// return (
    ///   sample direction,
    ///   phase pdf
    /// )
    fn sample_phase(&self, wo: Vector3<f32>) -> (Vector3<f32>, f32);

    fn transport_attenuation(&self, dist: f32) -> Color;

    fn phase(&self, wo: Vector3<f32>, wi: Vector3<f32>) -> f32;
}
