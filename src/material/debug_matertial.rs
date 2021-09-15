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
    texture::ScalarTex,
};

pub struct DebugMaterial {
    debug_color: Arc<dyn Texture<Color>>,
    normal_map: Option<Arc<dyn Texture<Color>>>,
}

impl DebugMaterial {
    pub fn new(debug_color: Arc<dyn Texture<Color>>, normal_map: Arc<dyn Texture<Color>>) -> Self {
        Self {
            debug_color,
            normal_map: Some(normal_map),
        }
    }

    #[allow(dead_code)]
    pub fn color_only(debug_color: Color) -> Self {
        Self {
            debug_color: Arc::new(ScalarTex::new(debug_color)),
            normal_map: None,
        }
    }
}

impl Material for DebugMaterial {
    fn apply_normal_map(&self, inter: &Intersection<'_>) -> glam::Vec3A {
        if let Some(map) = &self.normal_map {
            texture::get_normal_at(map, inter)
        } else {
            inter.normal
        }
    }

    fn scatter(&self, _: &Intersection<'_>) -> Box<dyn Scatter> {
        Box::new(LambertReflect::new(Color::BLACK)) as Box<dyn Scatter>
    }

    fn emissive(&self, inter: &Intersection<'_>) -> Color {
        self.debug_color.value_at(inter)
    }
}
