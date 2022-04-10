use std::sync::Arc;

use crate::{
    bxdf::{Bxdf, GgxMicrofacet, Lambert, MicrofacetPlastic, SchlickFresnel, SpecularPlastic},
    core::{intersection::Intersection, loader::InputParams, scene_resources::SceneResources},
    texture::{Texture, TextureChannel, TextureT},
};

use super::MaterialT;

pub struct PbrSpecular {
    diffuse: Arc<Texture>,
    specular: Arc<Texture>,
    roughness_x: Arc<Texture>,
    roughness_y: Arc<Texture>,
    roughness_chan: TextureChannel,
}

impl PbrSpecular {
    pub fn new(
        diffuse: Arc<Texture>,
        specular: Arc<Texture>,
        roughness_x: Arc<Texture>,
        roughness_y: Arc<Texture>,
        roughness_chan: TextureChannel,
    ) -> Self {
        Self {
            diffuse,
            specular,
            roughness_x,
            roughness_y,
            roughness_chan,
        }
    }

    pub fn load(rsc: &SceneResources, params: &mut InputParams) -> anyhow::Result<Self> {
        let diffuse = rsc.clone_texture(params.get_str("diffuse")?)?;

        let specular = rsc.clone_texture(params.get_str("specular")?)?;

        let (roughness_x, roughness_y) = if params.contains_key("roughness") {
            let roughness = rsc.clone_texture(params.get_str("roughness")?)?;
            (roughness.clone(), roughness)
        } else {
            let roughness_x = rsc.clone_texture(params.get_str("roughness_x")?)?;
            let roughness_y = rsc.clone_texture(params.get_str("roughness_y")?)?;
            (roughness_x, roughness_y)
        };

        Ok(Self::new(
            diffuse,
            specular,
            roughness_x,
            roughness_y,
            TextureChannel::R,
        ))
    }
}

impl MaterialT for PbrSpecular {
    fn bxdf_context(&self, inter: &Intersection<'_>) -> Bxdf {
        let diffuse = self.diffuse.color_at(inter.into());
        let specular = self.specular.color_at(inter.into());
        let roughness_x = self
            .roughness_x
            .float_at(inter.into(), self.roughness_chan)
            .powi(2);
        let roughness_y = self
            .roughness_y
            .float_at(inter.into(), self.roughness_chan)
            .powi(2);

        if roughness_x < 0.0001 || roughness_y < 0.0001 {
            SpecularPlastic::new(
                SchlickFresnel::new(specular).into(),
                Lambert::new(diffuse).into(),
            )
            .into()
        } else {
            MicrofacetPlastic::new(
                GgxMicrofacet::new(roughness_x, roughness_y).into(),
                SchlickFresnel::new(specular).into(),
                Lambert::new(diffuse).into(),
            )
            .into()
        }
    }
}
