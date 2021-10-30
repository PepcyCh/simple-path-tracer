use crate::core::intersection::Intersection;

pub trait Texture<T>: Send + Sync {
    fn value_at(&self, inter: &Intersection<'_>) -> T;
}
