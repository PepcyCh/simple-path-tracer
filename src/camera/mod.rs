mod perspective;

pub use perspective::*;

use crate::core::{
    loader::InputParams,
    ray::{AuxiliaryRay, Ray},
    scene::Scene,
};

#[enum_dispatch::enum_dispatch(Camera)]
pub trait CameraT: Send + Sync {
    fn generate_ray(&self, point: (f32, f32)) -> Ray;

    fn generate_ray_with_aux_ray(&self, point: (f32, f32), offset: (f32, f32)) -> Ray {
        let mut ray = self.generate_ray(point);
        let ray_x = self.generate_ray((point.0 + offset.0, point.1));
        let ray_y = self.generate_ray((point.0, point.1 + offset.1));
        ray.aux_ray = Some(AuxiliaryRay::from_rays(ray_x, ray_y));
        ray
    }
}

#[enum_dispatch::enum_dispatch]
pub enum Camera {
    PerspectiveCamera,
}

pub fn create_camera_from_params(
    scene: &mut Scene,
    params: &mut InputParams,
) -> anyhow::Result<()> {
    params.set_name("camera".into());
    let ty = params.get_str("type")?;
    let name = params.get_str("name")?;
    params.set_name(format!("camera-{}-{}", ty, name).into());

    let res = match ty.as_str() {
        "perspective" => PerspectiveCamera::load(scene, params)?.into(),
        _ => anyhow::bail!(format!("{}: unknown type '{}'", params.name(), ty)),
    };

    scene.add_camera(name, res)?;

    params.check_unused_keys();

    Ok(())
}
