use crate::core::{
    color::Color, intersection::Intersection, loader::InputParams, scene_resources::SceneResources,
};

use super::{TextureChannel, TextureT};

pub struct ScalarTex {
    value: Color,
}

impl ScalarTex {
    pub fn new(value: Color) -> Self {
        Self { value }
    }

    pub fn load(_rsc: &SceneResources, params: &mut InputParams) -> anyhow::Result<Self> {
        let value = params.get_float3("value")?.into();

        Ok(Self::new(value))
    }
}

impl TextureT for ScalarTex {
    fn color_at(&self, _inter: &Intersection<'_>) -> Color {
        self.value
    }

    fn float_at(&self, _inter: &Intersection<'_>, chan: TextureChannel) -> f32 {
        match chan {
            TextureChannel::R => self.value.r,
            TextureChannel::G => self.value.g,
            TextureChannel::B => self.value.b,
            TextureChannel::A => 1.0,
        }
    }

    fn average_color(&self) -> Color {
        self.value
    }

    fn average_float(&self, chan: TextureChannel) -> f32 {
        match chan {
            TextureChannel::R => self.value.r,
            TextureChannel::G => self.value.g,
            TextureChannel::B => self.value.b,
            TextureChannel::A => 1.0,
        }
    }
}
