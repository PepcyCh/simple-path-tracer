mod pt;

pub use pt::*;

use crate::{core::scene::Scene, filter::Filter, pixel_sampler::PixelSampler};

pub struct OutputConfig {
    pub width: u32,
    pub height: u32,
    pub output_filename: String,
    pub used_camera_name: Option<String>,
}

#[enum_dispatch::enum_dispatch(Renderer)]
pub trait RendererT {
    fn render(&self, scene: &Scene, config: &OutputConfig);
}

#[enum_dispatch::enum_dispatch]
pub enum Renderer {
    PathTracer,
}

pub fn create_renderer(
    ty: &str,
    max_depth: u32,
    pixel_sampler: PixelSampler,
    filter: Filter,
) -> anyhow::Result<Renderer> {
    let res = match ty {
        "pt" => PathTracer::new(max_depth, pixel_sampler, filter).into(),
        _ => anyhow::bail!(format!("renderer - unknown type '{}'", ty)),
    };

    Ok(res)
}
