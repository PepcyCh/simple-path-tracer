use std::sync::Arc;

use crate::{
    core::{color::Color, intersection::Intersection, loader::InputParams, scene::Scene},
    scatter::{FresnelDielectricRR, LambertReflect, MicrofacetReflect, Scatter, SpecularReflect},
    texture::{Texture, TextureChannel, TextureT},
};

use super::MaterialT;

pub struct Dielectric {
    ior: f32,
    albedo: Arc<Texture>,
    roughness_x: Arc<Texture>,
    roughness_y: Arc<Texture>,
}

impl Dielectric {
    pub fn new(
        ior: f32,
        albedo: Arc<Texture>,
        roughness_x: Arc<Texture>,
        roughness_y: Arc<Texture>,
    ) -> Self {
        Self {
            ior,
            albedo,
            roughness_x,
            roughness_y,
        }
    }

    pub fn load(scene: &Scene, params: &mut InputParams) -> anyhow::Result<Self> {
        let ior = params.get_float("ior")?;

        let albedo = scene.clone_texture(params.get_str("albedo")?)?;

        let (roughness_x, roughness_y) = if params.contains_key("roughness") {
            let roughness = scene.clone_texture(params.get_str("roughness")?)?;
            (roughness.clone(), roughness)
        } else {
            let roughness_x = scene.clone_texture(params.get_str("roughness_x")?)?;
            let roughness_y = scene.clone_texture(params.get_str("roughness_y")?)?;
            (roughness_x, roughness_y)
        };

        Ok(Dielectric::new(ior, albedo, roughness_x, roughness_y))
    }
}

impl MaterialT for Dielectric {
    fn scatter(&self, inter: &Intersection<'_>) -> Scatter {
        let albedo = self.albedo.color_at(inter);
        let roughness_x = self.roughness_x.float_at(inter, TextureChannel::R).powi(2);
        let roughness_y = self.roughness_y.float_at(inter, TextureChannel::R).powi(2);

        if roughness_x < 0.001 || roughness_y < 0.001 {
            FresnelDielectricRR::new(
                self.ior,
                SpecularReflect::new(Color::WHITE),
                LambertReflect::new(albedo),
            )
            .into()
        } else {
            FresnelDielectricRR::new(
                self.ior,
                MicrofacetReflect::new(Color::WHITE, roughness_x, roughness_y),
                LambertReflect::new(albedo),
            )
            .into()
        }
    }
}
