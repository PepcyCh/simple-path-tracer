use crate::core::{loader::InputParams, ray::Ray, scene::Scene};

use super::CameraT;

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

    pub fn load(_scene: &Scene, params: &mut InputParams) -> anyhow::Result<Self> {
        let eye = params.get_float3("eye")?.into();
        let forward = params.get_float3("forward")?.into();
        let up = params.get_float3("up")?.into();
        let fov_deg = params.get_float("fov")?;

        Ok(PerspectiveCamera::new(eye, forward, up, fov_deg))
    }
}

impl CameraT for PerspectiveCamera {
    fn generate_ray(&self, point: (f32, f32)) -> Ray {
        let origin = self.eye;
        let direction =
            (self.forward * self.half_cot_half_fov + self.right * point.0 + self.up * point.1)
                .normalize();
        Ray::new(origin, direction)
    }
}
