use std::sync::Arc;

use crate::{
    core::{color::Color, intersection::Intersection, loader::InputParams, scene::Scene},
    scatter::{
        LambertReflect, MicrofacetReflect, MixScatter, Scatter, SchlickFresnelDielectric,
        SchlickFresnelMetal, SpecularReflect,
    },
    texture::{Texture, TextureChannel, TextureT},
};

use super::MaterialT;

pub struct PbrMetallic {
    base_color: Arc<Texture>,
    roughness_x: Arc<Texture>,
    roughness_y: Arc<Texture>,
    metallic: Arc<Texture>,
}

impl PbrMetallic {
    pub fn new(
        base_color: Arc<Texture>,
        roughness_x: Arc<Texture>,
        roughness_y: Arc<Texture>,
        metallic: Arc<Texture>,
    ) -> Self {
        Self {
            base_color,
            roughness_x,
            roughness_y,
            metallic,
        }
    }

    pub fn load(scene: &Scene, params: &mut InputParams) -> anyhow::Result<Self> {
        let base_color = scene.clone_texture(params.get_str("base_color")?)?;

        let (roughness_x, roughness_y) = if params.contains_key("roughness") {
            let roughness = scene.clone_texture(params.get_str("roughness")?)?;
            (roughness.clone(), roughness)
        } else {
            let roughness_x = scene.clone_texture(params.get_str("roughness_x")?)?;
            let roughness_y = scene.clone_texture(params.get_str("roughness_y")?)?;
            (roughness_x, roughness_y)
        };

        let metallic = scene.clone_texture(params.get_str("metallic")?)?;

        Ok(PbrMetallic::new(
            base_color,
            roughness_x,
            roughness_y,
            metallic,
        ))
    }
}

impl MaterialT for PbrMetallic {
    fn scatter(&self, inter: &Intersection<'_>) -> Scatter {
        let base_color = self.base_color.color_at(inter);
        let roughness_x = self.roughness_x.float_at(inter, TextureChannel::R).powi(2);
        let roughness_y = self.roughness_y.float_at(inter, TextureChannel::R).powi(2);
        let metallic = self.metallic.float_at(inter, TextureChannel::R);

        if roughness_x < 0.001 || roughness_y < 0.001 {
            MixScatter::new(
                metallic,
                SchlickFresnelMetal::new(base_color, SpecularReflect::new(Color::WHITE)),
                SchlickFresnelDielectric::new(
                    Color::gray(0.04),
                    SpecularReflect::new(Color::WHITE),
                    LambertReflect::new(base_color),
                ),
            )
            .into()
        } else {
            MixScatter::new(
                metallic,
                SchlickFresnelMetal::new(
                    base_color,
                    MicrofacetReflect::new(Color::WHITE, roughness_x, roughness_y),
                ),
                SchlickFresnelDielectric::new(
                    Color::gray(0.04),
                    MicrofacetReflect::new(Color::WHITE, roughness_x, roughness_y),
                    LambertReflect::new(base_color),
                ),
            )
            .into()
        }
    }
}
