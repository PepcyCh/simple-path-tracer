use std::sync::Arc;

use crate::{
    core::{
        color::Color, intersection::Intersection, material::Material, scatter::Scatter,
        scene::Scene, texture::Texture,
    },
    loader::{self, JsonObject, LoadableSceneObject},
    scatter::{FresnelDielectricRR, LambertReflect, MicrofacetReflect, SpecularReflect},
};

pub struct Dielectric {
    ior: f32,
    albedo: Arc<dyn Texture<Color>>,
    roughness_x: Arc<dyn Texture<f32>>,
    roughness_y: Arc<dyn Texture<f32>>,
}

impl Dielectric {
    pub fn new(
        ior: f32,
        albedo: Arc<dyn Texture<Color>>,
        roughness_x: Arc<dyn Texture<f32>>,
        roughness_y: Arc<dyn Texture<f32>>,
    ) -> Self {
        Self {
            ior,
            albedo,
            roughness_x,
            roughness_y,
        }
    }
}

impl Material for Dielectric {
    fn scatter(&self, inter: &Intersection<'_>) -> Box<dyn Scatter> {
        let albedo = self.albedo.value_at(inter);
        let roughness_x = self.roughness_x.value_at(inter).powi(2);
        let roughness_y = self.roughness_y.value_at(inter).powi(2);

        if roughness_x < 0.001 || roughness_y < 0.001 {
            Box::new(FresnelDielectricRR::new(
                self.ior,
                SpecularReflect::new(Color::WHITE),
                LambertReflect::new(albedo),
            )) as Box<dyn Scatter>
        } else {
            Box::new(FresnelDielectricRR::new(
                self.ior,
                MicrofacetReflect::new(Color::WHITE, roughness_x, roughness_y),
                LambertReflect::new(albedo),
            )) as Box<dyn Scatter>
        }
    }
}

impl LoadableSceneObject for Dielectric {
    fn load(
        scene: &mut Scene,
        _path: &std::path::PathBuf,
        json_value: &JsonObject,
    ) -> anyhow::Result<()> {
        let name = loader::get_str_field(json_value, "material-dielectric", "name")?;
        let env = format!("material-dielectric({})", name);
        if scene.materials.contains_key(name) {
            anyhow::bail!(format!("{}: name is duplicated", env));
        }

        let ior = loader::get_float_field(json_value, &env, "ior")?;

        let albedo = loader::get_str_field(json_value, &env, "albedo")?;
        let albedo = if let Some(tex) = scene.textures_color.get(albedo) {
            tex.clone()
        } else {
            anyhow::bail!(format!("{}: albedo '{}' not found", env, albedo))
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

        let mat = Dielectric::new(ior, albedo, roughness_x, roughness_y);
        scene.materials.insert(name.to_owned(), Arc::new(mat));

        Ok(())
    }
}
