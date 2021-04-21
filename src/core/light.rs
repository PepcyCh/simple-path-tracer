use crate::core::color::Color;
use crate::core::sampler::Sampler;

pub trait Light: Send + Sync {
    /// return (sampled direction, pdf, light strength, light dist)
    fn sample(
        &self,
        position: cgmath::Point3<f32>,
        sampler: &mut dyn Sampler,
    ) -> (cgmath::Vector3<f32>, f32, Color, f32);

    /// return (light strength, light dist, pdf)
    fn strength_dist_pdf(
        &self,
        position: cgmath::Point3<f32>,
        wi: cgmath::Vector3<f32>,
    ) -> (Color, f32, f32);

    fn is_delta(&self) -> bool;
}
