use std::sync::Arc;

use crate::{
    core::{
        color::Color, intersection::Intersection, material::Material, scatter::Scatter,
        scene::Scene, texture::Texture,
    },
    loader::{self, JsonObject, LoadableSceneObject},
    scatter::{
        FresnelDielectricRT, MicrofacetReflect, MicrofacetTransmit, SpecularReflect,
        SpecularTransmit,
    },
};

pub struct Glass {
    ior: f32,
    reflectance: Arc<dyn Texture<Color>>,
    transmittance: Arc<dyn Texture<Color>>,
    roughness_x: Arc<dyn Texture<f32>>,
    roughness_y: Arc<dyn Texture<f32>>,
}

impl Glass {
    pub fn new(
        ior: f32,
        reflectance: Arc<dyn Texture<Color>>,
        transmittance: Arc<dyn Texture<Color>>,
        roughness_x: Arc<dyn Texture<f32>>,
        roughness_y: Arc<dyn Texture<f32>>,
    ) -> Self {
        Self {
            ior,
            reflectance,
            transmittance,
            roughness_x,
            roughness_y,
        }
    }
}

impl Material for Glass {
    fn scatter(&self, inter: &Intersection<'_>) -> Box<dyn Scatter> {
        let reflectance = self.reflectance.value_at(inter);
        let transmittance = self.transmittance.value_at(inter);
        let roughness_x = self.roughness_x.value_at(inter);
        let roughness_y = self.roughness_y.value_at(inter);

        if roughness_x < 0.001 || roughness_y < 0.001 {
            Box::new(FresnelDielectricRT::new(
                self.ior,
                SpecularReflect::new(reflectance),
                SpecularTransmit::new(transmittance, self.ior),
            )) as Box<dyn Scatter>
        } else {
            Box::new(FresnelDielectricRT::new(
                self.ior,
                MicrofacetReflect::new(reflectance, roughness_x, roughness_y),
                MicrofacetTransmit::new(transmittance, self.ior, roughness_x, roughness_y),
            )) as Box<dyn Scatter>
        }
    }
}

impl LoadableSceneObject for Glass {
    fn load(
        scene: &mut Scene,
        _path: &std::path::PathBuf,
        json_value: &JsonObject,
    ) -> anyhow::Result<()> {
        let name = loader::get_str_field(json_value, "material-glass", "name")?;
        let env = format!("material-glass({})", name);
        if scene.materials.contains_key(name) {
            anyhow::bail!(format!("{}: name is duplicated", env));
        }

        let ior = loader::get_float_field(json_value, &env, "ior")?;

        let reflectance = loader::get_str_field(json_value, &env, "reflectance")?;
        let reflectance = if let Some(tex) = scene.textures_color.get(reflectance) {
            tex.clone()
        } else {
            anyhow::bail!(format!("{}: reflectance '{}' not found", env, reflectance))
        };

        let transmittance = loader::get_str_field(json_value, &env, "transmittance")?;
        let transmittance = if let Some(tex) = scene.textures_color.get(transmittance) {
            tex.clone()
        } else {
            anyhow::bail!(format!(
                "{}: transmittance '{}' not found",
                env, transmittance
            ))
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

        let mat = Glass::new(ior, reflectance, transmittance, roughness_x, roughness_y);
        scene.materials.insert(name.to_owned(), Arc::new(mat));

        Ok(())
    }
}
