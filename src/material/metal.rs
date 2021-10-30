use std::sync::Arc;

use crate::{
    core::{
        color::Color, intersection::Intersection, material::Material, scatter::Scatter,
        scene::Scene, texture::Texture,
    },
    loader::{self, JsonObject, Loadable},
    scatter::{FresnelConductor, MicrofacetReflect, SpecularReflect},
};

pub struct Metal {
    ior: Arc<dyn Texture<Color>>,
    ior_k: Arc<dyn Texture<Color>>,
    roughness: Arc<dyn Texture<f32>>,
}

impl Metal {
    pub fn new(
        ior: Arc<dyn Texture<Color>>,
        ior_k: Arc<dyn Texture<Color>>,
        roughness: Arc<dyn Texture<f32>>,
    ) -> Self {
        Self {
            ior,
            ior_k,
            roughness,
        }
    }
}

impl Material for Metal {
    fn scatter(&self, inter: &Intersection<'_>) -> Box<dyn Scatter> {
        let ior = self.ior.value_at(inter);
        let ior_k = self.ior_k.value_at(inter);
        let roughness = self.roughness.value_at(inter).powi(2);

        if roughness < 0.001 {
            Box::new(FresnelConductor::new(
                ior,
                ior_k,
                SpecularReflect::new(Color::WHITE),
            )) as Box<dyn Scatter>
        } else {
            Box::new(FresnelConductor::new(
                ior,
                ior_k,
                MicrofacetReflect::new(Color::WHITE, roughness),
            )) as Box<dyn Scatter>
        }
    }
}

impl Loadable for Metal {
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

        let roughness = loader::get_str_field(json_value, &env, "roughness")?;
        let roughness = if let Some(tex) = scene.textures_f32.get(roughness) {
            tex.clone()
        } else {
            anyhow::bail!(format!("{}: roughness '{}' not found", env, roughness))
        };

        let mat = Metal::new(ior, ior_k, roughness);
        scene.materials.insert(name.to_owned(), Arc::new(mat));

        Ok(())
    }
}
