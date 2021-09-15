use crate::core::{color::Color, intersection::Intersection, texture::Texture};

pub struct UvMap {
    images: Vec<image::DynamicImage>,
    tiling: glam::Vec2,
    offset: glam::Vec2,
}

impl UvMap {
    pub fn new(image: image::DynamicImage, tiling: glam::Vec2, offset: glam::Vec2) -> Self {
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
        let uv = inter.texcoords * self.tiling + self.offset;
        let (u, v) = crate::texture::util::wrap_uv(uv.x, uv.y);
        crate::texture::util::sample_trilinear(
            &self.images,
            u,
            v,
            vec2_mul_point2(inter.duvdx, self.tiling),
            vec2_mul_point2(inter.duvdy, self.tiling),
        )
        .r
    }
}

impl Texture<Color> for UvMap {
    fn value_at(&self, inter: &Intersection<'_>) -> Color {
        let uv = inter.texcoords * self.tiling + self.offset;
        let (u, v) = crate::texture::util::wrap_uv(uv.x, uv.y);
        crate::texture::util::sample_trilinear(
            &self.images,
            u,
            v,
            vec2_mul_point2(inter.duvdx, self.tiling),
            vec2_mul_point2(inter.duvdy, self.tiling),
        )
    }
}

fn vec2_mul_point2(a: glam::Vec2, b: glam::Vec2) -> glam::Vec2 {
    glam::Vec2::new(a.x * b.x, a.y * b.y)
}
