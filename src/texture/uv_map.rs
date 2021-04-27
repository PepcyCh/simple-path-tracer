use crate::core::color::Color;
use crate::core::intersection::Intersection;
use crate::core::texture::Texture;

pub struct UvMap {
    images: Vec<image::DynamicImage>,
}

impl UvMap {
    pub fn new(image: image::DynamicImage) -> Self {
        let images = crate::texture::util::generate_mipmap(image);
        Self { images }
    }
}

impl Texture<f32> for UvMap {
    fn value_at(&self, inter: &Intersection<'_>) -> f32 {
        crate::texture::util::sample_trilinear(
            &self.images,
            inter.texcoords.x,
            inter.texcoords.y,
            inter.duvdx,
            inter.duvdy,
        ).r
    }
}

impl Texture<Color> for UvMap {
    fn value_at(&self, inter: &Intersection<'_>) -> Color {
        crate::texture::util::sample_trilinear(
            &self.images,
            inter.texcoords.x,
            inter.texcoords.y,
            inter.duvdx,
            inter.duvdy,
        )
    }
}
