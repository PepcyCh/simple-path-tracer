mod dielectric;
mod glass;
mod lambert;
mod metal;
mod pseudo;
mod subsurface;

pub use dielectric::*;
pub use glass::*;
pub use lambert::*;
pub use metal::*;
pub use pseudo::*;
pub use subsurface::*;

use crate::{
    core::{intersection::Intersection, loader::InputParams, scene::Scene},
    scatter::Scatter,
};

#[enum_dispatch::enum_dispatch(Material)]
pub trait MaterialT: Send + Sync {
    fn scatter(&self, inter: &Intersection<'_>) -> Scatter;
}

#[enum_dispatch::enum_dispatch]
pub enum Material {
    Dielectric,
    Glass,
    Lambert,
    Metal,
    PseudoMaterial,
    Subsurface,
}

pub fn create_material_from_params(
    scene: &mut Scene,
    params: &mut InputParams,
) -> anyhow::Result<()> {
    params.set_name("material".into());
    let ty = params.get_str("type")?;
    let name = params.get_str("name")?;
    params.set_name(format!("material-{}-{}", ty, name).into());

    let res = match ty.as_str() {
        "dielectric" => Dielectric::load(scene, params)?.into(),
        "glass" => Glass::load(scene, params)?.into(),
        "lambert" => Lambert::load(scene, params)?.into(),
        "metal" => Metal::load(scene, params)?.into(),
        "pseudo" => PseudoMaterial::load(scene, params)?.into(),
        "subsurface" => Subsurface::load(scene, params)?.into(),
        _ => anyhow::bail!(format!("{}: unknown type '{}'", params.name(), ty)),
    };

    scene.add_material(name, res)?;

    params.check_unused_keys();

    Ok(())
}
