use std::sync::Arc;

use crate::{
    core::{intersection::Intersection, loader::InputParams, scene::Scene},
    scatter::{
        FresnelDielectricRT, MicrofacetReflect, MicrofacetTransmit, Scatter, SpecularReflect,
        SpecularTransmit,
    },
    texture::{Texture, TextureChannel, TextureT},
};

use super::MaterialT;

pub struct Glass {
    ior: f32,
    reflectance: Arc<Texture>,
    transmittance: Arc<Texture>,
    roughness_x: Arc<Texture>,
    roughness_y: Arc<Texture>,
}

impl Glass {
    pub fn new(
        ior: f32,
        reflectance: Arc<Texture>,
        transmittance: Arc<Texture>,
        roughness_x: Arc<Texture>,
        roughness_y: Arc<Texture>,
    ) -> Self {
        Self {
            ior,
            reflectance,
            transmittance,
            roughness_x,
            roughness_y,
        }
    }

    pub fn load(scene: &Scene, params: &mut InputParams) -> anyhow::Result<Self> {
        let ior = params.get_float("ior")?;

        let reflectance = scene.clone_texture(params.get_str("reflectance")?)?;
        let transmittance = scene.clone_texture(params.get_str("transmittance")?)?;

        let (roughness_x, roughness_y) = if params.contains_key("roughness") {
            let roughness = scene.clone_texture(params.get_str("roughness")?)?;
            (roughness.clone(), roughness)
        } else {
            let roughness_x = scene.clone_texture(params.get_str("roughness_x")?)?;
            let roughness_y = scene.clone_texture(params.get_str("roughness_y")?)?;
            (roughness_x, roughness_y)
        };

        Ok(Glass::new(
            ior,
            reflectance,
            transmittance,
            roughness_x,
            roughness_y,
        ))
    }
}

impl MaterialT for Glass {
    fn scatter(&self, inter: &Intersection<'_>) -> Scatter {
        let reflectance = self.reflectance.color_at(inter);
        let transmittance = self.transmittance.color_at(inter);
        let roughness_x = self.roughness_x.float_at(inter, TextureChannel::R);
        let roughness_y = self.roughness_y.float_at(inter, TextureChannel::R);

        if roughness_x < 0.001 || roughness_y < 0.001 {
            FresnelDielectricRT::new(
                self.ior,
                SpecularReflect::new(reflectance),
                SpecularTransmit::new(transmittance, self.ior),
            )
            .into()
        } else {
            FresnelDielectricRT::new(
                self.ior,
                MicrofacetReflect::new(reflectance, roughness_x, roughness_y),
                MicrofacetTransmit::new(transmittance, self.ior, roughness_x, roughness_y),
            )
            .into()
        }
    }
}
