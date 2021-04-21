use crate::core::color::Color;
use crate::core::intersection::Intersection;
use crate::core::material::Material;
use crate::core::scatter::Scatter;
use crate::core::texture::Texture;
use crate::scatter::{FresnelDielectricRR, LambertReflect, MicrofacetReflect, SpecularReflect};
use std::sync::Arc;

pub struct Dielectric {
    ior: f32,
    albedo: Arc<dyn Texture<Color>>,
    roughness: Arc<dyn Texture<f32>>,
    emissive: Arc<dyn Texture<Color>>,
}

impl Dielectric {
    pub fn new(
        ior: f32,
        albedo: Arc<dyn Texture<Color>>,
        roughness: Arc<dyn Texture<f32>>,
        emissive: Arc<dyn Texture<Color>>,
    ) -> Self {
        Self {
            ior,
            albedo,
            roughness,
            emissive,
        }
    }
}

impl Material for Dielectric {
    fn scatter(&self, inter: &Intersection<'_>) -> Box<dyn Scatter> {
        let albedo = self.albedo.value_at(inter);
        let roughness = self.roughness.value_at(inter).powi(2);

        if roughness < 0.001 {
            Box::new(FresnelDielectricRR::new(
                self.ior,
                SpecularReflect::new(albedo),
                LambertReflect::new(albedo),
            )) as Box<dyn Scatter>
        } else {
            Box::new(FresnelDielectricRR::new(
                self.ior,
                MicrofacetReflect::new(albedo, roughness),
                LambertReflect::new(albedo),
            )) as Box<dyn Scatter>
        }
    }

    fn emissive(&self, inter: &Intersection<'_>) -> Color {
        self.emissive.value_at(inter)
    }
}
