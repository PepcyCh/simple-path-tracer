use crate::core::scene::Scene;

pub trait Renderer {
    fn render(&self, scene: &Scene);
}
