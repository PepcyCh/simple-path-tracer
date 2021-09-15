use crate::core::{color::Color, intersection::Intersection};

pub trait Texture<T>: Send + Sync {
    fn value_at(&self, inter: &Intersection<'_>) -> T;
}

pub fn get_normal_at<T>(tex: &T, inter: &Intersection<'_>) -> glam::Vec3A
where
    T: std::ops::Deref<Target = dyn Texture<Color>>,
{
    let value = tex.value_at(inter);
    let normal_color = value * 2.0 - Color::WHITE;
    let normal = glam::Vec3A::new(normal_color.r, normal_color.g, normal_color.b);
    normal.normalize()
}
