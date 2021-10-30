use anyhow::*;

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

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 1 {
        println!("Usage: simple-path-tracer <path-to-json>");
        return Ok(());
    }

    println!("Loading scene JSON and building aggregate...");
    let (scene, renderer) = loader::load(&args[0])?;

    println!("Scene JSON is loaded successfully. Rendering...");

    let begin_time = std::time::SystemTime::now();
    renderer.render(&scene);
    let duration = std::time::SystemTime::now().duration_since(begin_time)?;

    println!("Finished, time used: {:?}", duration);
    Ok(())
}
