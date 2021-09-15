use crate::core::{camera::Camera, ray::Ray};

pub struct PerspectiveCamera {
    eye: glam::Vec3A,
    forward: glam::Vec3A,
    up: glam::Vec3A,
    right: glam::Vec3A,
    _fov: f32,
    half_cot_half_fov: f32,
}

impl PerspectiveCamera {
    pub fn new(eye: glam::Vec3A, forward: glam::Vec3A, up: glam::Vec3A, fov_deg: f32) -> Self {
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
