use std::sync::Arc;

use crate::{
    bxdf::{Bxdf, ConductorFresnel, GgxMicrofacet, MicrofacetConductor, SpecularConductor},
    core::{intersection::Intersection, loader::InputParams, scene_resources::SceneResources},
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
    fn bxdf_context(&self, inter: &Intersection<'_>) -> Bxdf {
        let ior = self.ior.color_at(inter.into());
        let ior_k = self.ior_k.color_at(inter.into());
        let roughness_x = self
            .roughness_x
            .float_at(inter.into(), TextureChannel::R)
            .powi(2);
        let roughness_y = self
            .roughness_y
            .float_at(inter.into(), TextureChannel::R)
            .powi(2);

        if roughness_x < 0.0001 || roughness_y < 0.0001 {
            SpecularConductor::new(ConductorFresnel::new(ior, ior_k).into()).into()
        } else {
            MicrofacetConductor::new(
                GgxMicrofacet::new(roughness_x, roughness_y).into(),
                ConductorFresnel::new(ior, ior_k).into(),
            )
            .into()
        }
    }
}
