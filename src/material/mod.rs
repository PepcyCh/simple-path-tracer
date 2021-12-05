mod conductor;
mod dielectric;
mod glass;
mod lambert;
mod pbr_metallic;
mod pbr_specular;
mod pndf_dielectric;
mod pndf_metal;
mod pseudo;
mod subsurface;

pub use conductor::*;
pub use dielectric::*;
pub use glass::*;
pub use lambert::*;
pub use pbr_metallic::*;
pub use pbr_specular::*;
pub use pndf_dielectric::*;
pub use pndf_metal::*;
pub use pseudo::*;
pub use subsurface::*;

use crate::{
    core::{intersection::Intersection, loader::InputParams, scene_resources::SceneResources},
    scatter::Scatter,
};

#[enum_dispatch::enum_dispatch(Material)]
pub trait MaterialT: Send + Sync {
    fn scatter(&self, inter: &Intersection<'_>) -> Scatter;
}

#[enum_dispatch::enum_dispatch]
pub enum Material {
    Conductor,
    Dielectric,
    Glass,
    Lambert,
    PbrMetallic,
    PbrSpecular,
    PndfDielectric,
    PndfMetal,
    PseudoMaterial,
    Subsurface,
}

pub fn create_material_from_params(
    rsc: &mut SceneResources,
    params: &mut InputParams,
) -> anyhow::Result<()> {
    params.set_name("material".into());
    let ty = params.get_str("type")?;
    let name = params.get_str("name")?;
    params.set_name(format!("material-{}-{}", ty, name).into());

    let res = match ty.as_str() {
        "conductor" => Conductor::load(rsc, params)?.into(),
        "dielectric" => Dielectric::load(rsc, params)?.into(),
        "glass" => Glass::load(rsc, params)?.into(),
        "lambert" => Lambert::load(rsc, params)?.into(),
        "pbr_metallic" => PbrMetallic::load(rsc, params)?.into(),
        "pbr_specular" => PbrSpecular::load(rsc, params)?.into(),
        "pndf_dielectric" => PndfDielectric::load(rsc, params)?.into(),
        "pndf_metal" => PndfMetal::load(rsc, params)?.into(),
        "pseudo" => PseudoMaterial::load(rsc, params)?.into(),
        "subsurface" => Subsurface::load(rsc, params)?.into(),
        _ => anyhow::bail!(format!("{}: unknown type '{}'", params.name(), ty)),
    };

    rsc.add_material(name, res)?;

    params.check_unused_keys();

    Ok(())
}
