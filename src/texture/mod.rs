mod binary_op;
mod image_tex;
mod scalar;

pub use binary_op::*;
pub use image_tex::*;
pub use scalar::*;

use crate::core::{
    color::Color, intersection::Intersection, loader::InputParams, scene_resources::SceneResources,
};

#[derive(Clone, Copy)]
pub enum TextureChannel {
    #[allow(dead_code)]
    R,
    #[allow(dead_code)]
    G,
    #[allow(dead_code)]
    B,
    #[allow(dead_code)]
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

    rsc.add_texture(name, res)?;

    params.check_unused_keys();

    Ok(())
}
