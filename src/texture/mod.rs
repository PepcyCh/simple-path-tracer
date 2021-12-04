mod binary_op;
mod image_tex;
mod scalar;
mod srgb_tex;

use std::sync::Arc;

pub use binary_op::*;
pub use image_tex::*;
pub use scalar::*;
pub use srgb_tex::*;

use crate::core::{
    color::Color, intersection::Intersection, loader::InputParams, scene_resources::SceneResources,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TextureChannel {
    R,
    G,
    B,
    A,
}

#[enum_dispatch::enum_dispatch(Texture)]
pub trait TextureT: Send + Sync {
    fn color_at(&self, inter: &Intersection<'_>) -> Color;

    fn float_at(&self, inter: &Intersection<'_>, chan: TextureChannel) -> f32;

    /// returns an estimated value, not need to be accurate
    fn average_color(&self) -> Color;

    /// returns an estimated value, not need to be accurate
    fn average_float(&self, chan: TextureChannel) -> f32;
}

#[enum_dispatch::enum_dispatch]
pub enum Texture {
    ScalarTex,
    ImageTex,
    AddTex,
    SubTex,
    MulTex,
    DivTex,
    SrgbTex,
}

pub fn create_texture_from_params(
    rsc: &mut SceneResources,
    params: &mut InputParams,
) -> anyhow::Result<()> {
    params.set_name("texture".into());
    let ty = params.get_str("type")?;
    let name = params.get_str("name")?;
    params.set_name(format!("texture-{}-{}", ty, name).into());

    let res = match ty.as_str() {
        "scalar" => ScalarTex::load(rsc, params)?.into(),
        "image" => ImageTex::load(rsc, params)?.into(),
        "add" => AddTex::load(rsc, params)?.into(),
        "sub" => SubTex::load(rsc, params)?.into(),
        "mul" => MulTex::load(rsc, params)?.into(),
        "div" => DivTex::load(rsc, params)?.into(),
        _ => anyhow::bail!(format!("{}: unknown type '{}'", params.name(), ty)),
    };

    let is_srgb = params.get_bool_or("is_srgb", false);
    if is_srgb {
        let res = SrgbTex::new(Arc::new(res)).into();
        rsc.add_texture(name, res)?;
    } else {
        rsc.add_texture(name, res)?;
    }

    params.check_unused_keys();

    Ok(())
}
