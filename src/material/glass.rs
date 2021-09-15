use std::sync::Arc;

use crate::{
    core::{
        color::Color,
        intersection::Intersection,
        material::Material,
        scatter::Scatter,
        texture::{self, Texture},
    },
    scatter::{
        FresnelDielectricRT, MicrofacetReflect, MicrofacetTransmit, SpecularReflect,
        SpecularTransmit,
    },
};

pub struct Glass {
    ior: f32,
    reflectance: Arc<dyn Texture<Color>>,
    transmittance: Arc<dyn Texture<Color>>,
    roughness: Arc<dyn Texture<f32>>,
    normal_map: Arc<dyn Texture<Color>>,
}

impl Glass {
    pub fn new(
        ior: f32,
        reflectance: Arc<dyn Texture<Color>>,
        transmittance: Arc<dyn Texture<Color>>,
        roughness: Arc<dyn Texture<f32>>,
        normal_map: Arc<dyn Texture<Color>>,
    ) -> Self {
        Self {
            ior,
            reflectance,
            transmittance,
            roughness,
            normal_map,
        }
    }
}

impl Material for Glass {
    fn apply_normal_map(&self, inter: &Intersection<'_>) -> glam::Vec3A {
        texture::get_normal_at(&self.normal_map, inter)
    }

    fn scatter(&self, inter: &Intersection<'_>) -> Box<dyn Scatter> {
        let reflectance = self.reflectance.value_at(inter);
        let transmittance = self.transmittance.value_at(inter);
        let roughness = self.roughness.value_at(inter);

        if roughness < 0.001 {
            Box::new(FresnelDielectricRT::new(
                self.ior,
                SpecularReflect::new(reflectance),
                SpecularTransmit::new(transmittance, self.ior),
            )) as Box<dyn Scatter>
        } else {
            Box::new(FresnelDielectricRT::new(
                self.ior,
                MicrofacetReflect::new(reflectance, roughness),
                MicrofacetTransmit::new(transmittance, self.ior, roughness),
            )) as Box<dyn Scatter>
        }
    }

    fn emissive(&self, _inter: &Intersection<'_>) -> Color {
        Color::BLACK
    }
}
