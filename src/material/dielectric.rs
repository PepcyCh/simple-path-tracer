use std::sync::Arc;

use crate::{
    core::{
        color::Color,
        intersection::Intersection,
        material::Material,
        scatter::Scatter,
        texture::{self, Texture},
    },
    scatter::{FresnelDielectricRR, LambertReflect, MicrofacetReflect, SpecularReflect},
};

pub struct Dielectric {
    ior: f32,
    albedo: Arc<dyn Texture<Color>>,
    roughness: Arc<dyn Texture<f32>>,
    emissive: Arc<dyn Texture<Color>>,
    normal_map: Arc<dyn Texture<Color>>,
}

impl Dielectric {
    pub fn new(
        ior: f32,
        albedo: Arc<dyn Texture<Color>>,
        roughness: Arc<dyn Texture<f32>>,
        emissive: Arc<dyn Texture<Color>>,
        normal_map: Arc<dyn Texture<Color>>,
    ) -> Self {
        Self {
            ior,
            albedo,
            roughness,
            emissive,
            normal_map,
        }
    }
}

impl Material for Dielectric {
    fn apply_normal_map(&self, inter: &Intersection<'_>) -> glam::Vec3A {
        texture::get_normal_at(&self.normal_map, inter)
    }

    fn scatter(&self, inter: &Intersection<'_>) -> Box<dyn Scatter> {
        let albedo = self.albedo.value_at(inter);
        let roughness = self.roughness.value_at(inter).powi(2);

        if roughness < 0.001 {
            Box::new(FresnelDielectricRR::new(
                self.ior,
                SpecularReflect::new(Color::WHITE),
                LambertReflect::new(albedo),
            )) as Box<dyn Scatter>
        } else {
            Box::new(FresnelDielectricRR::new(
                self.ior,
                MicrofacetReflect::new(Color::WHITE, roughness),
                LambertReflect::new(albedo),
            )) as Box<dyn Scatter>
        }
    }

    fn emissive(&self, inter: &Intersection<'_>) -> Color {
        self.emissive.value_at(inter)
    }
}
