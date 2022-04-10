use std::sync::Arc;

use crate::{
    bxdf::{Bxdf, DielectricFresnel, GgxMicrofacet, MicrofacetDielectric, SpecularDielectric},
    core::{intersection::Intersection, loader::InputParams, scene_resources::SceneResources},
    texture::{Texture, TextureChannel, TextureT},
};

use super::MaterialT;

pub struct Dielectric {
    ior: f32,
    reflectance: Arc<Texture>,
    transmittance: Arc<Texture>,
    roughness_x: Arc<Texture>,
    roughness_y: Arc<Texture>,
}

impl Dielectric {
    pub fn new(
        int_ior: f32,
        ext_ior: f32,
        reflectance: Arc<Texture>,
        transmittance: Arc<Texture>,
        roughness_x: Arc<Texture>,
        roughness_y: Arc<Texture>,
    ) -> Self {
        let ior = int_ior / ext_ior;
        Self {
            ior,
            reflectance,
            transmittance,
            roughness_x,
            roughness_y,
        }
    }

    pub fn load(rsc: &SceneResources, params: &mut InputParams) -> anyhow::Result<Self> {
        let int_ior = params.get_float("int_ior")?;
        let ext_ior = params.get_float_or("ext_ior", 1.0);

        let reflectance = rsc.clone_texture(params.get_str("reflectance")?)?;
        let transmittance = rsc.clone_texture(params.get_str("transmittance")?)?;

        let (roughness_x, roughness_y) = if params.contains_key("roughness") {
            let roughness = rsc.clone_texture(params.get_str("roughness")?)?;
            (roughness.clone(), roughness)
        } else {
            let roughness_x = rsc.clone_texture(params.get_str("roughness_x")?)?;
            let roughness_y = rsc.clone_texture(params.get_str("roughness_y")?)?;
            (roughness_x, roughness_y)
        };

        Ok(Self::new(
            int_ior,
            ext_ior,
            reflectance,
            transmittance,
            roughness_x,
            roughness_y,
        ))
    }
}

impl MaterialT for Dielectric {
    fn bxdf_context(&self, inter: &Intersection<'_>) -> Bxdf {
        let reflectance = self.reflectance.color_at(inter.into());
        let transmittance = self.transmittance.color_at(inter.into());

        let roughness_x = self
            .roughness_x
            .float_at(inter.into(), TextureChannel::R)
            .powi(2);
        let roughness_y = self
            .roughness_y
            .float_at(inter.into(), TextureChannel::R)
            .powi(2);

        if roughness_x < 0.0001 || roughness_y < 0.0001 {
            SpecularDielectric::new(DielectricFresnel::new(self.ior).into()).into()
        } else {
            MicrofacetDielectric::new(
                GgxMicrofacet::new(roughness_x, roughness_y).into(),
                DielectricFresnel::new(self.ior).into(),
            )
            .into()
        }
    }
}
