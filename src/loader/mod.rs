use std::borrow::Cow;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Context;

use crate::camera::PerspectiveCamera;
use crate::core::{
    color::Color,
    filter::Filter,
    primitive::{Aggregate, Primitive},
    renderer::Renderer,
    scene::{Instance, Scene},
    surface::Surface,
};
use crate::filter::BoxFilter;
use crate::light::{DirLight, EnvLight, PointLight, RectangleLight};
use crate::material::{
    Dielectric, Glass, Lambert, Metal, PndfDielectric, PndfMetal, PseudoMaterial, Subsurface,
};
use crate::medium::Homogeneous;
use crate::primitive::{BvhAccel, CatmullClark, CubicBezier, Group, Sphere, TriMesh};
use crate::renderer::PathTracer;
use crate::texture::{ImageTex, ScalarTex};

pub type JsonObject = serde_json::Map<String, serde_json::Value>;

pub trait Loadable {
    fn load(scene: &mut Scene, path: &PathBuf, json_value: &JsonObject) -> anyhow::Result<()>;
}

pub fn load<P: AsRef<Path>>(path: P) -> anyhow::Result<(Scene, Box<dyn Renderer>)> {
    let path = path.as_ref().to_path_buf();
    let mut scene = Scene::default();

    let json_file = std::fs::File::open(&path)?;
    let json_reader = std::io::BufReader::new(json_file);
    let json_value: serde_json::Value = serde_json::from_reader(json_reader)?;

    let renderer = load_renderer(&path, &json_value)?;

    load_from_object_or_external(&mut scene, &path, &json_value, "top", "camera", load_camera)?;

    load_from_array_or_external(
        &mut scene,
        &path,
        &json_value,
        "top",
        "textures",
        load_texture,
    )?;

    load_from_array_or_external(
        &mut scene,
        &path,
        &json_value,
        "top",
        "materials",
        load_material,
    )?;

    load_from_array_or_external(
        &mut scene,
        &path,
        &json_value,
        "top",
        "mediums",
        load_medium,
    )?;

    load_from_array_or_external(
        &mut scene,
        &path,
        &json_value,
        "top",
        "primitives",
        load_primitive,
    )?;

    load_from_array_or_external(
        &mut scene,
        &path,
        &json_value,
        "top",
        "surfaces",
        Surface::load,
    )?;

    load_from_array_or_external(
        &mut scene,
        &path,
        &json_value,
        "top",
        "instances",
        Instance::load,
    )?;

    load_from_array_or_external(&mut scene, &path, &json_value, "top", "lights", load_light)?;

    if json_value.get("environment").is_some() {
        load_from_object_or_external(
            &mut scene,
            &path,
            &json_value,
            "top",
            "environment",
            EnvLight::load,
        )?;
        scene
            .lights
            .push(scene.environment.as_ref().unwrap().clone());
    }

    build_aggregate(&mut scene, &json_value)?;

    scene.collect_shape_lights();

    Ok((scene, renderer))
}

fn load_renderer(
    path: &PathBuf,
    json_value: &serde_json::Value,
) -> anyhow::Result<Box<dyn Renderer>> {
    let renderer_value = json_value
        .get("renderer")
        .context("top: 'renderer' is needed but not found")?;
    let renderer_value = if let Some(renderer_json_path) = renderer_value.as_str() {
        let json_file = std::fs::File::open(path.with_file_name(renderer_json_path))
            .context("renderer: external json file not found")?;
        let json_reader = std::io::BufReader::new(json_file);
        Cow::Owned(
            serde_json::from_reader(json_reader)
                .context("renderer: failed to parse external json")?,
        )
    } else {
        Cow::Borrowed(renderer_value)
    };
    let renderer_object = renderer_value
        .as_object()
        .context("renderer: should be an object")?;

    let ty = get_str_field(renderer_object, "renderer", "type")?;
    match ty {
        "pt" => {
            let width = get_int_field(renderer_object, "renderer-pt", "width")?;
            let height = get_int_field(renderer_object, "renderer-pt", "height")?;
            let spp = get_int_field(renderer_object, "renderer-pt", "spp")?;
            let max_depth = get_int_field(renderer_object, "renderer-pt", "max_depth")?;
            let sampler = get_sampler_field(renderer_object, "renderer-pt", "sampler")?;
            let filter = load_filter(
                renderer_object
                    .get("filter")
                    .context("renderer-pt: 'filter' is needed but not fount")?
                    .as_object()
                    .context("renderer-pt: 'filter' should be an object")?,
            )?;
            let output_filename = get_str_field(renderer_object, "renderer-pt", "output_filename")?;
            Ok(Box::new(PathTracer::new(
                width,
                height,
                spp,
                max_depth,
                sampler,
                filter,
                output_filename.to_owned(),
            )) as Box<dyn Renderer>)
        }
        _ => anyhow::bail!(format!("renderer: unknown type '{}'", ty)),
    }
}

fn load_from_object_or_external<F: Fn(&mut Scene, &PathBuf, &JsonObject) -> anyhow::Result<()>>(
    scene: &mut Scene,
    path: &PathBuf,
    json_value: &serde_json::Value,
    env: &str,
    field: &str,
    load_func: F,
) -> anyhow::Result<()> {
    let value = json_value
        .get(field)
        .context(format!("{}: '{}' is needed but not found", env, field))?;
    let value = if let Some(json_path) = value.as_str() {
        let json_file = std::fs::File::open(path.with_file_name(json_path))
            .context(format!("{}: external json file not found", field))?;
        let json_reader = std::io::BufReader::new(json_file);
        Cow::Owned(
            serde_json::from_reader(json_reader)
                .context(format!("{}: failed to parse external json", field))?,
        )
    } else {
        Cow::Borrowed(value)
    };
    let object = value
        .as_object()
        .context(format!("{}: should be an object", field))?;

    load_func(scene, path, object)
}

fn load_from_array_or_external<F: Fn(&mut Scene, &PathBuf, &JsonObject) -> anyhow::Result<()>>(
    scene: &mut Scene,
    path: &PathBuf,
    json_value: &serde_json::Value,
    env: &str,
    field: &str,
    load_func: F,
) -> anyhow::Result<()> {
    let value = json_value
        .get(field)
        .context(format!("{}: '{}' is needed but not found", env, field))?;
    let value = if let Some(json_path) = value.as_str() {
        let json_file = std::fs::File::open(path.with_file_name(json_path))
            .context(format!("{}: external json file not found", field))?;
        let json_reader = std::io::BufReader::new(json_file);
        Cow::Owned(
            serde_json::from_reader(json_reader)
                .context(format!("{}: failed to parse external json", field))?,
        )
    } else {
        Cow::Borrowed(value)
    };
    let array = value
        .as_array()
        .context(format!("{}: should be an array", field))?;

    for (id, ele) in array.iter().enumerate() {
        let value = if let Some(json_path) = ele.as_str() {
            let json_file = std::fs::File::open(path.with_file_name(json_path))
                .context(format!("{}[{}]: external json file not found", field, id))?;
            let json_reader = std::io::BufReader::new(json_file);
            Cow::Owned(
                serde_json::from_reader(json_reader)
                    .context(format!("{}[{}]: failed to parse external json", field, id))?,
            )
        } else {
            Cow::Borrowed(ele)
        };
        let object = value
            .as_object()
            .context(format!("{}[{}]: should be an object", field, id))?;

        load_func(scene, path, object)?;
    }

    Ok(())
}

fn load_camera(scene: &mut Scene, path: &PathBuf, json_value: &JsonObject) -> anyhow::Result<()> {
    let ty = get_str_field(json_value, "camera", "type")?;
    match ty {
        "perspective" => PerspectiveCamera::load(scene, path, json_value)?,
        _ => anyhow::bail!(format!("camera: unknown type '{}'", ty)),
    };
    Ok(())
}

fn load_texture(scene: &mut Scene, path: &PathBuf, json_value: &JsonObject) -> anyhow::Result<()> {
    let ty = get_str_field(json_value, "texture", "type")?;
    match ty {
        "scalar" => {
            let ele_ty = get_str_field(json_value, "texture", "ele")?;
            match ele_ty {
                "color" => ScalarTex::<Color>::load(scene, path, json_value)?,
                "float" => ScalarTex::<f32>::load(scene, path, json_value)?,
                _ => anyhow::bail!(format!("texture-scalar: unknown element type '{}'", ele_ty)),
            }
        }
        "image" => ImageTex::load(scene, path, json_value)?,
        _ => anyhow::bail!(format!("texture: unknown type '{}'", ty)),
    };
    Ok(())
}

fn load_material(scene: &mut Scene, path: &PathBuf, json_value: &JsonObject) -> anyhow::Result<()> {
    let ty = get_str_field(json_value, "material", "type")?;
    match ty {
        "pseudo" => PseudoMaterial::load(scene, path, json_value)?,
        "lambert" => Lambert::load(scene, path, json_value)?,
        "dielectric" => Dielectric::load(scene, path, json_value)?,
        "metal" => Metal::load(scene, path, json_value)?,
        "glass" => Glass::load(scene, path, json_value)?,
        "subsurface" => Subsurface::load(scene, path, json_value)?,
        "pndf_dielectric" => PndfDielectric::load(scene, path, json_value)?,
        "pndf_metal" => PndfMetal::load(scene, path, json_value)?,
        _ => anyhow::bail!(format!("material: unknown type '{}'", ty)),
    };
    Ok(())
}

fn load_medium(scene: &mut Scene, path: &PathBuf, json_value: &JsonObject) -> anyhow::Result<()> {
    let ty = get_str_field(json_value, "medium", "type")?;
    match ty {
        "homogeneous" => Homogeneous::load(scene, path, json_value)?,
        _ => anyhow::bail!(format!("medium: unknown type '{}'", ty)),
    };
    Ok(())
}

fn load_primitive(
    scene: &mut Scene,
    path: &PathBuf,
    json_value: &JsonObject,
) -> anyhow::Result<()> {
    let ty = get_str_field(json_value, "primitive", "type")?;
    match ty {
        "sphere" => Sphere::load(scene, path, json_value)?,
        "trimesh" => TriMesh::load(scene, path, json_value)?,
        "cubic_bezier" => CubicBezier::load(scene, path, json_value)?,
        "catmull_clark" => CatmullClark::load(scene, path, json_value)?,
        _ => anyhow::bail!(format!("primitive: unknown type '{}'", ty)),
    };
    Ok(())
}

fn load_light(scene: &mut Scene, path: &PathBuf, json_value: &JsonObject) -> anyhow::Result<()> {
    let ty = get_str_field(json_value, "light", "type")?;
    match ty {
        "point" => PointLight::load(scene, path, json_value)?,
        "directional" => DirLight::load(scene, path, json_value)?,
        "rectangle" => RectangleLight::load(scene, path, json_value)?,
        _ => anyhow::bail!(format!("light: unknown type '{}'", ty)),
    };
    Ok(())
}

fn load_filter(value: &JsonObject) -> anyhow::Result<Box<dyn Filter>> {
    let ty = get_str_field(value, "filter", "type")?;
    match ty {
        "box" => {
            let radius = get_float_field(value, "filter-box", "radius")?;
            Ok(Box::new(BoxFilter::new(radius)))
        }
        _ => anyhow::bail!(format!("filter: unknown type '{}'", ty)),
    }
}

fn build_aggregate(scene: &mut Scene, json_value: &serde_json::Value) -> anyhow::Result<()> {
    let instances = scene
        .instances
        .iter()
        .map(|(_, inst)| inst.clone() as Arc<dyn Primitive>)
        .collect::<Vec<_>>();

    let ty = json_value.get("aggregate");
    let aggregate = if let Some(ty) = ty {
        let ty = ty.as_str().context("top: 'aggregate' should be a string")?;

        match ty {
            "group" => Arc::new(Group::new(instances)) as Arc<dyn Aggregate>,
            "bvh" => Arc::new(BvhAccel::new(instances, 4, 16)) as Arc<dyn Aggregate>,
            _ => anyhow::bail!(format!("top: unknown aggregate type '{}'", ty)),
        }
    } else {
        Arc::new(BvhAccel::new(instances, 4, 16)) as Arc<dyn Aggregate>
    };
    scene.aggregate = Some(aggregate);

    Ok(())
}

// utils

pub fn get_str_field<'a>(value: &'a JsonObject, env: &str, field: &str) -> anyhow::Result<&'a str> {
    let field_value = value
        .get(field)
        .context(format!("{}: no '{}' field", env, field))?;
    field_value
        .as_str()
        .context(format!("{}: '{}' should be a string", env, field))
}

pub fn get_float_field(value: &JsonObject, env: &str, field: &str) -> anyhow::Result<f32> {
    let field_value = value
        .get(field)
        .context(format!("{}: no '{}' field", env, field))?;
    field_value
        .as_f64()
        .map(|f| f as f32)
        .context(format!("{}: '{}' should be a float", env, field))
}

pub fn get_int_field_or(
    value: &JsonObject,
    env: &str,
    field: &str,
    default: u32,
) -> anyhow::Result<u32> {
    if let Some(_) = value.get(field) {
        get_int_field(value, env, field)
    } else {
        Ok(default)
    }
}

pub fn get_int_field(value: &JsonObject, env: &str, field: &str) -> anyhow::Result<u32> {
    let field_value = value
        .get(field)
        .context(format!("{}: no '{}' field", env, field))?;
    field_value
        .as_u64()
        .map(|f| f as u32)
        .context(format!("{}: '{}' should be an int", env, field))
}

pub fn get_bool_field_or(
    value: &JsonObject,
    env: &str,
    field: &str,
    default: bool,
) -> anyhow::Result<bool> {
    if let Some(_) = value.get(field) {
        get_bool_field(value, env, field)
    } else {
        Ok(default)
    }
}

pub fn get_bool_field(value: &JsonObject, env: &str, field: &str) -> anyhow::Result<bool> {
    let field_value = value
        .get(field)
        .context(format!("{}: no '{}' field", env, field))?;
    field_value
        .as_bool()
        .context(format!("{}: '{}' should be a bool", env, field))
}

pub fn get_float_array2_field_or(
    value: &JsonObject,
    env: &str,
    field: &str,
    default: [f32; 2],
) -> anyhow::Result<[f32; 2]> {
    if let Some(_) = value.get(field) {
        get_float_array2_field(value, env, field)
    } else {
        Ok(default)
    }
}

pub fn get_float_array2_field(
    value: &JsonObject,
    env: &str,
    field: &str,
) -> anyhow::Result<[f32; 2]> {
    let field_value = value
        .get(field)
        .context(format!("{}: no '{}' field", env, field))?;
    let error_info = format!("{}: '{}' should be an array with 2 floats", env, field);
    let arr = field_value.as_array().context(error_info.clone())?;
    if arr.len() == 2 {
        let arr0 = arr[0].as_f64().context(error_info.clone())? as f32;
        let arr1 = arr[1].as_f64().context(error_info.clone())? as f32;
        Ok([arr0, arr1])
    } else {
        anyhow::bail!(error_info)
    }
}

pub fn get_float_array3_field_or(
    value: &JsonObject,
    env: &str,
    field: &str,
    default: [f32; 3],
) -> anyhow::Result<[f32; 3]> {
    if let Some(_) = value.get(field) {
        get_float_array3_field(value, env, field)
    } else {
        Ok(default)
    }
}

pub fn get_float_array3_field(
    value: &JsonObject,
    env: &str,
    field: &str,
) -> anyhow::Result<[f32; 3]> {
    let field_value = value
        .get(field)
        .context(format!("{}: no '{}' field", env, field))?;
    let error_info = format!("{}: '{}' should be an array with 3 floats", env, field);
    let arr = field_value.as_array().context(error_info.clone())?;
    if arr.len() == 3 {
        let arr0 = arr[0].as_f64().context(error_info.clone())? as f32;
        let arr1 = arr[1].as_f64().context(error_info.clone())? as f32;
        let arr2 = arr[2].as_f64().context(error_info.clone())? as f32;
        Ok([arr0, arr1, arr2])
    } else {
        anyhow::bail!(error_info)
    }
}

pub fn get_sampler_field(
    value: &JsonObject,
    env: &str,
    field: &str,
) -> anyhow::Result<&'static str> {
    let ty = get_str_field(value, env, field)?;
    match ty {
        "random" => Ok("random"),
        "jittered" => Ok("jittered"),
        _ => anyhow::bail!(format!("sampler: unknown type '{}'", ty)),
    }
}

pub fn get_image_field(
    value: &JsonObject,
    env: &str,
    field: &str,
    dir: &PathBuf,
) -> anyhow::Result<image::DynamicImage> {
    let path = dir.with_file_name(get_str_field(value, env, field)?);
    let path_str = path.to_str().unwrap();
    image::open(path_str).context(format!(
        "{}: '{}', can't find image '{}'",
        env, field, path_str
    ))
}

pub fn get_exr_image(path: &PathBuf) -> anyhow::Result<Vec<Vec<Color>>> {
    Ok(exr::image::read::read_first_rgba_layer_from_file(
        path,
        |resolution: exr::math::Vec2<usize>, _| {
            vec![vec![Color::BLACK; resolution.width()]; resolution.height()]
        },
        |image, pos, (r, g, b, _): (f32, f32, f32, f32)| {
            image[pos.height()][pos.width()] = Color::new(r, g, b)
        },
    )?
    .layer_data
    .channel_data
    .pixels)
}
