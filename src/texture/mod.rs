mod binary_op;
mod image_tex;
mod input_modifier;
mod scalar;
mod srgb_tex;

use std::sync::Arc;

use glam::Vec3Swizzles;

pub use binary_op::*;
pub use image_tex::*;
pub use input_modifier::*;
pub use scalar::*;
pub use srgb_tex::*;

use crate::core::{
    color::Color, intersection::Intersection, loader::InputParams, scene_resources::SceneResources,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TextureInputMode {
    Specified,
    Texcoords,
    Position,
    Normal,
    Tangent,
    Bitangent,
}

impl Default for TextureInputMode {
    fn default() -> Self {
        Self::Specified
    }
}

#[derive(Clone, Copy)]
pub enum TextureInputWrapMode {
    Repeat,
    MirrorRepeat,
    Clamp,
    MirrorClamp,
}

impl Default for TextureInputWrapMode {
    fn default() -> Self {
        Self::Repeat
    }
}

#[derive(Clone, Copy, Default)]
pub struct TextureInput {
    pub specified: glam::Vec3A,
    pub position: glam::Vec3A,
    pub normal: glam::Vec3A,
    pub tangent: glam::Vec3A,
    pub bitangent: glam::Vec3A,
    pub texcoords: glam::Vec2,
    pub duvdx: glam::Vec2,
    pub duvdy: glam::Vec2,
    pub mode: TextureInputMode,
    pub wrap: TextureInputWrapMode,
}

impl TextureInput {
    pub fn specified(value: glam::Vec3A) -> Self {
        Self {
            specified: value,
            mode: TextureInputMode::Specified,
            ..Default::default()
        }
    }

    pub fn value_vec2(&self) -> glam::Vec2 {
        match self.mode {
            TextureInputMode::Specified => self.specified.xy(),
            TextureInputMode::Texcoords => self.texcoords,
            TextureInputMode::Position => self.position.xy(),
            TextureInputMode::Normal => self.normal.xy(),
            TextureInputMode::Tangent => self.tangent.xy(),
            TextureInputMode::Bitangent => self.bitangent.xy(),
        }
    }

    pub fn value_vec2_wrapped(&self) -> glam::Vec2 {
        let value = self.value_vec2();
        match self.wrap {
            TextureInputWrapMode::Repeat => {
                let x_fract = value.x.fract();
                let x_new = if value.x >= 0.0 {
                    x_fract
                } else {
                    1.0 + x_fract
                };
                let y_fract = value.y.fract();
                let y_new = if value.y >= 0.0 {
                    y_fract
                } else {
                    1.0 + y_fract
                };
                glam::Vec2::new(x_new, y_new)
            }
            TextureInputWrapMode::MirrorRepeat => {
                let x_fract = value.x.fract();
                let x_new = if value.x >= 0.0 {
                    x_fract
                } else {
                    1.0 + x_fract
                };
                let x_new = if value.x as i32 % 2 == 0 {
                    x_new
                } else {
                    1.0 - x_new
                };
                let y_fract = value.y.fract();
                let y_new = if value.y >= 0.0 {
                    y_fract
                } else {
                    1.0 + y_fract
                };
                let y_new = if value.y as i32 % 2 == 0 {
                    y_new
                } else {
                    1.0 - y_new
                };
                glam::Vec2::new(x_new, y_new)
            }
            TextureInputWrapMode::Clamp => {
                glam::Vec2::new(value.x.clamp(0.0, 1.0), value.y.clamp(0.0, 1.0))
            }
            TextureInputWrapMode::MirrorClamp => {
                glam::Vec2::new(value.x.clamp(0.0, 1.0).abs(), value.y.clamp(0.0, 1.0).abs())
            }
        }
    }

    pub fn value_vec3(&self) -> glam::Vec3A {
        match self.mode {
            TextureInputMode::Specified => self.specified,
            TextureInputMode::Texcoords => self.texcoords.extend(0.0).into(),
            TextureInputMode::Position => self.position,
            TextureInputMode::Normal => self.normal,
            TextureInputMode::Tangent => self.tangent,
            TextureInputMode::Bitangent => self.bitangent,
        }
    }
}

impl From<&Intersection<'_>> for TextureInput {
    fn from(inter: &Intersection<'_>) -> Self {
        Self {
            texcoords: inter.texcoords,
            duvdx: inter.duvdx,
            duvdy: inter.duvdy,
            mode: TextureInputMode::Texcoords,
            position: inter.position,
            normal: inter.normal,
            tangent: inter.tangent,
            bitangent: inter.bitangent,
            ..Default::default()
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TextureChannel {
    R,
    G,
    B,
    A,
}

#[enum_dispatch::enum_dispatch(Texture)]
pub trait TextureT: Send + Sync {
    fn color_at(&self, input: TextureInput) -> Color;

    fn float_at(&self, input: TextureInput, chan: TextureChannel) -> f32;

    /// returns an estimated value, not need to be accurate
    fn average_color(&self) -> Color;

    /// returns an estimated value, not need to be accurate
    fn average_float(&self, chan: TextureChannel) -> f32;

    fn dimensions(&self) -> Option<(u32, u32, u32)> {
        None
    }

    fn tiling(&self) -> glam::Vec3A {
        glam::Vec3A::ONE
    }

    fn offset(&self) -> glam::Vec3A {
        glam::Vec3A::ZERO
    }
}

#[enum_dispatch::enum_dispatch]
pub enum Texture {
    ScalarTex,
    ImageTex,
    TexInputModifier,
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

    let mut res = match ty.as_str() {
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
        res = SrgbTex::new(Arc::new(res)).into();
    }

    if params.num_unused_keys() > 0 {
        res = TexInputModifier::load_with_tex(params, Arc::new(res))?.into();
    }

    rsc.add_texture(name, res)?;

    params.check_unused_keys();

    Ok(())
}
