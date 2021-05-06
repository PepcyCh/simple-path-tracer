use crate::core::color::Color;
use crate::core::intersection::Intersection;
use crate::core::scatter::Scatter;

pub trait Material: Send + Sync {
    fn apply_normal_map(&self, inter: &Intersection<'_>) -> cgmath::Vector3<f32> {
        inter.normal
    }

    fn scatter(&self, inter: &Intersection<'_>) -> Box<dyn Scatter>;

    fn emissive(&self, inter: &Intersection<'_>) -> Color;
}
