use std::sync::Arc;

use crate::{
    core::{
        color::Color, intersection::Intersection, loader::InputParams,
        scene_resources::SceneResources,
    },
    scatter::{FresnelConductor, MicrofacetReflect, Scatter, SpecularReflect},
    texture::{Texture, TextureChannel, TextureT},
};

use super::MaterialT;

pub struct Conductor {
    ior: Arc<Texture>,
    ior_k: Arc<Texture>,
    roughness_x: Arc<Texture>,
    roughness_y: Arc<Texture>,
}

impl Conductor {
    pub fn new(
        ior: Arc<Texture>,
        ior_k: Arc<Texture>,
        roughness_x: Arc<Texture>,
        roughness_y: Arc<Texture>,
    ) -> Self {
        Self {
            ior,
            ior_k,
            roughness_x,
            roughness_y,
        }
    }

    pub fn load(rsc: &SceneResources, params: &mut InputParams) -> anyhow::Result<Self> {
        let ior = rsc.clone_texture(params.get_str("ior")?)?;
        let ior_k = rsc.clone_texture(params.get_str("ior_k")?)?;

        let (roughness_x, roughness_y) = if params.contains_key("roughness") {
            let roughness = rsc.clone_texture(params.get_str("roughness")?)?;
            (roughness.clone(), roughness)
        } else {
            let roughness_x = rsc.clone_texture(params.get_str("roughness_x")?)?;
            let roughness_y = rsc.clone_texture(params.get_str("roughness_y")?)?;
            (roughness_x, roughness_y)
        };

        Ok(Self::new(ior, ior_k, roughness_x, roughness_y))
    }
}

impl MaterialT for Conductor {
    fn scatter(&self, inter: &Intersection<'_>) -> Scatter {
        let ior = self.ior.color_at(inter);
        let ior_k = self.ior_k.color_at(inter);
        let roughness_x = self.roughness_x.float_at(inter, TextureChannel::R).powi(2);
        let roughness_y = self.roughness_y.float_at(inter, TextureChannel::R).powi(2);

        if roughness_x < 0.001 || roughness_y < 0.001 {
            FresnelConductor::new(ior, ior_k, SpecularReflect::new(Color::WHITE)).into()
        } else {
            FresnelConductor::new(
                ior,
                ior_k,
                MicrofacetReflect::new(Color::WHITE, roughness_x, roughness_y),
            )
            .into()
        }
    }
}
