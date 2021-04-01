use crate::core::color::Color;

pub trait Light {
    /// return (sampled direction, pdf, L_i, dist to light)
    fn sample_light(
        &self,
        position: cgmath::Point3<f32>,
    ) -> (cgmath::Vector3<f32>, f32, Color, f32);
}
