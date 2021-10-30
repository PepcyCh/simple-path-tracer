use std::{collections::HashMap, sync::Arc};

use anyhow::Context;

use crate::{
    core::{
        bbox::Bbox,
        camera::Camera,
        color::Color,
        intersection::Intersection,
        light::Light,
        material::Material,
        medium::Medium,
        primitive::{Aggregate, Primitive},
        ray::Ray,
        sampler::Sampler,
        surface::Surface,
        texture::Texture,
        transform::Transform,
    },
    light::{EnvLight, ShapeLight},
    loader::{self, JsonObject, Loadable},
};

pub struct Instance {
    primitive: Arc<dyn Primitive>,
    trans: Transform,
    trans_inv: glam::Affine3A,
    bbox: Bbox,
    surface: Arc<Surface>,
}

#[derive(Default)]
pub struct Scene {
    pub camera: Option<Arc<dyn Camera>>,
    pub aggregate: Option<Arc<dyn Aggregate>>,
    pub instances: HashMap<String, Arc<Instance>>,
    pub primitives: HashMap<String, Arc<dyn Primitive>>,
    pub surfaces: HashMap<String, Arc<Surface>>,
    pub materials: HashMap<String, Arc<dyn Material>>,
    pub mediums: HashMap<String, Arc<dyn Medium>>,
    pub textures_color: HashMap<String, Arc<dyn Texture<Color>>>,
    pub textures_f32: HashMap<String, Arc<dyn Texture<f32>>>,
    pub lights: Vec<Arc<dyn Light>>,
    pub environment: Option<Arc<EnvLight>>,
}

impl Instance {
    pub fn new(
        primitive: Arc<dyn Primitive>,
        trans: glam::Affine3A,
        surface: Arc<Surface>,
    ) -> Self {
        let trans_inv = trans.inverse();
        let bbox = primitive.bbox().transformed_by(trans);
        Self {
            primitive,
            trans: Transform::new(trans),
            trans_inv,
            bbox,
            surface,
        }
    }

    pub fn surface(&self) -> &Arc<Surface> {
        &self.surface
    }
}

impl Scene {
    pub fn aggregate(&self) -> &dyn Aggregate {
        self.aggregate
            .as_ref()
            .map(|aggregate| aggregate.as_ref())
            .unwrap()
    }

    pub fn camera(&self) -> &dyn Camera {
        self.camera.as_ref().map(|camera| camera.as_ref()).unwrap()
    }

    pub fn collect_shape_lights(&mut self) {
        for instance in self.instances.values() {
            if instance.surface().is_emissive() {
                let light = ShapeLight::new(instance.clone());
                self.lights.push(Arc::new(light));
            }
        }
    }
}

impl Primitive for Instance {
    fn intersect_test(&self, ray: &Ray, t_max: f32) -> bool {
        let transformed_ray = ray.transformed_by(self.trans_inv);
        self.primitive.intersect_test(&transformed_ray, t_max)
    }

    fn intersect<'a>(&'a self, ray: &Ray, inter: &mut Intersection<'a>) -> bool {
        let transformed_ray = ray.transformed_by(self.trans_inv);
        if self.primitive.intersect(&transformed_ray, inter) {
            inter.surface = Some(self.surface.as_ref());
            inter.position = ray.point_at(inter.t);

            inter.normal = self.trans.transform_normal3a(inter.normal);
            inter.tangent = self.trans.transform_vector3a(inter.tangent);
            inter.bitangent = self.trans.transform_vector3a(inter.bitangent);
            true
        } else {
            false
        }
    }

    fn bbox(&self) -> Bbox {
        self.bbox
    }

    fn sample<'a>(
        &'a self,
        trans: Transform,
        sampler: &mut dyn Sampler,
    ) -> (Intersection<'a>, f32) {
        let (mut inter, pdf) = self.primitive.sample(trans * self.trans, sampler);
        inter.surface = Some(self.surface.as_ref());
        (inter, pdf)
    }

    fn pdf(&self, trans: Transform, inter: &Intersection<'_>) -> f32 {
        self.primitive.pdf(trans * self.trans, inter)
    }
}

impl Loadable for Instance {
    fn load(
        scene: &mut Scene,
        _path: &std::path::PathBuf,
        json_value: &JsonObject,
    ) -> anyhow::Result<()> {
        let name = loader::get_str_field(json_value, "instance", "name")?;
        let env = format!("instance({})", name);
        if scene.instances.contains_key(name) {
            anyhow::bail!(format!("{}: name is duplicated", env));
        }

        let mut trans = glam::Affine3A::IDENTITY;
        if let Some(mat_value) = json_value.get("matrix") {
            let error_info = format!("{}: 'matrix' should be an array with 16 floats", env);

            let mat_arr = mat_value.as_array().context(error_info.clone())?;
            if mat_arr.len() != 16 {
                anyhow::bail!(error_info.clone());
            }
            let mut matrix = glam::Mat4::IDENTITY;
            matrix.col_mut(0).x = mat_arr[0].as_f64().context(error_info.clone())? as f32;
            matrix.col_mut(0).y = mat_arr[1].as_f64().context(error_info.clone())? as f32;
            matrix.col_mut(0).z = mat_arr[2].as_f64().context(error_info.clone())? as f32;
            matrix.col_mut(0).w = mat_arr[3].as_f64().context(error_info.clone())? as f32;
            matrix.col_mut(1).x = mat_arr[4].as_f64().context(error_info.clone())? as f32;
            matrix.col_mut(1).y = mat_arr[5].as_f64().context(error_info.clone())? as f32;
            matrix.col_mut(1).z = mat_arr[6].as_f64().context(error_info.clone())? as f32;
            matrix.col_mut(1).w = mat_arr[7].as_f64().context(error_info.clone())? as f32;
            matrix.col_mut(2).x = mat_arr[8].as_f64().context(error_info.clone())? as f32;
            matrix.col_mut(2).y = mat_arr[9].as_f64().context(error_info.clone())? as f32;
            matrix.col_mut(2).z = mat_arr[10].as_f64().context(error_info.clone())? as f32;
            matrix.col_mut(2).w = mat_arr[11].as_f64().context(error_info.clone())? as f32;
            matrix.col_mut(3).x = mat_arr[12].as_f64().context(error_info.clone())? as f32;
            matrix.col_mut(3).y = mat_arr[13].as_f64().context(error_info.clone())? as f32;
            matrix.col_mut(3).z = mat_arr[14].as_f64().context(error_info.clone())? as f32;
            matrix.col_mut(3).w = mat_arr[15].as_f64().context(error_info.clone())? as f32;
            trans = glam::Affine3A::from_mat4(matrix);
        }
        if let Some(_) = json_value.get("scale") {
            let scale = loader::get_float_array3_field(json_value, &env, "scale")?;
            trans =
                glam::Affine3A::from_scale(glam::Vec3::new(scale[0], scale[1], scale[2])) * trans;
        }
        if let Some(_) = json_value.get("rotate") {
            let rotate = loader::get_float_array3_field(json_value, &env, "rotate")?;
            trans = glam::Affine3A::from_rotation_z(rotate[2] * std::f32::consts::PI / 180.0)
                * glam::Affine3A::from_rotation_x(rotate[0] * std::f32::consts::PI / 180.0)
                * glam::Affine3A::from_rotation_y(rotate[1] * std::f32::consts::PI / 180.0)
                * trans;
        }
        if let Some(_) = json_value.get("translate") {
            let translate = loader::get_float_array3_field(json_value, &env, "translate")?;
            trans = glam::Affine3A::from_translation(glam::Vec3::new(
                translate[0],
                translate[1],
                translate[2],
            )) * trans;
        }
        if trans.matrix3.determinant() == 0.0 {
            anyhow::bail!(format!("{}: matrix is singular", env))
        }

        let surface = if json_value.contains_key("surface") {
            let surf_name = loader::get_str_field(json_value, &env, "surface")?;
            if let Some(surf) = scene.surfaces.get(surf_name) {
                surf.clone()
            } else {
                anyhow::bail!(format!("{}: surface '{}' not found", env, surf_name))
            }
        } else if json_value.contains_key("material") {
            let mat_name = loader::get_str_field(json_value, &env, "material")?;
            if let Some(mat) = scene.materials.get(mat_name) {
                let surf_name = format!("${}", mat_name);
                scene
                    .surfaces
                    .entry(surf_name)
                    .or_insert(Arc::new(Surface::new(
                        mat.clone(),
                        None,
                        None,
                        Color::BLACK,
                        None,
                        false,
                        None,
                    )))
                    .clone()
            } else {
                anyhow::bail!(format!("{}: material '{}' not found", env, mat_name))
            }
        } else {
            anyhow::bail!(format!(
                "{}: neither 'surface' nor 'material' is found",
                env
            ))
        };

        let prim_name = loader::get_str_field(json_value, &env, "primitive")?;
        let primitive = if let Some(prim) = scene.primitives.get(prim_name) {
            prim.clone()
        } else {
            anyhow::bail!(format!("{}: primitive '{}' not found", env, prim_name))
        };

        scene.instances.insert(
            name.to_owned(),
            Arc::new(Self::new(primitive, trans, surface)),
        );

        Ok(())
    }
}
