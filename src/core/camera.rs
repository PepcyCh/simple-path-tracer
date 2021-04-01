use crate::core::ray::Ray;

pub trait Camera {
    fn generate_ray(&self, point: (f32, f32)) -> Ray;
}
