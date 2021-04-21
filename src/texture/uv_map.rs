use crate::core::color::Color;
use crate::core::intersection::Intersection;
use crate::core::texture::Texture;
use image::GenericImageView;

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
        let x = (inter.texcoords.x * self.image.width() as f32) as u32;
        let y = (inter.texcoords.y * self.image.height() as f32) as u32;
        let pixel = self.image.get_pixel(x, y);
        pixel.0[0] as f32 / 255.0
    }
}

impl Texture<Color> for UvMap {
    fn value_at(&self, inter: &Intersection<'_>) -> Color {
        // TODO - interpolate ?
        let x = (inter.texcoords.x * self.image.width() as f32) as u32;
        let y = (inter.texcoords.y * self.image.height() as f32) as u32;
        let pixel = self.image.get_pixel(x, y);
        pixel.into()
    }
}
