use std::sync::Arc;

use crate::{
    core::{intersection::Intersection, loader::InputParams, scene::Scene},
    scatter::{
        FresnelDielectricRSsr, MicrofacetReflect, Scatter, SpecularReflect, SubsurfaceReflect,
    },
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
        ior: f32,
        albedo: Arc<Texture>,
        ld: Arc<Texture>,
        roughness_x: Arc<Texture>,
        roughness_y: Arc<Texture>,
    ) -> Self {
        Self {
            ior,
            albedo,
            ld,
            roughness_x,
            roughness_y,
        }
    }

    pub fn load(scene: &Scene, params: &mut InputParams) -> anyhow::Result<Self> {
        let ior = params.get_float("ior")?;

        let albedo = scene.clone_texture(params.get_str("albedo")?)?;
        let ld = scene.clone_texture(params.get_str("ld")?)?;

        let (roughness_x, roughness_y) = if params.contains_key("roughness") {
            let roughness = scene.clone_texture(params.get_str("roughness")?)?;
            (roughness.clone(), roughness)
        } else {
            let roughness_x = scene.clone_texture(params.get_str("roughness_x")?)?;
            let roughness_y = scene.clone_texture(params.get_str("roughness_y")?)?;
            (roughness_x, roughness_y)
        };

        Ok(Subsurface::new(ior, albedo, ld, roughness_x, roughness_y))
    }
}

impl MaterialT for Subsurface {
    fn scatter(&self, inter: &Intersection<'_>) -> Scatter {
        let albedo = self.albedo.color_at(inter);
        let ld = self.ld.float_at(inter, TextureChannel::R);
        let roughness_x = self.roughness_x.float_at(inter, TextureChannel::R).powi(2);
        let roughness_y = self.roughness_y.float_at(inter, TextureChannel::R).powi(2);

        if roughness_x < 0.001 || roughness_y < 0.001 {
            FresnelDielectricRSsr::new(
                self.ior,
                SpecularReflect::new(albedo),
                SubsurfaceReflect::new(albedo, ld, self.ior),
            )
            .into()
        } else {
            FresnelDielectricRSsr::new(
                self.ior,
                MicrofacetReflect::new(albedo, roughness_x, roughness_y),
                SubsurfaceReflect::new(albedo, ld, self.ior),
            )
            .into()
        }
    }
}
