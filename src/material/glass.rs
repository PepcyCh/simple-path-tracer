use crate::core::color::Color;
use crate::core::intersection::Intersection;
use crate::core::material::Material;
use crate::core::scatter::Scatter;
use crate::core::texture::Texture;
use crate::scatter::{
    FresnelDielectricRT, MicrofacetReflect, MicrofacetTransmit, SpecularReflect, SpecularTransmit,
};
use std::sync::Arc;

pub struct Glass {
    ior: f32,
    reflectance: Arc<dyn Texture<Color>>,
    transmittance: Arc<dyn Texture<Color>>,
    roughness: Arc<dyn Texture<f32>>,
}

impl Glass {
    pub fn new(
        ior: f32,
        reflectance: Arc<dyn Texture<Color>>,
        transmittance: Arc<dyn Texture<Color>>,
        roughness: Arc<dyn Texture<f32>>,
    ) -> Self {
        Self {
            ior,
            reflectance,
            transmittance,
            roughness,
        }
    }
}

impl Material for Glass {
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
