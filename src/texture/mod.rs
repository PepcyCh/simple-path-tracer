mod image_tex;
mod scalar;

pub use image_tex::*;
pub use scalar::*;

use crate::core::{color::Color, intersection::Intersection, loader::InputParams, scene::Scene};

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
}

#[enum_dispatch::enum_dispatch]
pub enum Texture {
    ScalarTex,
    ImageTex,
}

pub fn create_texture_from_params(
    scene: &mut Scene,
    params: &mut InputParams,
) -> anyhow::Result<()> {
    params.set_name("texture".into());
    let ty = params.get_str("type")?;
    let name = params.get_str("name")?;
    params.set_name(format!("texture-{}-{}", ty, name).into());

    let res = match ty.as_str() {
        "scalar" => ScalarTex::load(scene, params)?.into(),
        "image" => ImageTex::load(scene, params)?.into(),
        _ => anyhow::bail!(format!("{}: unknown type '{}'", params.name(), ty)),
    };

    scene.add_texture(name, res)?;

    params.check_unused_keys();

    Ok(())
}
