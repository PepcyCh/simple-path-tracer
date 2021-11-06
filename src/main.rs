use std::path::PathBuf;

use anyhow::*;
use structopt::StructOpt;

use crate::core::renderer::OutputConfig;

mod camera;
mod core;
mod filter;
mod light;
mod loader;
mod material;
mod medium;
mod primitive;
mod renderer;
mod sampler;
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
}

fn main() -> Result<()> {
    let opt = Opt::from_args();

    println!("Loading from JSON and building aggregate...");
    let scene = loader::load_scene(opt.scene)?;
    let renderer = loader::load_renderer(opt.renderer)?;
    let output_config = OutputConfig {
        width: opt.width,
        height: opt.height,
        output_filename: opt.output,
    };

    println!("Scene JSON is loaded successfully. Rendering...");

    let begin_time = std::time::SystemTime::now();
    renderer.render(&scene, &output_config);
    let duration = std::time::SystemTime::now().duration_since(begin_time)?;

    println!("Finished, time used: {:?}", duration);
    Ok(())
}
