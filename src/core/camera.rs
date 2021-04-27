use crate::core::ray::{Ray, AuxiliaryRay};

pub trait Camera: Send + Sync {
    fn generate_ray(&self, point: (f32, f32)) -> Ray;

    fn generate_ray_with_aux_ray(&self, point: (f32, f32), offset: (f32, f32)) -> Ray {
        let mut ray = self.generate_ray(point);
        let ray_x = self.generate_ray((point.0 + offset.0, point.1));
        let ray_y = self.generate_ray((point.0, point.1 + offset.1));
        ray.aux_ray = Some(AuxiliaryRay::from_rays(ray_x, ray_y));
        ray
    }
}
