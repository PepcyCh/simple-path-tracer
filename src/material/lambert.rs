use std::sync::Arc;

use crate::{
    core::{
        color::Color,
        intersection::Intersection,
        material::Material,
        scatter::Scatter,
        texture::{self, Texture},
    },
    scatter::LambertReflect,
};

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
    fn apply_normal_map(&self, inter: &Intersection<'_>) -> glam::Vec3A {
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
