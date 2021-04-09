use anyhow::*;

mod camera;
mod core;
mod filter;
mod light;
mod loader;
mod material;
mod medium;
mod primitive;
mod sampler;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 1 {
        println!("Usage: simple-path-tracer <path-to-json>");
        return Ok(());
    }

    println!("Loading scene JSON and building aggregate...");
    let (mut path_tracer, output_config) = loader::load(&args[0])?;

    println!("Scene JSON is loaded successfully. Rendering...");
    println!(
        "  width = {}, height = {}, output_name = '{}'",
        output_config.width, output_config.height, output_config.file
    );
    let begin_time = std::time::SystemTime::now();
    let image = path_tracer.render(output_config.width, output_config.height);
    let duration = std::time::SystemTime::now().duration_since(begin_time)?;
    image.save(&output_config.file)?;

    println!("Finished, time used: {:?}", duration);
    Ok(())
}
