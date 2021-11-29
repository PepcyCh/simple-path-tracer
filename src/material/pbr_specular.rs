use std::sync::Arc;

use crate::{
    core::{color::Color, intersection::Intersection, loader::InputParams, scene::Scene},
    scatter::{
        LambertReflect, MicrofacetReflect, Scatter, SchlickFresnelDielectric, SpecularReflect,
    },
    texture::{Texture, TextureChannel, TextureT},
};

use super::MaterialT;

pub struct PbrSpecular {
    diffuse: Arc<Texture>,
    specular: Arc<Texture>,
    roughness_x: Arc<Texture>,
    roughness_y: Arc<Texture>,
}

impl PbrSpecular {
    pub fn new(
        diffuse: Arc<Texture>,
        specular: Arc<Texture>,
        roughness_x: Arc<Texture>,
        roughness_y: Arc<Texture>,
    ) -> Self {
        Self {
            diffuse,
            specular,
            roughness_x,
            roughness_y,
        }
    }

    pub fn load(scene: &Scene, params: &mut InputParams) -> anyhow::Result<Self> {
        let diffuse = scene.clone_texture(params.get_str("diffuse")?)?;

        let specular = scene.clone_texture(params.get_str("specular")?)?;

        let (roughness_x, roughness_y) = if params.contains_key("roughness") {
            let roughness = scene.clone_texture(params.get_str("roughness")?)?;
            (roughness.clone(), roughness)
        } else {
            let roughness_x = scene.clone_texture(params.get_str("roughness_x")?)?;
            let roughness_y = scene.clone_texture(params.get_str("roughness_y")?)?;
            (roughness_x, roughness_y)
        };

        Ok(PbrSpecular::new(
            diffuse,
            specular,
            roughness_x,
            roughness_y,
        ))
    }
}

impl MaterialT for PbrSpecular {
    fn scatter(&self, inter: &Intersection<'_>) -> Scatter {
        let diffuse = self.diffuse.color_at(inter);
        let specular = self.specular.color_at(inter);
        let roughness_x = self.roughness_x.float_at(inter, TextureChannel::R).powi(2);
        let roughness_y = self.roughness_y.float_at(inter, TextureChannel::R).powi(2);

        if roughness_x < 0.001 || roughness_y < 0.001 {
            SchlickFresnelDielectric::new(
                specular,
                SpecularReflect::new(Color::WHITE),
                LambertReflect::new(diffuse),
            )
            .into()
        } else {
            SchlickFresnelDielectric::new(
                specular,
                MicrofacetReflect::new(Color::WHITE, roughness_x, roughness_y),
                LambertReflect::new(diffuse),
            )
            .into()
        }
    }
}
