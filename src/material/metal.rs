use crate::core::color::Color;
use crate::core::intersection::Intersection;
use crate::core::material::Material;
use crate::core::scatter::Scatter;
use crate::core::texture::{self, Texture};
use crate::scatter::{FresnelConductor, MicrofacetReflect, SpecularReflect};
use std::sync::Arc;

pub struct Metal {
    ior: Arc<dyn Texture<Color>>,
    ior_k: Arc<dyn Texture<Color>>,
    roughness: Arc<dyn Texture<f32>>,
    emissive: Arc<dyn Texture<Color>>,
    normal_map: Arc<dyn Texture<Color>>,
}

impl Metal {
    pub fn new(
        ior: Arc<dyn Texture<Color>>,
        ior_k: Arc<dyn Texture<Color>>,
        roughness: Arc<dyn Texture<f32>>,
        emissive: Arc<dyn Texture<Color>>,
        normal_map: Arc<dyn Texture<Color>>,
    ) -> Self {
        Self {
            ior,
            ior_k,
            roughness,
            emissive,
            normal_map,
        }
    }
}

impl Material for Metal {
    fn apply_normal_map(&self, inter: &Intersection<'_>) -> cgmath::Vector3<f32> {
        texture::get_normal_at(&self.normal_map, inter)
    }

    fn scatter(&self, inter: &Intersection<'_>) -> Box<dyn Scatter> {
        let ior = self.ior.value_at(inter);
        let ior_k = self.ior_k.value_at(inter);
        let roughness = self.roughness.value_at(inter).powi(2);

        if roughness < 0.001 {
            Box::new(FresnelConductor::new(
                ior,
                ior_k,
                SpecularReflect::new(Color::WHITE),
            )) as Box<dyn Scatter>
        } else {
            Box::new(FresnelConductor::new(
                ior,
                ior_k,
                MicrofacetReflect::new(Color::WHITE, roughness),
            ))
        }
    }

    fn emissive(&self, inter: &Intersection<'_>) -> Color {
        self.emissive.value_at(inter)
    }
}
