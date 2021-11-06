use crate::core::scene::Scene;

pub struct OutputConfig {
    pub width: u32,
    pub height: u32,
    pub output_filename: String,
}

pub trait Renderer {
    fn render(&self, scene: &Scene, config: &OutputConfig);
}
