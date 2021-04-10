use crate::core::ray::Ray;

pub trait Camera: Send + Sync {
    fn generate_ray(&self, point: (f32, f32)) -> Ray;
}
