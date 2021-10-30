use std::sync::Arc;

use crate::{
    core::{color::Color, intersection::Intersection, scene::Scene, texture::Texture},
    loader::{self, JsonObject, Loadable},
};

pub struct ImageTex {
    images: Vec<image::DynamicImage>,
    tiling: glam::Vec2,
    offset: glam::Vec2,
}

impl ImageTex {
    pub fn new(image: image::DynamicImage, tiling: glam::Vec2, offset: glam::Vec2) -> Self {
        let images = crate::texture::util::generate_mipmap(image);
        Self {
            images,
            tiling,
            offset,
        }
    }
}

impl Texture<f32> for ImageTex {
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

impl Texture<Color> for ImageTex {
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

impl Loadable for ImageTex {
    fn load(
        scene: &mut Scene,
        path: &std::path::PathBuf,
        json_value: &JsonObject,
    ) -> anyhow::Result<()> {
        let name = loader::get_str_field(json_value, "texture-image", "name")?;
        let env = format!("texture-image({})", name);
        if scene.textures_color.contains_key(name) || scene.textures_f32.contains_key(name) {
            anyhow::bail!(format!("{}: name is duplicated", env));
        }

        let ele_ty = loader::get_str_field(json_value, &env, "ele")?;

        let value = loader::get_image_field(json_value, &env, "image_file", path)?;
        let tiling = loader::get_float_array2_field_or(json_value, &env, "tiling", [1.0, 1.0])?;
        let offset = loader::get_float_array2_field_or(json_value, &env, "offset", [0.0, 0.0])?;

        match ele_ty {
            "color" => {
                let tex = ImageTex::new(value, tiling.into(), offset.into());
                scene.textures_color.insert(name.to_owned(), Arc::new(tex));
            }
            "float" => {
                let tex = ImageTex::new(value, tiling.into(), offset.into());
                scene.textures_f32.insert(name.to_owned(), Arc::new(tex));
            }
            _ => anyhow::bail!(format!("{}: unknown element type '{}'", env, ele_ty)),
        }

        Ok(())
    }
}
