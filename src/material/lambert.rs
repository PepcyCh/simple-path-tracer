use std::sync::Arc;

use crate::{
    core::{
        color::Color, intersection::Intersection, material::Material, scatter::Scatter,
        scene::Scene, texture::Texture,
    },
    loader::{self, JsonObject, Loadable},
    scatter::LambertReflect,
};

pub struct Lambert {
    albedo: Arc<dyn Texture<Color>>,
}

impl Lambert {
    pub fn new(albedo: Arc<dyn Texture<Color>>) -> Self {
        Self { albedo }
    }
}

impl Material for Lambert {
    fn scatter(&self, inter: &Intersection<'_>) -> Box<dyn Scatter> {
        let albedo = self.albedo.value_at(inter);
        Box::new(LambertReflect::new(albedo)) as Box<dyn Scatter>
    }
}

impl Loadable for Lambert {
    fn load(
        scene: &mut Scene,
        _path: &std::path::PathBuf,
        json_value: &JsonObject,
    ) -> anyhow::Result<()> {
        let name = loader::get_str_field(json_value, "material-lambert", "name")?;
        let env = format!("material-lambert({})", name);
        if scene.materials.contains_key(name) {
            anyhow::bail!(format!("{}: name is duplicated", env));
        }

        let albedo = loader::get_str_field(json_value, &env, "albedo")?;
        let albedo = if let Some(tex) = scene.textures_color.get(albedo) {
            tex.clone()
        } else {
            anyhow::bail!(format!("{}: albedo '{}' not found", env, albedo))
        };

        let mat = Lambert::new(albedo);
        scene.materials.insert(name.to_owned(), Arc::new(mat));

        Ok(())
    }
}
