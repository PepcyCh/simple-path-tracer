use crate::core::color::Color;
use crate::core::intersection::Intersection;
use crate::core::texture::Texture;

pub struct UvMap {
    image: image::DynamicImage,
}

impl UvMap {
    pub fn new(image: image::DynamicImage) -> Self {
        Self { image }
    }
}

impl Texture<f32> for UvMap {
    fn value_at(&self, inter: &Intersection<'_>) -> f32 {
        crate::texture::util::get_pixel(&self.image, inter.texcoords.x, inter.texcoords.y).r
    }
}

impl Texture<Color> for UvMap {
    fn value_at(&self, inter: &Intersection<'_>) -> Color {
        crate::texture::util::get_pixel(&self.image, inter.texcoords.x, inter.texcoords.y)
    }
}
