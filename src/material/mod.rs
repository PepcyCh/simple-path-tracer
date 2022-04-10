mod conductor;
mod dielectric;
mod lambert;
mod pbr_metallic;
mod pbr_specular;
mod plastic;
mod pndf_conductor;
mod pndf_plastic;
mod pseudo;
mod subsurface;

pub use conductor::*;
pub use dielectric::*;
pub use lambert::*;
pub use pbr_metallic::*;
pub use pbr_specular::*;
pub use plastic::*;
pub use pndf_conductor::*;
pub use pndf_plastic::*;
pub use pseudo::*;
pub use subsurface::*;

use crate::{
    bxdf::Bxdf,
    core::{intersection::Intersection, loader::InputParams, scene_resources::SceneResources},
};

#[enum_dispatch::enum_dispatch(Material)]
pub trait MaterialT: Send + Sync {
    fn bxdf_context(&self, inter: &Intersection<'_>) -> Bxdf;
}

#[enum_dispatch::enum_dispatch]
pub enum Material {
    Conductor,
    Dielectric,
    Plastic,
    Lambert,
    PbrMetallic,
    PbrSpecular,
    PndfConductor,
    PndfPlastic,
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
        "plastic" => Plastic::load(rsc, params)?.into(),
        "lambert" => Lambert::load(rsc, params)?.into(),
        "pbr_metallic" => PbrMetallic::load(rsc, params)?.into(),
        "pbr_specular" => PbrSpecular::load(rsc, params)?.into(),
        "pndf_conductor" => PndfConductor::load(rsc, params)?.into(),
        "pndf_plastic" => PndfPlastic::load(rsc, params)?.into(),
        "pseudo" => PseudoMaterial::load(rsc, params)?.into(),
        "subsurface" => Subsurface::load(rsc, params)?.into(),
        _ => anyhow::bail!(format!("{}: unknown type '{}'", params.name(), ty)),
    };

    rsc.add_material(name, res)?;

    params.check_unused_keys();

    Ok(())
}
