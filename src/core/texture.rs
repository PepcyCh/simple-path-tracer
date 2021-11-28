use crate::core::{color::Color, intersection::Intersection};

#[derive(Clone, Copy)]
pub enum TextureChannel {
    R,
    G,
    B,
    A,
}

pub trait Texture: Send + Sync {
    fn color_at(&self, inter: &Intersection<'_>) -> Color;

    fn float_at(&self, inter: &Intersection<'_>, chan: TextureChannel) -> f32;
}
