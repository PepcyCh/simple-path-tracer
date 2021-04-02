use crate::core::color::Color;

// TODO - add uv param to sample, bsdf, pdf, emissive (vec3 uv for procedure generated texture ?)
pub trait Material {
    /// return (sampled direction, pdf, bsdf)
    fn sample(&self, wo: cgmath::Vector3<f32>) -> (cgmath::Vector3<f32>, f32, Color);

    fn bsdf(&self, wo: cgmath::Vector3<f32>, wi: cgmath::Vector3<f32>) -> Color;

    fn pdf(&self, wo: cgmath::Vector3<f32>, wi: cgmath::Vector3<f32>) -> f32;

    fn is_delta(&self) -> bool;

    fn emissive(&self) -> Color;
}
