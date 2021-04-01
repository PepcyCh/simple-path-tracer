use crate::core::color::Color;

pub trait Light {
    /// return (sampled direction, pdf, light strength, light dist)
    fn sample(&self, position: cgmath::Point3<f32>) -> (cgmath::Vector3<f32>, f32, Color, f32);

    fn is_delta(&self) -> bool;
}
