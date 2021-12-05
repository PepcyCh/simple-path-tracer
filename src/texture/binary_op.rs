use std::sync::Arc;

use crate::core::{color::Color, loader::InputParams, scene_resources::SceneResources};

use super::{Texture, TextureChannel, TextureInput, TextureT};

pub struct AddTex {
    t1: Arc<Texture>,
    t2: Arc<Texture>,
}

pub struct SubTex {
    t1: Arc<Texture>,
    t2: Arc<Texture>,
}

pub struct MulTex {
    t1: Arc<Texture>,
    t2: Arc<Texture>,
}

pub struct DivTex {
    t1: Arc<Texture>,
    t2: Arc<Texture>,
}

macro_rules! impl_binary_op_tex {
    ( $( ( $name:ident, $op:tt ) ),+ $(,)? ) => {
        $(
            paste::paste! {
                impl [<$name Tex>] {
                    pub fn new(t1: Arc<Texture>, t2: Arc<Texture>) -> Self {
                        Self { t1, t2 }
                    }

                    pub fn load(
                        rsc: &SceneResources,
                        params: &mut InputParams,
                    ) -> anyhow::Result<Self> {
                        let t1 = rsc.clone_texture(params.get_str("t1")?)?;
                        let t2 = rsc.clone_texture(params.get_str("t2")?)?;
                        Ok(Self::new(t1, t2))
                    }
                }

                impl TextureT for [<$name Tex>] {
                    fn color_at(&self, input: TextureInput) -> Color {
                        self.t1.color_at(input) $op self.t2.color_at(input)
                    }

                    fn float_at(&self, input: TextureInput, chan: TextureChannel) -> f32 {
                        self.t1.float_at(input, chan) $op self.t2.float_at(input, chan)
                    }

                    fn average_color(&self) -> Color {
                        self.t1.average_color() $op self.t2.average_color()
                    }

                    fn average_float(&self, chan: TextureChannel) -> f32 {
                        self.t1.average_float(chan) $op self.t2.average_float(chan)
                    }
                }

            }
        )+
    };
}

impl_binary_op_tex! {
    (Add, +),
    (Sub, -),
    (Mul, *),
    (Div, /),
}
