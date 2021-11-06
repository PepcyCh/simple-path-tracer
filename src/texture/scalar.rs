use std::sync::Arc;

use crate::{
    core::{color::Color, intersection::Intersection, scene::Scene, texture::Texture},
    loader::{self, JsonObject, LoadableSceneObject},
};

pub struct ScalarTex<T> {
    value: T,
}

impl<T: Copy> ScalarTex<T> {
    pub fn new(value: T) -> Self {
        Self { value }
    }
}

impl<T: Copy + Send + Sync> Texture<T> for ScalarTex<T> {
    fn value_at(&self, _inter: &Intersection<'_>) -> T {
        self.value
    }
}

impl LoadableSceneObject for ScalarTex<f32> {
    fn load(
        scene: &mut Scene,
        _path: &std::path::PathBuf,
        json_value: &JsonObject,
    ) -> anyhow::Result<()> {
        let name = loader::get_str_field(json_value, "texture-scalar", "name")?;
        let env = format!("texture-scalar({})", name);
        if scene.textures_color.contains_key(name) || scene.textures_f32.contains_key(name) {
            anyhow::bail!(format!("{}: name is duplicated", env));
        }

        let value = loader::get_float_field(json_value, &env, "value")?;

        let tex = ScalarTex::new(value);
        scene.textures_f32.insert(name.to_owned(), Arc::new(tex));

        Ok(())
    }
}

impl LoadableSceneObject for ScalarTex<Color> {
    fn load(
        scene: &mut Scene,
        _path: &std::path::PathBuf,
        json_value: &JsonObject,
    ) -> anyhow::Result<()> {
        let name = loader::get_str_field(json_value, "texture-scalar", "name")?;
        let env = format!("texture-scalar({})", name);
        if scene.textures_color.contains_key(name) || scene.textures_f32.contains_key(name) {
            anyhow::bail!(format!("{}: name is duplicated", env));
        }

        let value = loader::get_float_array3_field(json_value, &env, "value")?;

        let tex = ScalarTex::new(value.into());
        scene.textures_color.insert(name.to_owned(), Arc::new(tex));

        Ok(())
    }
}
