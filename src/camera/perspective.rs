use crate::core::camera::Camera;
use crate::core::ray::Ray;
use cgmath::InnerSpace;

pub struct PerspectiveCamera {
    eye: cgmath::Point3<f32>,
    forward: cgmath::Vector3<f32>,
    up: cgmath::Vector3<f32>,
    right: cgmath::Vector3<f32>,
    _fov: f32,
    half_cot_half_fov: f32,
}

impl PerspectiveCamera {
    pub fn new(
        eye: cgmath::Point3<f32>,
        forward: cgmath::Vector3<f32>,
        up: cgmath::Vector3<f32>,
        fov_deg: f32,
    ) -> Self {
        let forward = forward.normalize();
        let right = forward.cross(up).normalize();
        let up = right.cross(forward);
        let fov = fov_deg * std::f32::consts::PI / 180.0;
        Self {
            eye,
            forward,
            up,
            right,
            _fov: fov,
            half_cot_half_fov: 0.5 / (fov * 0.5).tan(),
        }
    }
}

impl Camera for PerspectiveCamera {
    fn generate_ray(&self, point: (f32, f32)) -> Ray {
        let origin = self.eye;
        let direction =
            (self.forward * self.half_cot_half_fov + self.right * point.0 + self.up * point.1)
                .normalize();
        Ray::new(origin, direction)
    }
}
