mod gltf;
mod json;

use std::path::Path;

use crate::{core::scene::Scene, renderer::Renderer};

pub fn load_renderer<P: AsRef<Path>>(path: P) -> anyhow::Result<Renderer> {
    if let Some(ext) = path.as_ref().extension() {
        let ext = ext.to_str().unwrap();
        match ext {
            "json" => json::load_renderer(path),
            _ => anyhow::bail!(format!("File extension '{}' is not recognized", ext)),
        }
    } else {
        anyhow::bail!("There is no file extension")
    }
}

pub fn load_scene<P: AsRef<Path>>(path: P) -> anyhow::Result<Scene> {
    if let Some(ext) = path.as_ref().extension() {
        let ext = ext.to_str().unwrap();
        match ext {
            "json" => json::load_scene(path),
            "gltf" => gltf::load_scene(path),
            _ => anyhow::bail!(format!("File extension '{}' is not recognized", ext)),
        }
    } else {
        anyhow::bail!("There is no file extension")
    }
}
