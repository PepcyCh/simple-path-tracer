use crate::core::{color::Color, sampler::Sampler};

pub trait Medium: Send + Sync {
    /// return (
    ///   sample position pi,
    ///   still in medium or not,
    ///   transport attenuation / transport pdf
    /// )
    fn sample_pi(
        &self,
        po: glam::Vec3A,
        wo: glam::Vec3A,
        t_max: f32,
        sampler: &mut dyn Sampler,
    ) -> (glam::Vec3A, bool, Color);

    /// return (
    ///   sample direction wi,
    ///   phase pdf
    /// )
    fn sample_wi(&self, wo: glam::Vec3A, sampler: &mut dyn Sampler) -> (glam::Vec3A, f32);

    fn transport_attenuation(&self, dist: f32) -> Color;

    fn phase(&self, wo: glam::Vec3A, wi: glam::Vec3A) -> f32;
}
