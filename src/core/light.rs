use crate::core::{color::Color, sampler::Sampler};

pub trait Light: Send + Sync {
    /// return (sampled direction, pdf, light strength, light dist)
    fn sample(
        &self,
        position: glam::Vec3A,
        sampler: &mut dyn Sampler,
    ) -> (glam::Vec3A, f32, Color, f32);

    /// return (light strength, light dist, pdf)
    fn strength_dist_pdf(&self, position: glam::Vec3A, wi: glam::Vec3A) -> (Color, f32, f32);

    fn is_delta(&self) -> bool;
}
