use crate::core::color::Color;
use crate::core::intersection::Intersection;
use crate::core::material::Material;
use crate::core::scatter::Scatter;
use crate::core::texture::{self, Texture};
use crate::scatter::{
    FresnelDielectricRSsr, MicrofacetReflect, SpecularReflect, SubsurfaceReflect,
};
use std::sync::Arc;

pub struct Subsurface {
    ior: f32,
    albedo: Arc<dyn Texture<Color>>,
    ld: Arc<dyn Texture<f32>>,
    roughness: Arc<dyn Texture<f32>>,
    emissive: Arc<dyn Texture<Color>>,
    normal_map: Arc<dyn Texture<Color>>,
}

impl Subsurface {
    pub fn new(
        ior: f32,
        albedo: Arc<dyn Texture<Color>>,
        ld: Arc<dyn Texture<f32>>,
        roughness: Arc<dyn Texture<f32>>,
        emissive: Arc<dyn Texture<Color>>,
        normal_map: Arc<dyn Texture<Color>>,
    ) -> Self {
        Self {
            ior,
            albedo,
            ld,
            roughness,
            emissive,
            normal_map,
        }
    }
}

impl Material for Subsurface {
    fn apply_normal_map(&self, inter: &Intersection<'_>) -> cgmath::Vector3<f32> {
        texture::get_normal_at(&self.normal_map, inter)
    }

    fn scatter(&self, inter: &Intersection<'_>) -> Box<dyn Scatter> {
        let albedo = self.albedo.value_at(inter);
        let ld = self.ld.value_at(inter);
        let roughness = self.roughness.value_at(inter).powi(2);

        if roughness < 0.001 {
            Box::new(FresnelDielectricRSsr::new(
                self.ior,
                SpecularReflect::new(albedo),
                SubsurfaceReflect::new(albedo, ld, self.ior),
            )) as Box<dyn Scatter>
        } else {
            Box::new(FresnelDielectricRSsr::new(
                self.ior,
                MicrofacetReflect::new(albedo, roughness),
                SubsurfaceReflect::new(albedo, ld, self.ior),
            )) as Box<dyn Scatter>
        }
    }

    fn emissive(&self, inter: &Intersection<'_>) -> Color {
        self.emissive.value_at(inter)
    }
}
