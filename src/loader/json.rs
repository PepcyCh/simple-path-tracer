use std::{
    convert::TryInto,
    path::{Path, PathBuf},
};

use anyhow::Context;

use crate::{
    camera,
    core::{loader::InputParams, scene::Scene, surface::Surface},
    filter,
    light::{self, EnvLight},
    material, medium, pixel_sampler,
    primitive::{self, Instance},
    renderer::{self, Renderer},
    texture,
};

pub fn load_renderer<P: AsRef<Path>>(path: P) -> anyhow::Result<Renderer> {
    let json_file = std::fs::File::open(&path)?;
    let json_reader = std::io::BufReader::new(json_file);
    let json_value: serde_json::Value = serde_json::from_reader(json_reader)?;

    let max_depth = json_value
        .get("max_depth")
        .context("renderer - There is no 'max_depth' field")?
        .as_i64()
        .context("renderer - 'max_depth' shoule be integer")? as u32;

    let sampler_value = json_value
        .get("sampler")
        .context("renderer - There is no 'sampler' field")?;
    let mut sampler_params: InputParams = sampler_value.try_into()?;
    let sampler = pixel_sampler::create_sampler_from_params(&mut sampler_params)?;
    sampler_params.check_unused_keys();

    let filter_value = json_value
        .get("filter")
        .context("renderer - There is no 'filter' field")?;
    let mut filter_params: InputParams = filter_value.try_into()?;
    let filter = filter::create_filter_from_params(&mut filter_params)?;
    filter_params.check_unused_keys();

    let ty = json_value
        .get("type")
        .context("renderer - There is no 'type' field")?
        .as_str()
        .context("renderer - 'type' shoule be string")?;

    renderer::create_renderer(ty, max_depth, sampler, filter)
}

pub fn load_scene<P: AsRef<Path>>(path: P) -> anyhow::Result<Scene> {
    let path = path.as_ref().to_path_buf();
    let mut scene = Scene::default();

    let json_file = std::fs::File::open(&path)?;
    let json_reader = std::io::BufReader::new(json_file);
    let json_value: serde_json::Value = serde_json::from_reader(json_reader)?;

    let camera_value = json_value
        .get("cameras")
        .context("scene - There is no 'cameras' field")?;
    load_from_value_or_external(
        &mut scene,
        &path,
        camera_value,
        "json-cameras",
        &camera::create_camera_from_params,
        true,
    )?;

    let texture_value = json_value
        .get("textures")
        .context("scene - There is no 'textures' field")?;
    load_from_value_or_external(
        &mut scene,
        &path,
        texture_value,
        "json-textures",
        &texture::create_texture_from_params,
        true,
    )?;

    let material_value = json_value
        .get("materials")
        .context("scene - There is no 'materials' field")?;
    load_from_value_or_external(
        &mut scene,
        &path,
        material_value,
        "json-materials",
        &material::create_material_from_params,
        true,
    )?;

    let medium_value = json_value
        .get("mediums")
        .context("scene - There is no 'mediums' field")?;
    load_from_value_or_external(
        &mut scene,
        &path,
        medium_value,
        "json-mediums",
        &medium::create_medium_from_params,
        true,
    )?;

    let primitive_value = json_value
        .get("primitives")
        .context("scene - There is no 'primitives' field")?;
    load_from_value_or_external(
        &mut scene,
        &path,
        primitive_value,
        "json-primitives",
        &primitive::create_primitive_from_params,
        true,
    )?;

    let surface_value = json_value
        .get("surfaces")
        .context("scene - There is no 'surfaces' field")?;
    load_from_value_or_external(
        &mut scene,
        &path,
        surface_value,
        "json-surfaces",
        &Surface::load,
        true,
    )?;

    let instance_value = json_value
        .get("instances")
        .context("scene - There is no 'instances' field")?;
    load_from_value_or_external(
        &mut scene,
        &path,
        instance_value,
        "json-instances",
        &Instance::load,
        true,
    )?;

    let light_value = json_value
        .get("lights")
        .context("scene - There is no 'lights' field")?;
    load_from_value_or_external(
        &mut scene,
        &path,
        light_value,
        "json-lights",
        &light::create_light_from_params,
        true,
    )?;

    if let Some(env_value) = json_value.get("environment") {
        load_from_value_or_external(
            &mut scene,
            &path,
            env_value,
            "json-environment",
            &EnvLight::load,
            false,
        )?;
    }

    let aggregate_type = if let Some(aggr_value) = json_value.get("aggregate") {
        Some(
            aggr_value
                .as_str()
                .context("scene - 'aggregate' should be string")?,
        )
    } else {
        None
    };
    scene.build_aggregate(aggregate_type)?;

    scene.collect_shape_lights();

    Ok(scene)
}

fn load_from_object<F: Fn(&mut Scene, &mut InputParams) -> anyhow::Result<()>>(
    scene: &mut Scene,
    path: &PathBuf,
    value: &serde_json::Value,
    load_func: &F,
) -> anyhow::Result<()> {
    let mut params: InputParams = value.try_into()?;
    params.set_base_path(path.clone());
    load_func(scene, &mut params)
}

fn load_from_value_or_external<F: Fn(&mut Scene, &mut InputParams) -> anyhow::Result<()>>(
    scene: &mut Scene,
    path: &PathBuf,
    value: &serde_json::Value,
    env: &str,
    load_func: &F,
    allow_array: bool,
) -> anyhow::Result<()> {
    if let Some(json_path) = value.as_str() {
        let json_file = std::fs::File::open(path.with_file_name(json_path))
            .context(format!("{} - External json file not found", env))?;
        let json_reader = std::io::BufReader::new(json_file);
        let json_value: serde_json::Value = serde_json::from_reader(json_reader)?;
        load_from_value_or_external(scene, path, &json_value, env, load_func, allow_array)?;
    } else if let Some(array) = value.as_array() {
        if allow_array {
            for ele in array {
                load_from_object(scene, path, ele, load_func)?;
            }
        } else {
            anyhow::bail!("{} - Field should not be an array");
        }
    } else {
        load_from_object(scene, path, value, load_func)?;
    }

    Ok(())
}
