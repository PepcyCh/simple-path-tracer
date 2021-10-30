use std::sync::Arc;

use crate::{
    core::{
        color::Color, intersection::Intersection, material::Material, scatter::Scatter,
        scene::Scene, texture::Texture,
    },
    loader::{self, JsonObject, Loadable},
    scatter::{FresnelDielectricRR, LambertReflect, MicrofacetReflect, SpecularReflect},
};

pub struct Dielectric {
    ior: f32,
    albedo: Arc<dyn Texture<Color>>,
    roughness: Arc<dyn Texture<f32>>,
}

impl Dielectric {
    pub fn new(
        ior: f32,
        albedo: Arc<dyn Texture<Color>>,
        roughness: Arc<dyn Texture<f32>>,
    ) -> Self {
        Self {
            ior,
            albedo,
            roughness,
        }
    }
}

impl Material for Dielectric {
    fn scatter(&self, inter: &Intersection<'_>) -> Box<dyn Scatter> {
        let albedo = self.albedo.value_at(inter);
        let roughness = self.roughness.value_at(inter).powi(2);

        if roughness < 0.001 {
            Box::new(FresnelDielectricRR::new(
                self.ior,
                SpecularReflect::new(Color::WHITE),
                LambertReflect::new(albedo),
            )) as Box<dyn Scatter>
        } else {
            Box::new(FresnelDielectricRR::new(
                self.ior,
                MicrofacetReflect::new(Color::WHITE, roughness),
                LambertReflect::new(albedo),
            )) as Box<dyn Scatter>
        }
    }
}

impl Loadable for Dielectric {
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

        let roughness = loader::get_str_field(json_value, &env, "roughness")?;
        let roughness = if let Some(tex) = scene.textures_f32.get(roughness) {
            tex.clone()
        } else {
            anyhow::bail!(format!("{}: roughness '{}' not found", env, roughness))
        };

        let mat = Dielectric::new(ior, albedo, roughness);
        scene.materials.insert(name.to_owned(), Arc::new(mat));

        Ok(())
    }
}
