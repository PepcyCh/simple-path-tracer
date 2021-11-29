use std::path::PathBuf;

use anyhow::*;
use structopt::StructOpt;

use crate::renderer::{OutputConfig, RendererT};

mod camera;
mod core;
mod filter;
mod light;
mod light_sampler;
mod loader;
mod material;
mod medium;
mod pixel_sampler;
mod primitive;
mod renderer;
mod scatter;
mod texture;

#[macro_use]
extern crate lazy_static;

#[derive(StructOpt)]
#[structopt(name = "simple-path-tracer")]
struct Opt {
    #[structopt(short, long, parse(from_os_str))]
    scene: PathBuf,
    #[structopt(short, long, parse(from_os_str))]
    renderer: PathBuf,
    #[structopt(short, long, default_value = "512")]
    width: u32,
    #[structopt(short, long, default_value = "512")]
    height: u32,
    #[structopt(short, long)]
    output: String,
    #[structopt(short, long)]
    camera: Option<String>,
}

fn main() -> Result<()> {
    env_logger::init();

    let opt = Opt::from_args();

    log::info!("Loading from JSON and building aggregate...");
    let scene = loader::load_scene(opt.scene)?;
    let renderer = loader::load_renderer(opt.renderer)?;
    let output_config = OutputConfig {
        width: opt.width,
        height: opt.height,
        output_filename: opt.output,
        used_camera_name: opt.camera,
    };

    log::info!("Scene JSON is loaded successfully. Rendering...");

    let begin_time = std::time::SystemTime::now();
    renderer.render(&scene, &output_config);
    let duration = std::time::SystemTime::now().duration_since(begin_time)?;

    log::info!("Finished, time used: {:?}", duration);
    Ok(())
}
