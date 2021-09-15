use crate::core::{intersection::Intersection, texture::Texture};

pub struct ScalarTex<T> {
    value: T,
}

impl<T: Copy> ScalarTex<T> {
    pub fn new(value: T) -> Self {
        Self { value }
    }
}

impl<T: Copy + Send + Sync> Texture<T> for ScalarTex<T> {
    fn value_at(&self, _inter: &Intersection<'_>) -> T {
        self.value
    }
}
