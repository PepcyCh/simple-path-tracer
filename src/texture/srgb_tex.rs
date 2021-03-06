use std::sync::Arc;

use crate::core::color::Color;

use super::{Texture, TextureChannel, TextureInput, TextureT};

pub struct SrgbTex {
    tex: Arc<Texture>,
}

impl SrgbTex {
    pub fn new(tex: Arc<Texture>) -> Self {
        Self { tex }
    }
}

impl TextureT for SrgbTex {
    fn color_at(&self, input: TextureInput) -> Color {
        srgb_to_linear_color(self.tex.color_at(input))
    }

    fn float_at(&self, input: TextureInput, chan: TextureChannel) -> f32 {
        if chan == TextureChannel::A {
            self.tex.float_at(input, chan)
        } else {
            srgb_to_linear(self.tex.float_at(input, chan))
        }
    }

    fn average_color(&self) -> Color {
        srgb_to_linear_color(self.tex.average_color())
    }

    fn average_float(&self, chan: TextureChannel) -> f32 {
        if chan == TextureChannel::A {
            self.tex.average_float(chan)
        } else {
            srgb_to_linear(self.tex.average_float(chan))
        }
    }

    fn dimensions(&self) -> Option<(u32, u32, u32)> {
        self.tex.dimensions()
    }

    fn tiling(&self) -> glam::Vec3A {
        self.tex.tiling()
    }

    fn offset(&self) -> glam::Vec3A {
        self.tex.offset()
    }
}

fn srgb_to_linear(s: f32) -> f32 {
    if s <= 0.04045 {
        s / 12.92
    } else {
        ((s + 0.055) / 1.055).powf(2.4)
    }
}

fn srgb_to_linear_color(s: Color) -> Color {
    Color::new(
        srgb_to_linear(s.r),
        srgb_to_linear(s.g),
        srgb_to_linear(s.b),
    )
}
