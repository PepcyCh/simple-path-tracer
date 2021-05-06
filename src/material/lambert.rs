use crate::core::color::Color;
use crate::core::intersection::Intersection;
use crate::core::material::Material;
use crate::core::scatter::Scatter;
use crate::core::texture::{self, Texture};
use crate::scatter::LambertReflect;
use std::sync::Arc;

pub struct Lambert {
    albedo: Arc<dyn Texture<Color>>,
    emissive: Arc<dyn Texture<Color>>,
    normal_map: Arc<dyn Texture<Color>>,
}

impl Lambert {
    pub fn new(
        albedo: Arc<dyn Texture<Color>>,
        emissive: Arc<dyn Texture<Color>>,
        normal_map: Arc<dyn Texture<Color>>,
    ) -> Self {
        Self {
            albedo,
            emissive,
            normal_map,
        }
    }
}

impl Material for Lambert {
    fn apply_normal_map(&self, inter: &Intersection<'_>) -> cgmath::Vector3<f32> {
        texture::get_normal_at(&self.normal_map, inter)
    }

    fn scatter(&self, inter: &Intersection<'_>) -> Box<dyn Scatter> {
        let albedo = self.albedo.value_at(inter);
        Box::new(LambertReflect::new(albedo)) as Box<dyn Scatter>
    }

    fn emissive(&self, inter: &Intersection<'_>) -> Color {
        self.emissive.value_at(inter)
    }
}
