use crate::core::color::Color;
use crate::core::sampler::Sampler;
use cgmath::{Point3, Vector3};

pub trait Medium: Send + Sync {
    /// return (
    ///   sample position pi,
    ///   still in medium or not,
    ///   transport attenuation / transport pdf
    /// )
    fn sample_pi(
        &self,
        po: Point3<f32>,
        wo: Vector3<f32>,
        t_max: f32,
        sampler: &mut dyn Sampler,
    ) -> (Point3<f32>, bool, Color);

    /// return (
    ///   sample direction wi,
    ///   phase pdf
    /// )
    fn sample_wi(&self, wo: Vector3<f32>, sampler: &mut dyn Sampler) -> (Vector3<f32>, f32);

    fn transport_attenuation(&self, dist: f32) -> Color;

    fn phase(&self, wo: Vector3<f32>, wi: Vector3<f32>) -> f32;
}
