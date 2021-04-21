use crate::core::color::Color;
use crate::core::intersection::Intersection;
use crate::core::material::Material;
use crate::core::scatter::Scatter;
use crate::core::texture::Texture;
use crate::scatter::LambertReflect;
use std::sync::Arc;

pub struct Lambert {
    albedo: Arc<dyn Texture<Color>>,
    emissive: Arc<dyn Texture<Color>>,
}

impl Lambert {
    pub fn new(albedo: Arc<dyn Texture<Color>>, emissive: Arc<dyn Texture<Color>>) -> Self {
        Self { albedo, emissive }
    }
}

impl Material for Lambert {
    fn scatter(&self, inter: &Intersection<'_>) -> Box<dyn Scatter> {
        let albedo = self.albedo.value_at(inter);
        Box::new(LambertReflect::new(albedo)) as Box<dyn Scatter>
    }

    fn emissive(&self, inter: &Intersection<'_>) -> Color {
        self.emissive.value_at(inter)
    }
}
