use std::sync::Arc;

use crate::{
    bxdf::{self, Bxdf, DielectricFresnel, GgxMicrofacet, MicrofacetPlastic, SpecularPlastic},
    core::{intersection::Intersection, loader::InputParams, scene_resources::SceneResources},
    texture::{Texture, TextureChannel, TextureT},
};

use super::MaterialT;

pub struct Subsurface {
    ior: f32,
    albedo: Arc<Texture>,
    ld: Arc<Texture>,
    roughness_x: Arc<Texture>,
    roughness_y: Arc<Texture>,
}

impl Subsurface {
    pub fn new(
        int_ior: f32,
        ext_ior: f32,
        albedo: Arc<Texture>,
        ld: Arc<Texture>,
        roughness_x: Arc<Texture>,
        roughness_y: Arc<Texture>,
    ) -> Self {
        let ior = int_ior / ext_ior;
        Self {
            ior,
            albedo,
            ld,
            roughness_x,
            roughness_y,
        }
    }

    pub fn load(rsc: &SceneResources, params: &mut InputParams) -> anyhow::Result<Self> {
        let int_ior = params.get_float("int_ior")?;
        let ext_ior = params.get_float_or("ext_ior", 1.0);

        let albedo = rsc.clone_texture(params.get_str("albedo")?)?;
        let ld = rsc.clone_texture(params.get_str("ld")?)?;

        let (roughness_x, roughness_y) = if params.contains_key("roughness") {
            let roughness = rsc.clone_texture(params.get_str("roughness")?)?;
            (roughness.clone(), roughness)
        } else {
            let roughness_x = rsc.clone_texture(params.get_str("roughness_x")?)?;
            let roughness_y = rsc.clone_texture(params.get_str("roughness_y")?)?;
            (roughness_x, roughness_y)
        };

        Ok(Subsurface::new(
            int_ior,
            ext_ior,
            albedo,
            ld,
            roughness_x,
            roughness_y,
        ))
    }
}

impl MaterialT for Subsurface {
    fn bxdf_context(&self, inter: &Intersection<'_>) -> Bxdf {
        let albedo = self.albedo.color_at(inter.into());
        let ld = self.ld.float_at(inter.into(), TextureChannel::R);

        let roughness_x = self
            .roughness_x
            .float_at(inter.into(), TextureChannel::R)
            .powi(2);
        let roughness_y = self
            .roughness_y
            .float_at(inter.into(), TextureChannel::R)
            .powi(2);

        if roughness_x < 0.0001 || roughness_y < 0.0001 {
            SpecularPlastic::new(
                DielectricFresnel::new(self.ior).into(),
                bxdf::Subsurface::new(albedo, self.ior, ld).into(),
            )
            .into()
        } else {
            MicrofacetPlastic::new(
                GgxMicrofacet::new(roughness_x, roughness_y).into(),
                DielectricFresnel::new(self.ior).into(),
                bxdf::Subsurface::new(albedo, self.ior, ld).into(),
            )
            .into()
        }
    }
}
