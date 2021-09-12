use crate::core::color::Color;
use crate::core::intersection::Intersection;
use crate::core::material::Material;
use crate::core::scatter::Scatter;
use crate::core::texture::{self, Texture};
use crate::scatter::LambertReflect;
use crate::texture::ScalarTex;
use std::sync::Arc;

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
    fn apply_normal_map(&self, inter: &Intersection<'_>) -> cgmath::Vector3<f32> {
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
