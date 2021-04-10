use crate::camera::PerspectiveCamera;
use crate::core::camera::Camera;
use crate::core::filter::Filter;
use crate::core::light::Light;
use crate::core::material::Material;
use crate::core::medium::Medium;
use crate::core::path_tracer::PathTracer;
use crate::core::primitive::{Aggregate, Primitive};
use crate::core::sampler::Sampler;
use crate::filter::BoxFilter;
use crate::light::{DirLight, PointLight, RectangleLight};
use crate::material::{Glass, Lambert, Microfacet, PseudoMaterial};
use crate::medium::Homogeneous;
use crate::primitive::{BvhAccel, Group, MeshVertex, Sphere, Transform, TriangleMesh};
use crate::sampler::{JitteredSampler, RandomSampler};
use anyhow::*;
use cgmath::{Matrix4, Point3, SquareMatrix, Vector3};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

pub struct OutputConfig {
    pub file: String,
    pub width: u32,
    pub height: u32,
}

struct InputLoader {
    path: PathBuf,
    materials: Vec<Arc<dyn Material>>,
    mediums: Vec<Arc<dyn Medium>>,
}

pub fn load<P: AsRef<Path>>(path: P) -> Result<(PathTracer, OutputConfig)> {
    let mut loader = InputLoader::new(path);
    loader.load()
}

impl InputLoader {
    fn new<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref().to_path_buf();
        Self {
            path,
            materials: vec![],
            mediums: vec![],
        }
    }

    fn load(&mut self) -> Result<(PathTracer, OutputConfig)> {
        let json_file = std::fs::File::open(&self.path)?;
        let json_reader = std::io::BufReader::new(json_file);
        let json_value: serde_json::Value = serde_json::from_reader(json_reader)?;

        let output_config_json = json_value.get("output").context("top: no 'output' field")?;
        let output_config = self.load_output(output_config_json)?;

        let camera_json = json_value.get("camera").context("top: no 'camera' field")?;
        let camera = self.load_camera(camera_json)?;

        let spp;
        let sampler;
        if let Some(pixel_sampler_json) = json_value.get("pixel_sampler") {
            spp = if let Some(spp_value) = pixel_sampler_json.get("spp") {
                spp_value
                    .as_u64()
                    .context("pixel_sampler: 'spp' should be an int")? as u32
            } else {
                1_u32
            };
            sampler = get_sampler_field(pixel_sampler_json)?;
        } else {
            spp = 1_u32;
            sampler = Box::new(RandomSampler::new()) as Box<dyn Sampler>;
        }

        let max_depth = get_int_field_option(&json_value, "top", "max_depth")?
            .or(Some(1))
            .unwrap();

        let filter = if let Some(filter_value) = json_value.get("filter") {
            self.load_filter(filter_value)?
        } else {
            Box::new(BoxFilter::new(0.5))
        };

        let materials_json = json_value
            .get("materials")
            .context("top: no 'materials' field")?;
        self.materials = self.load_materials(materials_json)?;

        let mediums_json = json_value
            .get("mediums")
            .context("top: no 'mediums' field")?;
        self.mediums = self.load_mediums(mediums_json)?;

        let objects_json = json_value
            .get("objects")
            .context("top: no 'objects' field")?;
        let objects = self.load_objects(objects_json)?;

        let aggregate_json = json_value
            .get("aggregate")
            .context("top: no 'aggregate' field")?;
        let aggregate = self.load_aggregate(aggregate_json, objects)?;

        let lights_json = json_value.get("lights").context("top: no 'lights' field")?;
        let lights = self.load_lights(lights_json)?;

        let path_tracer =
            PathTracer::new(camera, aggregate, lights, spp, sampler, max_depth, filter);

        Ok((path_tracer, output_config))
    }

    fn load_output(&self, value: &serde_json::Value) -> Result<OutputConfig> {
        let file = get_str_field(value, "output", "file")?;
        let width = get_int_field(value, "output", "width")?;
        let height = get_int_field(value, "output", "height")?;
        Ok(OutputConfig {
            file: file.to_string(),
            width,
            height,
        })
    }

    fn load_camera(&self, value: &serde_json::Value) -> Result<Box<dyn Camera>> {
        let ty = get_str_field(value, "camera", "type")?;
        match ty {
            "perspective" => {
                let eye = get_float_array3_field(value, "camera-perspective", "eye")?;
                let forward = get_float_array3_field(value, "camera-perspective", "forward")?;
                let up = get_float_array3_field(value, "camera-perspective", "up")?;
                let fov = get_float_field(value, "camera-perspective", "fov")?;
                Ok(Box::new(PerspectiveCamera::new(
                    eye.into(),
                    forward.into(),
                    up.into(),
                    fov,
                )))
            }
            _ => Err(LoadError::new(format!("camera: unknown type '{}'", ty)))?,
        }
    }

    fn load_filter(&self, value: &serde_json::Value) -> Result<Box<dyn Filter>> {
        let ty = get_str_field(value, "filter", "type")?;
        match ty {
            "box" => {
                let radius = get_float_field(value, "filter-box", "radius")?;
                Ok(Box::new(BoxFilter::new(radius)))
            }
            _ => Err(LoadError::new(format!("filter: unknown type '{}'", ty)))?,
        }
    }

    fn load_materials(&self, value: &serde_json::Value) -> Result<Vec<Arc<dyn Material>>> {
        let arr = value
            .as_array()
            .context("top: 'materials' should be an array")?;
        let mut materials = Vec::with_capacity(arr.len());
        for mat_json in arr {
            let ty = get_str_field(mat_json, "material", "type")?;
            let mat = match ty {
                "pseudo" => Arc::new(PseudoMaterial::new()) as Arc<dyn Material>,
                "lambert" => {
                    let albedo = get_float_array3_field(mat_json, "material-lambert", "albedo")?;
                    let emissive =
                        get_float_array3_field_option(mat_json, "material-lambert", "emissive")?;
                    let sampler = get_sampler_field_option(mat_json)?;
                    Arc::new(Lambert::new(albedo.into(), emissive.into(), sampler))
                        as Arc<dyn Material>
                }
                "glass" => {
                    let reflectance =
                        get_float_array3_field(mat_json, "material-glass", "reflectance")?;
                    let transmittance =
                        get_float_array3_field(mat_json, "material-glass", "transmittance")?;
                    let ior = get_float_field(mat_json, "material-glass", "ior")?;
                    let sampler = get_sampler_field_option(mat_json)?;
                    Arc::new(Glass::new(
                        reflectance.into(),
                        transmittance.into(),
                        ior,
                        sampler,
                    )) as Arc<dyn Material>
                }
                "microfacet" => {
                    let albedo = get_float_array3_field(mat_json, "material-microfacet", "albedo")?;
                    let emissive =
                        get_float_array3_field(mat_json, "material-microfacet", "emissive")?;
                    let roughness = get_float_field(mat_json, "material-microfacet", "roughness")?;
                    let metallic = get_float_field(mat_json, "material-microfacet", "metallic")?;
                    let sampler = get_sampler_field_option(mat_json)?;
                    Arc::new(Microfacet::new(
                        albedo.into(),
                        emissive.into(),
                        roughness * roughness,
                        metallic,
                        sampler,
                    )) as Arc<dyn Material>
                }
                _ => Err(LoadError::new(format!("material: unknown type '{}'", ty)))?,
            };
            materials.push(mat);
        }
        Ok(materials)
    }

    fn load_mediums(&self, value: &serde_json::Value) -> Result<Vec<Arc<dyn Medium>>> {
        let arr = value
            .as_array()
            .context("top: 'mediums' should be an array")?;
        let mut mediums = Vec::with_capacity(arr.len());
        for med_json in arr {
            let ty = get_str_field(med_json, "medium", "type")?;
            let med = match ty {
                "homogeneous" => {
                    let sigma_t =
                        get_float_array3_field(med_json, "medium-homogeneous", "sigma_t")?;
                    let sigma_s =
                        get_float_array3_field(med_json, "medium-homogeneous", "sigma_s")?;
                    let sampler = get_sampler_field_option(med_json)?;
                    Arc::new(Homogeneous::new(sigma_t.into(), sigma_s.into(), sampler))
                        as Arc<dyn Medium>
                }
                _ => Err(LoadError::new(format!("medium: unknown type '{}'", ty)))?,
            };
            mediums.push(med);
        }
        Ok(mediums)
    }

    fn load_objects(&self, value: &serde_json::Value) -> Result<Vec<Box<dyn Primitive>>> {
        let arr = value
            .as_array()
            .context("top: 'objects' should be an array")?;
        let mut objects = vec![];
        for obj_json in arr {
            let mut primitives = self.load_object(obj_json)?;
            objects.append(&mut primitives);
        }
        Ok(objects)
    }

    fn load_object(&self, value: &serde_json::Value) -> Result<Vec<Box<dyn Primitive>>> {
        let ty = get_str_field(value, "object", "type")?;
        match ty {
            "sphere" => {
                let center = get_float_array3_field(value, "object-sphere", "center")?;
                let radius = get_float_field(value, "object-sphere", "radius")?;
                let material = get_int_field(value, "object-sphere", "material")? as usize;
                let medium = get_int_field_option(value, "object-sphere", "medium")?
                    .map(|ind| self.mediums[ind as usize].clone());
                Ok(vec![Box::new(Sphere::new(
                    center.into(),
                    radius,
                    self.materials[material].clone(),
                    medium,
                )) as Box<dyn Primitive>])
            }
            "transform" => {
                let trans = self.load_transform(value, "object-transform")?;
                let prim_json = value
                    .get("primitive")
                    .context("object-transform: no 'primitive' field")?;
                let primitive = self.load_object(prim_json)?;
                Ok(primitive
                    .into_iter()
                    .map(|prim| Box::new(Transform::new(prim, trans)) as Box<dyn Primitive>)
                    .collect())
            }
            "obj_mesh" => {
                let material = get_int_field(value, "object-obj_mesh", "material")? as usize;
                let medium = get_int_field_option(value, "object-sphere", "medium")?
                    .map(|ind| self.mediums[ind as usize].clone());
                let file = get_str_field(value, "object-obj_mesh", "file")?;
                let (models, _) = tobj::load_obj(self.path.with_file_name(file), true)?;
                let mut triangles = vec![];
                for model in models {
                    let indices = model.mesh.indices;
                    let vertex_count = model.mesh.positions.len() / 3;
                    let mut vertices = vec![MeshVertex::default(); vertex_count];
                    for i in 0..vertex_count {
                        let i0 = 3 * i;
                        let i1 = 3 * i + 1;
                        let i2 = 3 * i + 2;
                        if i2 < model.mesh.positions.len() {
                            vertices[i].position = Point3::new(
                                model.mesh.positions[i0],
                                model.mesh.positions[i1],
                                model.mesh.positions[i2],
                            );
                        }
                        if i2 < model.mesh.normals.len() {
                            vertices[i].normal = Vector3::new(
                                model.mesh.normals[i0],
                                model.mesh.normals[i1],
                                model.mesh.normals[i2],
                            );
                        }
                        if 2 * i + 1 < model.mesh.texcoords.len() {
                            vertices[i].texcoords = cgmath::Point2::new(
                                model.mesh.texcoords[2 * i],
                                model.mesh.texcoords[2 * i + 1],
                            );
                        }
                    }
                    let mesh = TriangleMesh::new(
                        vertices,
                        indices,
                        self.materials[material].clone(),
                        medium.clone(),
                    );
                    let mut primitives = mesh.into_triangles();
                    triangles.append(&mut primitives);
                }
                Ok(triangles)
            }
            _ => Err(LoadError::new(format!("object: unknown type '{}'", ty)))?,
        }
    }

    fn load_transform(&self, value: &serde_json::Value, env: &str) -> Result<Matrix4<f32>> {
        let trans_json = value
            .get("transform")
            .context(format!("{}: no field 'transform'", env))?;
        let mut matrix = Matrix4::identity();
        if let Some(mat_json) = trans_json.get("matrix") {
            let error_info = format!("{}: 'matrix' should be an array with 16 floats", env);
            let mat_arr = mat_json.as_array().context(error_info.clone())?;
            if mat_arr.len() != 16 {
                Err(LoadError::new(error_info.clone()))?
            }
            matrix.x.x = mat_arr[0].as_f64().context(error_info.clone())? as f32;
            matrix.x.y = mat_arr[1].as_f64().context(error_info.clone())? as f32;
            matrix.x.z = mat_arr[2].as_f64().context(error_info.clone())? as f32;
            matrix.x.w = mat_arr[3].as_f64().context(error_info.clone())? as f32;
            matrix.y.x = mat_arr[4].as_f64().context(error_info.clone())? as f32;
            matrix.y.y = mat_arr[5].as_f64().context(error_info.clone())? as f32;
            matrix.y.z = mat_arr[6].as_f64().context(error_info.clone())? as f32;
            matrix.y.w = mat_arr[7].as_f64().context(error_info.clone())? as f32;
            matrix.z.x = mat_arr[8].as_f64().context(error_info.clone())? as f32;
            matrix.z.y = mat_arr[9].as_f64().context(error_info.clone())? as f32;
            matrix.z.z = mat_arr[10].as_f64().context(error_info.clone())? as f32;
            matrix.z.w = mat_arr[11].as_f64().context(error_info.clone())? as f32;
            matrix.w.x = mat_arr[12].as_f64().context(error_info.clone())? as f32;
            matrix.w.y = mat_arr[13].as_f64().context(error_info.clone())? as f32;
            matrix.w.z = mat_arr[14].as_f64().context(error_info.clone())? as f32;
            matrix.w.w = mat_arr[15].as_f64().context(error_info.clone())? as f32;
        }
        if let Some(_) = trans_json.get("scale") {
            let scale = get_float_array3_field(trans_json, env, "scale")?;
            matrix = Matrix4::from_nonuniform_scale(scale[0], scale[1], scale[2]) * matrix;
        }
        if let Some(_) = trans_json.get("rotate") {
            let rotate = get_float_array3_field(trans_json, env, "rotate")?;
            matrix = Matrix4::from_angle_z(cgmath::Deg(rotate[2]))
                * Matrix4::from_angle_x(cgmath::Deg(rotate[0]))
                * Matrix4::from_angle_y(cgmath::Deg(rotate[1]))
                * matrix;
        }
        if let Some(_) = trans_json.get("translate") {
            let translate = get_float_array3_field(trans_json, env, "translate")?;
            matrix =
                Matrix4::from_translation(Vector3::new(translate[0], translate[1], translate[2]))
                    * matrix;
        }
        if !matrix.is_invertible() {
            println!("WARNING: singular transform matrix found");
        }
        Ok(matrix)
    }

    fn load_lights(&self, value: &serde_json::Value) -> Result<Vec<Box<dyn Light>>> {
        let arr = value
            .as_array()
            .context("top: 'lights' should be an array")?;
        let mut lights = Vec::with_capacity(arr.len());
        for light_json in arr {
            let ty = get_str_field(light_json, "light", "type")?;
            let light = match ty {
                "point" => {
                    let position = get_float_array3_field(light_json, "light-point", "position")?;
                    let strength = get_float_array3_field(light_json, "light-point", "strength")?;
                    Box::new(PointLight::new(
                        Point3::new(position[0], position[1], position[2]),
                        strength.into(),
                    )) as Box<dyn Light>
                }
                "directional" => {
                    let direction =
                        get_float_array3_field(light_json, "light-directional", "direction")?;
                    let strength =
                        get_float_array3_field(light_json, "light-directional", "strength")?;
                    Box::new(DirLight::new(
                        Vector3::new(direction[0], direction[1], direction[2]),
                        strength.into(),
                    )) as Box<dyn Light>
                }
                "rectangle" => {
                    let center = get_float_array3_field(light_json, "light-rectangle", "center")?;
                    let direction =
                        get_float_array3_field(light_json, "light-rectangle", "direction")?;
                    let strength =
                        get_float_array3_field(light_json, "light-rectangle", "strength")?;
                    let up = get_float_array3_field(light_json, "light-rectangle", "up")?;
                    let width = get_float_field(light_json, "light-rectangle", "width")?;
                    let height = get_float_field(light_json, "light-rectangle", "height")?;
                    let sampler = get_sampler_field_option(light_json)?;
                    Box::new(RectangleLight::new(
                        Point3::new(center[0], center[1], center[2]),
                        Vector3::new(direction[0], direction[1], direction[2]),
                        width,
                        height,
                        Vector3::new(up[0], up[1], up[2]),
                        strength.into(),
                        sampler,
                    )) as Box<dyn Light>
                }
                _ => Err(LoadError::new(format!("light: unknown type '{}'", ty)))?,
            };
            lights.push(light)
        }
        Ok(lights)
    }

    fn load_aggregate(
        &self,
        value: &serde_json::Value,
        primitives: Vec<Box<dyn Primitive>>,
    ) -> Result<Box<dyn Aggregate>> {
        let ty = get_str_field(value, "aggregate", "type")?;
        match ty {
            "group" => Ok(Box::new(Group::new(primitives)) as Box<dyn Aggregate>),
            "bvh" => {
                let max_leaf_size = if let Some(leaf_json) = value.get("max_leaf_size") {
                    leaf_json
                        .as_u64()
                        .context("aggregate-bvh: 'max_leaf_size' should be an int")?
                        as usize
                } else {
                    4_usize
                };
                let bucket_number = if let Some(bucket_json) = value.get("bucket_number") {
                    bucket_json
                        .as_u64()
                        .context("aggregate-bvh: 'bucket_number' should be an int")?
                        as usize
                } else {
                    16_usize
                };
                Ok(
                    Box::new(BvhAccel::new(primitives, max_leaf_size, bucket_number))
                        as Box<dyn Aggregate>,
                )
            }
            _ => Err(LoadError::new(format!("aggregate: unknown type '{}'", ty)))?,
        }
    }
}

fn get_str_field<'a>(value: &'a serde_json::Value, env: &str, field: &str) -> Result<&'a str> {
    let field_value = value
        .get(field)
        .context(format!("{}: no '{}' field", env, field))?;
    field_value
        .as_str()
        .context(format!("{}: '{}' should be a string", env, field))
}

fn get_float_field(value: &serde_json::Value, env: &str, field: &str) -> Result<f32> {
    let field_value = value
        .get(field)
        .context(format!("{}: no '{}' field", env, field))?;
    field_value
        .as_f64()
        .map(|f| f as f32)
        .context(format!("{}: '{}' should be a float", env, field))
}

fn get_int_field_option(value: &serde_json::Value, env: &str, field: &str) -> Result<Option<u32>> {
    if let Some(field_value) = value.get(field) {
        field_value
            .as_u64()
            .map(|i| i as u32)
            .context(format!("{}: '{}' should be an int", env, field))
            .map(|i| Some(i))
    } else {
        Ok(None)
    }
}
fn get_int_field(value: &serde_json::Value, env: &str, field: &str) -> Result<u32> {
    let field_value = value
        .get(field)
        .context(format!("{}: no '{}' field", env, field))?;
    field_value
        .as_u64()
        .map(|f| f as u32)
        .context(format!("{}: '{}' should be an int", env, field))
}

fn get_float_array3_field_option(
    value: &serde_json::Value,
    env: &str,
    field: &str,
) -> Result<[f32; 3]> {
    if let Some(_) = value.get(field) {
        get_float_array3_field(value, env, field)
    } else {
        Ok([0.0, 0.0, 0.0])
    }
}

fn get_float_array3_field(value: &serde_json::Value, env: &str, field: &str) -> Result<[f32; 3]> {
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
        Err(LoadError::new(error_info.clone()))?
    }
}

fn get_sampler_field_option(value: &serde_json::Value) -> Result<Box<Mutex<dyn Sampler>>> {
    if let Some(sampler_json) = value.get("sampler") {
        get_sampler_field_mutex(sampler_json)
    } else {
        Ok(Box::new(Mutex::new(RandomSampler::new())))
    }
}

fn get_sampler_field_mutex(value: &serde_json::Value) -> Result<Box<Mutex<dyn Sampler>>> {
    let ty = get_str_field(value, "sample", "type")?;
    match ty {
        "random" => Ok(Box::new(Mutex::new(RandomSampler::new()))),
        "jittered" => {
            let division = get_int_field(value, "sampler-jittered", "division")?;
            Ok(Box::new(Mutex::new(JitteredSampler::new(division))))
        }
        _ => Err(LoadError::new(format!("sampler: unknown type '{}'", ty)))?,
    }
}

fn get_sampler_field(value: &serde_json::Value) -> Result<Box<dyn Sampler>> {
    let ty = get_str_field(value, "sample", "type")?;
    match ty {
        "random" => Ok(Box::new(RandomSampler::new())),
        "jittered" => {
            let division = get_int_field(value, "sampler-jittered", "division")?;
            Ok(Box::new(JitteredSampler::new(division)))
        }
        _ => Err(LoadError::new(format!("sampler: unknown type '{}'", ty)))?,
    }
}

#[derive(Debug)]
pub struct LoadError {
    cause: String,
}

impl LoadError {
    pub fn new<S: ToString>(cause: S) -> Self {
        Self {
            cause: cause.to_string(),
        }
    }
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.cause)
    }
}

impl std::error::Error for LoadError {}
