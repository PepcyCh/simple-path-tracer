use std::sync::Arc;

use crate::core::{color::Color, loader::InputParams};

use super::{
    Texture, TextureChannel, TextureInput, TextureInputMode, TextureInputWrapMode, TextureT,
};

pub struct TexInputModifier {
    tex: Arc<Texture>,
    tiling: glam::Vec3A,
    offset: glam::Vec3A,
    mode: Option<TextureInputMode>,
    wrap: Option<TextureInputWrapMode>,
}

impl TexInputModifier {
    pub fn new(
        tex: Arc<Texture>,
        tiling: glam::Vec3A,
        offset: glam::Vec3A,
        mode: Option<TextureInputMode>,
        wrap: Option<TextureInputWrapMode>,
    ) -> Self {
        Self {
            tex,
            tiling,
            offset,
            mode,
            wrap,
        }
    }

    fn apply_modifier(&self, input: TextureInput) -> TextureInput {
        TextureInput {
            specified: input.specified * self.tiling + self.offset,
            position: input.position * self.tiling + self.offset,
            normal: input.normal * self.tiling + self.offset,
            tangent: input.tangent * self.tiling + self.offset,
            bitangent: input.bitangent * self.tiling + self.offset,
            texcoords: input.texcoords * self.tiling.truncate() + self.offset.truncate(),
            duvdx: input.duvdx * self.tiling.truncate(),
            duvdy: input.duvdy * self.tiling.truncate(),
            mode: self.mode.unwrap_or(input.mode),
            wrap: self.wrap.unwrap_or(input.wrap),
        }
    }

    pub fn load_with_tex(params: &mut InputParams, tex: Arc<Texture>) -> anyhow::Result<Self> {
        let mode = load_mode(params)?;
        let wrap = load_wrap(params)?;
        let (tiling, offset) =
            if mode.unwrap_or(TextureInputMode::Texcoords) == TextureInputMode::Texcoords {
                let tiling = params.get_float2_or("tiling", [1.0, 1.0]);
                let offset = params.get_float2_or("offset", [0.0, 0.0]);
                (
                    glam::Vec3A::new(tiling[0], tiling[1], 1.0),
                    glam::Vec3A::new(offset[0], offset[1], 0.0),
                )
            } else {
                let tiling = params.get_float3_or("tiling", [1.0, 1.0, 1.0]).into();
                let offset = params.get_float3_or("offset", [0.0, 0.0, 0.0]).into();
                (tiling, offset)
            };
        Ok(Self::new(tex, tiling, offset, mode, wrap))
    }
}

impl TextureT for TexInputModifier {
    fn color_at(&self, input: TextureInput) -> Color {
        self.tex.color_at(self.apply_modifier(input))
    }

    fn float_at(&self, input: TextureInput, chan: TextureChannel) -> f32 {
        self.tex.float_at(self.apply_modifier(input), chan)
    }

    fn average_color(&self) -> Color {
        self.tex.average_color()
    }

    fn average_float(&self, chan: TextureChannel) -> f32 {
        self.tex.average_float(chan)
    }

    fn dimensions(&self) -> Option<(u32, u32, u32)> {
        self.tex.dimensions()
    }

    fn tiling(&self) -> glam::Vec3A {
        self.tex.tiling() * self.tiling
    }

    fn offset(&self) -> glam::Vec3A {
        self.tex.tiling() * self.offset + self.tex.offset()
    }
}

fn load_mode(params: &mut InputParams) -> anyhow::Result<Option<TextureInputMode>> {
    if params.contains_key("mode") {
        let mode_str = params.get_str("mode")?;
        Ok(Some(match mode_str.as_str() {
            "texcoords" => TextureInputMode::Texcoords,
            "position" => TextureInputMode::Position,
            "normal" => TextureInputMode::Normal,
            "tangent" => TextureInputMode::Tangent,
            "bitangent" => TextureInputMode::Bitangent,
            _ => anyhow::bail!(format!(
                "{} - Unknown texture input mode '{}'",
                params.name(),
                mode_str
            )),
        }))
    } else {
        Ok(None)
    }
}

fn load_wrap(params: &mut InputParams) -> anyhow::Result<Option<TextureInputWrapMode>> {
    if params.contains_key("mode") {
        let mode_str = params.get_str("wrap")?;
        Ok(Some(match mode_str.as_str() {
            "repeat" => TextureInputWrapMode::Repeat,
            "mirror_repeat" => TextureInputWrapMode::MirrorRepeat,
            "clamp" => TextureInputWrapMode::Clamp,
            "mirror_clamp" => TextureInputWrapMode::MirrorClamp,
            _ => anyhow::bail!(format!(
                "{} - Unknown texture input wrap mode '{}'",
                params.name(),
                mode_str
            )),
        }))
    } else {
        Ok(None)
    }
}
