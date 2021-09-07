use crate::camera::PerspectiveCamera;
use crate::core::camera::Camera;
use crate::core::color::Color;
use crate::core::filter::Filter;
use crate::core::light::Light;
use crate::core::material::Material;
use crate::core::medium::Medium;
use crate::core::primitive::{Aggregate, Primitive};
use crate::core::texture::Texture;
use crate::filter::BoxFilter;
use crate::light::{DirLight, EnvLight, PointLight, RectangleLight};
use crate::material::{
    Dielectric, Glass, Lambert, Metal, PndfDielectric, PndfMetal, PseudoMaterial, Subsurface,
};
use crate::medium::Homogeneous;
use crate::primitive::{BvhAccel, CubicBezier, Group, MeshVertex, Sphere, Transform, TriangleMesh};
use crate::renderer::PathTracer;
use crate::texture::{ScalarTex, UvMap};
use anyhow::*;
use cgmath::{Matrix4, Point3, SquareMatrix, Vector3};
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub struct OutputConfig {
    pub file: String,
    pub width: u32,
    pub height: u32,
}

struct InputLoader {
    path: PathBuf,
    textures_float: Vec<Arc<dyn Texture<f32>>>,
    textures_color: Vec<Arc<dyn Texture<Color>>>,
    textures_indices: Vec<usize>,
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
            textures_color: vec![],
            textures_float: vec![],
            textures_indices: vec![],
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
        let sampler_type;
        if let Some(pixel_sampler_json) = json_value.get("pixel_sampler") {
            spp = if let Some(spp_value) = pixel_sampler_json.get("spp") {
                spp_value
                    .as_u64()
                    .context("pixel_sampler: 'spp' should be an int")? as u32
            } else {
                1_u32
            };
            sampler_type = get_sampler_type(pixel_sampler_json)?;
        } else {
            spp = 1_u32;
            sampler_type = "random";
        }

        let max_depth = get_int_field_option(&json_value, "top", "max_depth")?
            .or(Some(1))
            .unwrap();

        let filter = if let Some(filter_value) = json_value.get("filter") {
            self.load_filter(filter_value)?
        } else {
            Box::new(BoxFilter::new(0.5))
        };

        let textures_json = json_value
            .get("textures")
            .context("top: no 'textures' field")?;
        let (textures_float, textures_color, textures_indices) =
            self.load_textures(textures_json)?;
        self.textures_float = textures_float;
        self.textures_color = textures_color;
        self.textures_indices = textures_indices;

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
        let mut lights = self.load_lights(lights_json)?;

        let environment = if let Some(environment_json) = json_value.get("environment") {
            let env = self.load_environment(environment_json)?;
            lights.push(env.clone() as Arc<dyn Light>);
            Some(env)
        } else {
            None
        };

        let path_tracer = PathTracer::new(
            camera,
            aggregate,
            lights,
            environment,
            spp,
            sampler_type,
            max_depth,
            filter,
        );

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
            _ => bail!(format!("camera: unknown type '{}'", ty)),
        }
    }

    fn load_filter(&self, value: &serde_json::Value) -> Result<Box<dyn Filter>> {
        let ty = get_str_field(value, "filter", "type")?;
        match ty {
            "box" => {
                let radius = get_float_field(value, "filter-box", "radius")?;
                Ok(Box::new(BoxFilter::new(radius)))
            }
            _ => bail!(format!("filter: unknown type '{}'", ty)),
        }
    }

    fn load_textures(
        &self,
        value: &serde_json::Value,
    ) -> Result<(
        Vec<Arc<dyn Texture<f32>>>,
        Vec<Arc<dyn Texture<Color>>>,
        Vec<usize>,
    )> {
        let arr = value
            .as_array()
            .context("top: 'textures' should be an array")?;
        let mut textures_float = Vec::with_capacity(arr.len());
        let mut textures_color = Vec::with_capacity(arr.len());
        let mut textures_indices = Vec::with_capacity(arr.len());
        for tex_json in arr {
            let ty = get_str_field(tex_json, "texture", "type")?;
            let ele = get_str_field(tex_json, "texture", "ele")?;
            match ele {
                "float" => {
                    let tex = match ty {
                        "scalar" => {
                            let value = get_float_field(tex_json, "texture-scalar", "value")?;
                            Arc::new(ScalarTex::new(value)) as Arc<dyn Texture<f32>>
                        }
                        "uvmap" => {
                            let value =
                                get_image_field(tex_json, "texture-uvmap", "value", &self.path)?;
                            let tiling = get_float_array2_field_or_default(
                                tex_json,
                                "texture-uvmap",
                                "tiling",
                                [1.0, 1.0],
                            )?
                            .into();
                            let offset = get_float_array2_field_or_default(
                                tex_json,
                                "texture-uvmap",
                                "offset",
                                [0.0, 0.0],
                            )?
                            .into();
                            Arc::new(UvMap::new(value, tiling, offset)) as Arc<dyn Texture<f32>>
                        }
                        _ => bail!(format!("texture: unknown type: '{}'", ty)),
                    };
                    textures_indices.push(textures_float.len());
                    textures_float.push(tex);
                }
                "color" => {
                    let tex = match ty {
                        "scalar" => {
                            let value =
                                get_float_array3_field(tex_json, "texture-scalar", "value")?;
                            Arc::new(ScalarTex::new(value.into())) as Arc<dyn Texture<Color>>
                        }
                        "uvmap" => {
                            let value =
                                get_image_field(tex_json, "texture-uvmap", "value", &self.path)?;
                            let tiling = get_float_array2_field_or_default(
                                tex_json,
                                "texture-uvmap",
                                "tiling",
                                [1.0, 1.0],
                            )?
                            .into();
                            let offset = get_float_array2_field_or_default(
                                tex_json,
                                "texture-uvmap",
                                "offset",
                                [0.0, 0.0],
                            )?
                            .into();
                            Arc::new(UvMap::new(value, tiling, offset)) as Arc<dyn Texture<Color>>
                        }
                        _ => bail!(format!("texture: unknown type: '{}'", ty)),
                    };
                    textures_indices.push(textures_color.len());
                    textures_color.push(tex);
                }
                _ => bail!(format!("texture: unknown element type: '{}'", ele)),
            }
        }
        Ok((textures_float, textures_color, textures_indices))
    }

    fn load_materials(&self, value: &serde_json::Value) -> Result<Vec<Arc<dyn Material>>> {
        let arr = value
            .as_array()
            .context("top: 'materials' should be an array")?;
        let mut materials = Vec::with_capacity(arr.len());
        let default_normal_map =
            Arc::new(ScalarTex::new(Color::new(0.5, 0.5, 1.0))) as Arc<dyn Texture<Color>>;
        for mat_json in arr {
            let ty = get_str_field(mat_json, "material", "type")?;
            let mat = match ty {
                "pseudo" => Arc::new(PseudoMaterial::new()) as Arc<dyn Material>,
                "lambert" => {
                    let albedo = get_int_field(mat_json, "material-lambert", "albedo")? as usize;
                    let emissive =
                        get_int_field(mat_json, "material-lambert", "emissive")? as usize;
                    let normal = get_int_field_option(mat_json, "material-lambert", "normal_map")?;
                    Arc::new(Lambert::new(
                        self.get_texture_color(albedo),
                        self.get_texture_color(emissive),
                        normal.map_or(default_normal_map.clone(), |ind| {
                            self.get_texture_color(ind as usize)
                        }),
                    )) as Arc<dyn Material>
                }
                "glass" => {
                    let ior = get_float_field(mat_json, "material-glass", "ior")?;
                    let reflectance =
                        get_int_field(mat_json, "material-glass", "reflectance")? as usize;
                    let transmittance =
                        get_int_field(mat_json, "material-glass", "transmittance")? as usize;
                    let roughness =
                        get_int_field(mat_json, "material-glass", "roughness")? as usize;
                    let normal = get_int_field_option(mat_json, "material-glass", "normal_map")?;
                    Arc::new(Glass::new(
                        ior,
                        self.get_texture_color(reflectance),
                        self.get_texture_color(transmittance),
                        self.get_texture_float(roughness),
                        normal.map_or(default_normal_map.clone(), |ind| {
                            self.get_texture_color(ind as usize)
                        }),
                    )) as Arc<dyn Material>
                }
                "dielectric" => {
                    let ior = get_float_field(mat_json, "material-dielectric", "ior")?;
                    let albedo = get_int_field(mat_json, "material-dielectric", "albedo")? as usize;
                    let roughness =
                        get_int_field(mat_json, "material-dielectric", "roughness")? as usize;
                    let emissive =
                        get_int_field(mat_json, "material-dielectric", "emissive")? as usize;
                    let normal =
                        get_int_field_option(mat_json, "material-dielectric", "normal_map")?;
                    Arc::new(Dielectric::new(
                        ior,
                        self.get_texture_color(albedo),
                        self.get_texture_float(roughness),
                        self.get_texture_color(emissive),
                        normal.map_or(default_normal_map.clone(), |ind| {
                            self.get_texture_color(ind as usize)
                        }),
                    )) as Arc<dyn Material>
                }
                "metal" => {
                    let ior = get_int_field(mat_json, "material-metal", "ior")? as usize;
                    let ior_k = get_int_field(mat_json, "material-metal", "ior_k")? as usize;
                    let roughness =
                        get_int_field(mat_json, "material-metal", "roughness")? as usize;
                    let emissive = get_int_field(mat_json, "material-metal", "emissive")? as usize;
                    let normal = get_int_field_option(mat_json, "material-metal", "normal_map")?;
                    Arc::new(Metal::new(
                        self.get_texture_color(ior),
                        self.get_texture_color(ior_k),
                        self.get_texture_float(roughness),
                        self.get_texture_color(emissive),
                        normal.map_or(default_normal_map.clone(), |ind| {
                            self.get_texture_color(ind as usize)
                        }),
                    )) as Arc<dyn Material>
                }
                "subsurface" => {
                    let ior = get_float_field(mat_json, "material-subsurface", "ior")?;
                    let albedo = get_int_field(mat_json, "material-subsurface", "albedo")? as usize;
                    let ld = get_int_field(mat_json, "material-subsurface", "ld")? as usize;
                    let roughness =
                        get_int_field(mat_json, "material-subsurface", "roughness")? as usize;
                    let emissive =
                        get_int_field(mat_json, "material-subsurface", "emissive")? as usize;
                    let normal =
                        get_int_field_option(mat_json, "material-subsurface", "normal_map")?;
                    Arc::new(Subsurface::new(
                        ior,
                        self.get_texture_color(albedo),
                        self.get_texture_float(ld),
                        self.get_texture_float(roughness),
                        self.get_texture_color(emissive),
                        normal.map_or(default_normal_map.clone(), |ind| {
                            self.get_texture_color(ind as usize)
                        }),
                    )) as Arc<dyn Material>
                }
                "pndf_dielectric" => {
                    let ior = get_float_field(mat_json, "material-pndf", "ior")?;
                    let albedo = get_int_field(mat_json, "material-pndf", "albedo")? as usize;
                    let sigma_r = get_float_field(mat_json, "material-pndf", "sigma_r")?;
                    let base_normal =
                        get_image_field(mat_json, "material-pndf", "base_normal", &self.path)?;
                    let base_normal_tiling = get_float_array2_field_or_default(
                        mat_json,
                        "material-pndf",
                        "base_normal_tiling",
                        [1.0, 1.0],
                    )?
                    .into();
                    let base_normal_offset = get_float_array2_field_or_default(
                        mat_json,
                        "material-pndf",
                        "base_normal_offset",
                        [0.0, 0.0],
                    )?
                    .into();
                    let fallback_roughness =
                        get_int_field(mat_json, "material-pndf", "fallback_roughness")? as usize;
                    let h = get_float_field(mat_json, "material-pndf", "h")?;
                    let emissive = get_int_field(mat_json, "material-pndf", "emissive")? as usize;
                    let normal = get_int_field_option(mat_json, "material-pndf", "normal_map")?;
                    Arc::new(PndfDielectric::new(
                        ior,
                        self.get_texture_color(albedo),
                        sigma_r,
                        base_normal,
                        base_normal_tiling,
                        base_normal_offset,
                        self.get_texture_float(fallback_roughness),
                        h,
                        self.get_texture_color(emissive),
                        normal.map_or(default_normal_map.clone(), |ind| {
                            self.get_texture_color(ind as usize)
                        }),
                    )) as Arc<dyn Material>
                }
                "pndf_metal" => {
                    let albedo = get_int_field(mat_json, "material-pndf", "albedo")? as usize;
                    let sigma_r = get_float_field(mat_json, "material-pndf", "sigma_r")?;
                    let base_normal =
                        get_image_field(mat_json, "material-pndf", "base_normal", &self.path)?;
                    let base_normal_tiling = get_float_array2_field_or_default(
                        mat_json,
                        "material-pndf",
                        "base_normal_tiling",
                        [1.0, 1.0],
                    )?
                    .into();
                    let base_normal_offset = get_float_array2_field_or_default(
                        mat_json,
                        "material-pndf",
                        "base_normal_offset",
                        [0.0, 0.0],
                    )?
                    .into();
                    let fallback_roughness =
                        get_int_field(mat_json, "material-pndf", "fallback_roughness")? as usize;
                    let h = get_float_field(mat_json, "material-pndf", "h")?;
                    let emissive = get_int_field(mat_json, "material-pndf", "emissive")? as usize;
                    let normal = get_int_field_option(mat_json, "material-pndf", "normal_map")?;
                    Arc::new(PndfMetal::new(
                        self.get_texture_color(albedo),
                        sigma_r,
                        base_normal,
                        base_normal_tiling,
                        base_normal_offset,
                        self.get_texture_float(fallback_roughness),
                        h,
                        self.get_texture_color(emissive),
                        normal.map_or(default_normal_map.clone(), |ind| {
                            self.get_texture_color(ind as usize)
                        }),
                    )) as Arc<dyn Material>
                }
                _ => bail!(format!("material: unknown type '{}'", ty)),
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
                    let sigma_a =
                        get_float_array3_field(med_json, "medium-homogeneous", "sigma_a")?;
                    let sigma_s =
                        get_float_array3_field(med_json, "medium-homogeneous", "sigma_s")?;
                    let asymmetric = get_float_field(med_json, "medium-homogeneous", "asymmetric")?;
                    Arc::new(Homogeneous::new(sigma_a.into(), sigma_s.into(), asymmetric))
                        as Arc<dyn Medium>
                }
                _ => bail!(format!("medium: unknown type '{}'", ty)),
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
            "cubic_bezier" => {
                let material = get_int_field(value, "object-cubic_bezier", "material")? as usize;
                let cp_value = value
                    .get("control_points")
                    .context("object-cubic_bezier: no 'control_points' field")?;
                let error_info =
                    "object-cubic_bezier: 'control_points' should be a 4x4 array of float3";
                let cp_arr = cp_value.as_array().context(error_info)?;
                if cp_arr.len() != 4 {
                    bail!(error_info);
                }
                let mut control_points = [[Point3::new(0.0, 0.0, 0.0); 4]; 4];
                for i in 0..4 {
                    let cp_row_arr = cp_arr[i].as_array().context(error_info)?;
                    if cp_row_arr.len() != 4 {
                        bail!(error_info);
                    }
                    for j in 0..4 {
                        let cp_point_arr = cp_row_arr[j].as_array().context(error_info)?;
                        if cp_point_arr.len() != 3 {
                            bail!(error_info);
                        }
                        control_points[i][j] = Point3::new(
                            cp_point_arr[0].as_f64().context(error_info)? as f32,
                            cp_point_arr[1].as_f64().context(error_info)? as f32,
                            cp_point_arr[2].as_f64().context(error_info)? as f32,
                        );
                    }
                }
                Ok(vec![Box::new(CubicBezier::new(
                    control_points,
                    self.materials[material].clone(),
                ))])
            }
            _ => bail!(format!("object: unknown type '{}'", ty)),
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
                bail!(error_info.clone());
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

    fn load_lights(&self, value: &serde_json::Value) -> Result<Vec<Arc<dyn Light>>> {
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
                    Arc::new(PointLight::new(
                        Point3::new(position[0], position[1], position[2]),
                        strength.into(),
                    )) as Arc<dyn Light>
                }
                "directional" => {
                    let direction =
                        get_float_array3_field(light_json, "light-directional", "direction")?;
                    let strength =
                        get_float_array3_field(light_json, "light-directional", "strength")?;
                    Arc::new(DirLight::new(
                        Vector3::new(direction[0], direction[1], direction[2]),
                        strength.into(),
                    )) as Arc<dyn Light>
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
                    Arc::new(RectangleLight::new(
                        Point3::new(center[0], center[1], center[2]),
                        Vector3::new(direction[0], direction[1], direction[2]),
                        width,
                        height,
                        Vector3::new(up[0], up[1], up[2]),
                        strength.into(),
                    )) as Arc<dyn Light>
                }
                _ => bail!(format!("light: unknown type '{}'", ty)),
            };
            lights.push(light)
        }
        Ok(lights)
    }

    fn load_environment(&self, value: &serde_json::Value) -> Result<Arc<EnvLight>> {
        let ty = get_str_field(value, "environment", "type")?;
        let env = match ty {
            "color" => {
                let color: Color = get_float_array3_field(value, "environment", "color")?.into();
                let scale: Color = get_float_array3_field_or_default(
                    value,
                    "environment",
                    "scale",
                    [1.0, 1.0, 1.0],
                )?
                .into();
                Arc::new(EnvLight::new(vec![vec![color]], scale))
            }
            "texture" => {
                let path = self
                    .path
                    .with_file_name(get_str_field(value, "environment", "file")?);
                let path_str = path.to_str().unwrap();
                let image = get_exr_image(&path).context(format!(
                    "environment: 'file', can't find image '{}'",
                    path_str
                ))?;
                let scale: Color = get_float_array3_field_or_default(
                    value,
                    "environment",
                    "scale",
                    [1.0, 1.0, 1.0],
                )?
                .into();
                Arc::new(EnvLight::new(image, scale))
            }
            _ => bail!(format!("environment: unknown type '{}'", ty)),
        };
        Ok(env)
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
            _ => bail!(format!("aggregate: unknown type '{}'", ty)),
        }
    }

    fn get_texture_float(&self, ind: usize) -> Arc<dyn Texture<f32>> {
        self.textures_float[self.textures_indices[ind]].clone()
    }

    fn get_texture_color(&self, ind: usize) -> Arc<dyn Texture<Color>> {
        self.textures_color[self.textures_indices[ind]].clone()
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

fn get_float_array2_field_or_default(
    value: &serde_json::Value,
    env: &str,
    field: &str,
    default: [f32; 2],
) -> Result<[f32; 2]> {
    if let Some(_) = value.get(field) {
        get_float_array2_field(value, env, field)
    } else {
        Ok(default)
    }
}

fn get_float_array2_field(value: &serde_json::Value, env: &str, field: &str) -> Result<[f32; 2]> {
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
        bail!(error_info)
    }
}

fn get_float_array3_field_or_default(
    value: &serde_json::Value,
    env: &str,
    field: &str,
    default: [f32; 3],
) -> Result<[f32; 3]> {
    if let Some(_) = value.get(field) {
        get_float_array3_field(value, env, field)
    } else {
        Ok(default)
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
        bail!(error_info)
    }
}

fn get_sampler_type(value: &serde_json::Value) -> Result<&'static str> {
    let ty = get_str_field(value, "sample", "type")?;
    match ty {
        "random" => Ok("random"),
        "jittered" => Ok("jittered"),
        _ => bail!(format!("sampler: unknown type '{}'", ty)),
    }
}

fn get_image_field(
    value: &serde_json::Value,
    env: &str,
    field: &str,
    dir: &PathBuf,
) -> Result<image::DynamicImage> {
    let path = dir.with_file_name(get_str_field(value, env, field)?);
    let path_str = path.to_str().unwrap();
    image::open(path_str).context(format!(
        "{}: '{}', can't find image '{}'",
        env, field, path_str
    ))
}

fn get_exr_image(path: &PathBuf) -> Result<Vec<Vec<Color>>> {
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
