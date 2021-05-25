use cgmath::{ElementWise, Point2};

use crate::core::color::Color;
use crate::core::intersection::Intersection;
use crate::core::texture::Texture;

pub struct UvMap {
    images: Vec<image::DynamicImage>,
    tiling: Point2<f32>,
    offset: Point2<f32>,
}

impl UvMap {
    pub fn new(image: image::DynamicImage, tiling: Point2<f32>, offset: Point2<f32>) -> Self {
        let images = crate::texture::util::generate_mipmap(image);
        Self {
            images,
            tiling,
            offset,
        }
    }
}

impl Texture<f32> for UvMap {
    fn value_at(&self, inter: &Intersection<'_>) -> f32 {
        let uv = inter
            .texcoords
            .mul_element_wise(self.tiling)
            .add_element_wise(self.offset);
        let (u, v) = crate::texture::util::wrap_uv(uv.x, uv.y);
        crate::texture::util::sample_trilinear(&self.images, u, v, inter.duvdx, inter.duvdy).r
    }
}

impl Texture<Color> for UvMap {
    fn value_at(&self, inter: &Intersection<'_>) -> Color {
        let uv = inter
            .texcoords
            .mul_element_wise(self.tiling)
            .add_element_wise(self.offset);
        let (u, v) = crate::texture::util::wrap_uv(uv.x, uv.y);
        crate::texture::util::sample_trilinear(&self.images, u, v, inter.duvdx, inter.duvdy)
    }
}
