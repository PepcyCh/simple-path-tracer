use crate::core::{color::Color, intersection::Intersection, scatter::Scatter};

pub trait Material: Send + Sync {
    fn apply_normal_map(&self, inter: &Intersection<'_>) -> glam::Vec3A {
        inter.normal
    }

    fn scatter(&self, inter: &Intersection<'_>) -> Box<dyn Scatter>;

    fn emissive(&self, inter: &Intersection<'_>) -> Color;
}
