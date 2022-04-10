use std::sync::Arc;

use crate::{
    bxdf::{Bxdf, GgxMicrofacet, Lambert, MicrofacetPlastic, SchlickFresnel, SpecularPlastic},
    core::{
        color::Color, intersection::Intersection, loader::InputParams,
        scene_resources::SceneResources,
    },
    texture::{Texture, TextureChannel, TextureT},
};

use super::MaterialT;

pub struct PbrMetallic {
    base_color: Arc<Texture>,
    roughness_x: Arc<Texture>,
    roughness_y: Arc<Texture>,
    roughness_chan: TextureChannel,
    metallic: Arc<Texture>,
    metallic_chan: TextureChannel,
}

impl PbrMetallic {
    pub fn new(
        base_color: Arc<Texture>,
        roughness_x: Arc<Texture>,
        roughness_y: Arc<Texture>,
        roughness_chan: TextureChannel,
        metallic: Arc<Texture>,
        metallic_chan: TextureChannel,
    ) -> Self {
        Self {
            base_color,
            roughness_x,
            roughness_y,
            roughness_chan,
            metallic,
            metallic_chan,
        }
    }

    pub fn load(rsc: &SceneResources, params: &mut InputParams) -> anyhow::Result<Self> {
        let base_color = rsc.clone_texture(params.get_str("base_color")?)?;

        let (roughness_x, roughness_y) = if params.contains_key("roughness") {
            let roughness = rsc.clone_texture(params.get_str("roughness")?)?;
            (roughness.clone(), roughness)
        } else {
            let roughness_x = rsc.clone_texture(params.get_str("roughness_x")?)?;
            let roughness_y = rsc.clone_texture(params.get_str("roughness_y")?)?;
            (roughness_x, roughness_y)
        };

        let metallic = rsc.clone_texture(params.get_str("metallic")?)?;

        Ok(Self::new(
            base_color,
            roughness_x,
            roughness_y,
            TextureChannel::R,
            metallic,
            TextureChannel::R,
        ))
    }
}

impl MaterialT for PbrMetallic {
    fn bxdf_context(&self, inter: &Intersection<'_>) -> Bxdf {
        let base_color = self.base_color.color_at(inter.into());
        let roughness_x = self
            .roughness_x
            .float_at(inter.into(), self.roughness_chan)
            .powi(2);
        let roughness_y = self
            .roughness_y
            .float_at(inter.into(), self.roughness_chan)
            .powi(2);
        let metallic = self.metallic.float_at(inter.into(), self.metallic_chan);

        let specular = metallic * base_color + (1.0 - metallic) * Color::gray(0.04);
        let diffuse = base_color * (1.0 - metallic);

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
