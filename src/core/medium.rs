use crate::core::color::Color;
use cgmath::{Point3, Vector3};

pub trait Medium: Send + Sync {
    /// return (sample position, sample direction, still in medium or not, pdf, attenuation)
    fn sample(
        &self,
        position: Point3<f32>,
        wo: Vector3<f32>,
        t_max: f32,
    ) -> (Point3<f32>, Vector3<f32>, bool, f32, Color);
}
