use std::sync::Arc;

use crate::{
    core::{
        color::Color,
        intersection::Intersection,
        material::Material,
        scatter::Scatter,
        texture::{self, Texture},
    },
    scatter::{FresnelConductor, MicrofacetReflect, SpecularReflect},
};

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
    fn apply_normal_map(&self, inter: &Intersection<'_>) -> glam::Vec3A {
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
            )) as Box<dyn Scatter>
        }
    }

    fn emissive(&self, inter: &Intersection<'_>) -> Color {
        self.emissive.value_at(inter)
    }
}
