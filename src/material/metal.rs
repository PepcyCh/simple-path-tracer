use std::sync::Arc;

use crate::{
    core::{
        color::Color, intersection::Intersection, material::Material, scatter::Scatter,
        scene::Scene, texture::Texture,
    },
    loader::{self, JsonObject, LoadableSceneObject},
    scatter::{FresnelConductor, MicrofacetReflect, SpecularReflect},
};

pub struct Metal {
    ior: Arc<dyn Texture<Color>>,
    ior_k: Arc<dyn Texture<Color>>,
    roughness_x: Arc<dyn Texture<f32>>,
    roughness_y: Arc<dyn Texture<f32>>,
}

impl Metal {
    pub fn new(
        ior: Arc<dyn Texture<Color>>,
        ior_k: Arc<dyn Texture<Color>>,
        roughness_x: Arc<dyn Texture<f32>>,
        roughness_y: Arc<dyn Texture<f32>>,
    ) -> Self {
        Self {
            ior,
            ior_k,
            roughness_x,
            roughness_y,
        }
    }
}

impl Material for Metal {
    fn scatter(&self, inter: &Intersection<'_>) -> Box<dyn Scatter> {
        let ior = self.ior.value_at(inter);
        let ior_k = self.ior_k.value_at(inter);
        let roughness_x = self.roughness_x.value_at(inter).powi(2);
        let roughness_y = self.roughness_y.value_at(inter).powi(2);

        if roughness_x < 0.001 || roughness_y < 0.001 {
            Box::new(FresnelConductor::new(
                ior,
                ior_k,
                SpecularReflect::new(Color::WHITE),
            )) as Box<dyn Scatter>
        } else {
            Box::new(FresnelConductor::new(
                ior,
                ior_k,
                MicrofacetReflect::new(Color::WHITE, roughness_x, roughness_y),
            )) as Box<dyn Scatter>
        }
    }
}

impl LoadableSceneObject for Metal {
    fn load(
        scene: &mut Scene,
        _path: &std::path::PathBuf,
        json_value: &JsonObject,
    ) -> anyhow::Result<()> {
        let name = loader::get_str_field(json_value, "material-metal", "name")?;
        let env = format!("material-metal({})", name);
        if scene.materials.contains_key(name) {
            anyhow::bail!(format!("{}: name is duplicated", env));
        }

        let ior = loader::get_str_field(json_value, &env, "ior")?;
        let ior = if let Some(tex) = scene.textures_color.get(ior) {
            tex.clone()
        } else {
            anyhow::bail!(format!("{}: ior '{}' not found", env, ior))
        };

        let ior_k = loader::get_str_field(json_value, &env, "ior_k")?;
        let ior_k = if let Some(tex) = scene.textures_color.get(ior_k) {
            tex.clone()
        } else {
            anyhow::bail!(format!("{}: ior_k '{}' not found", env, ior_k))
        };

        let (roughness_x, roughness_y) = if json_value.contains_key("roughness") {
            let roughness = loader::get_str_field(json_value, &env, "roughness")?;
            let roughness = if let Some(tex) = scene.textures_f32.get(roughness) {
                tex.clone()
            } else {
                anyhow::bail!(format!("{}: roughness '{}' not found", env, roughness))
            };
            (roughness.clone(), roughness)
        } else {
            let roughness_x = loader::get_str_field(json_value, &env, "roughness_x")?;
            let roughness_x = if let Some(tex) = scene.textures_f32.get(roughness_x) {
                tex.clone()
            } else {
                anyhow::bail!(format!("{}: roughness_x '{}' not found", env, roughness_x))
            };
            let roughness_y = loader::get_str_field(json_value, &env, "roughness_y")?;
            let roughness_y = if let Some(tex) = scene.textures_f32.get(roughness_y) {
                tex.clone()
            } else {
                anyhow::bail!(format!("{}: roughness_y '{}' not found", env, roughness_y))
            };
            (roughness_x, roughness_y)
        };

        let mat = Metal::new(ior, ior_k, roughness_x, roughness_y);
        scene.materials.insert(name.to_owned(), Arc::new(mat));

        Ok(())
    }
}
