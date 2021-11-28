mod directional;
mod environment;
mod point;
mod shape_light;

pub use directional::*;
pub use environment::*;
pub use point::*;
pub use shape_light::*;

use crate::core::{color::Color, loader::InputParams, rng::Rng, scene::Scene};

#[enum_dispatch::enum_dispatch(Light)]
pub trait LightT: Send + Sync {
    /// return (sampled direction, pdf, light strength, light dist)
    fn sample(&self, position: glam::Vec3A, sampler: &mut Rng) -> (glam::Vec3A, f32, Color, f32);

    /// return (light strength, light dist, pdf)
    fn strength_dist_pdf(&self, position: glam::Vec3A, wi: glam::Vec3A) -> (Color, f32, f32);

    fn is_delta(&self) -> bool;
}

#[enum_dispatch::enum_dispatch]
pub enum Light {
    DirLight,
    EnvLight,
    PointLight,
    ShapeLight,
}

pub fn create_light_from_params(scene: &mut Scene, params: &mut InputParams) -> anyhow::Result<()> {
    params.set_name("light".into());
    let ty = params.get_str("type")?;
    let name = params.get_str("name")?;
    params.set_name(format!("light-{}-{}", ty, name).into());

    let res = match ty.as_str() {
        "directional" => DirLight::load(scene, params)?.into(),
        "point" => PointLight::load(scene, params)?.into(),
        _ => anyhow::bail!(format!("{}: unknown type '{}'", params.name(), ty)),
    };

    scene.add_light(name, res)?;

    params.check_unused_keys();

    Ok(())
}
