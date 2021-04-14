use crate::core::color::Color;
use crate::core::primitive::Aggregate;

// TODO - add uv param to sample, bsdf, pdf, emissive (vec3 uv for procedure generated texture ?)
pub trait Material: Send + Sync {
    /// return (sampled direction, pdf, bsdf)
    fn sample(&self, wo: cgmath::Vector3<f32>) -> (cgmath::Vector3<f32>, f32, Color);

    fn bsdf(&self, wo: cgmath::Vector3<f32>, wi: cgmath::Vector3<f32>) -> Color;

    fn pdf(&self, wo: cgmath::Vector3<f32>, wi: cgmath::Vector3<f32>) -> f32;

    fn is_delta(&self) -> bool;

    fn emissive(&self) -> Color;

    /// return (sampled position, normal of that position, Sp / pdf)
    fn sample_sp(
        &self,
        p: cgmath::Point3<f32>,
        _wo: cgmath::Vector3<f32>,
        normal_to_world: cgmath::Matrix3<f32>,
        _scene: &dyn Aggregate,
    ) -> (cgmath::Point3<f32>, cgmath::Vector3<f32>, Color) {
        (p, normal_to_world * cgmath::Vector3::unit_z(), Color::WHITE)
    }
}
