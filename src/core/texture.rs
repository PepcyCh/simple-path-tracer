use cgmath::InnerSpace;
use crate::core::intersection::Intersection;
use crate::core::color::Color;

pub trait Texture<T>: Send + Sync {
    fn value_at(&self, inter: &Intersection<'_>) -> T;
}

pub fn get_normal_at<T>(tex: &T, inter: &Intersection<'_>) -> cgmath::Vector3<f32>
where
    T: std::ops::Deref<Target = dyn Texture<Color>>
{
    let value = tex.value_at(inter);
    let normal_color = value * 2.0 - Color::WHITE;
    let normal = cgmath::Vector3::new(normal_color.r, normal_color.g, normal_color.b);
    normal.normalize()
}