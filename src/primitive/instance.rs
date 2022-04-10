use std::sync::Arc;

use crate::core::{
    bbox::Bbox, color::Color, intersection::Intersection, loader::InputParams, ray::Ray, rng::Rng,
    scene_resources::SceneResources, surface::Surface, transform::Transform,
};

use super::{Primitive, PrimitiveT};

pub struct Instance {
    primitive: Arc<Primitive>,
    trans: Transform,
    trans_inv: glam::Affine3A,
    bbox: Bbox,
    surface: Arc<Surface>,
}

impl Instance {
    pub fn new(primitive: Arc<Primitive>, trans: glam::Affine3A, surface: Arc<Surface>) -> Self {
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

    pub fn load(rsc: &mut SceneResources, params: &mut InputParams) -> anyhow::Result<()> {
        params.set_name("instance".into());
        let name = params.get_str("name")?;
        params.set_name(format!("instance-{}", name).into());

        let mut trans = glam::Affine3A::IDENTITY;
        if params.contains_key("matrix") {
            trans = glam::Affine3A::from_mat4(params.get_matrix("matrix")?);
        }
        if params.contains_key("scale") {
            trans = glam::Affine3A::from_scale(params.get_float3("scale")?.into()) * trans;
        }
        if params.contains_key("rotate") {
            let rotate = params.get_float3("rotate")?;
            trans = glam::Affine3A::from_rotation_z(rotate[2] * std::f32::consts::PI / 180.0)
                * glam::Affine3A::from_rotation_x(rotate[0] * std::f32::consts::PI / 180.0)
                * glam::Affine3A::from_rotation_y(rotate[1] * std::f32::consts::PI / 180.0)
                * trans;
        }
        if params.contains_key("translate") {
            trans =
                glam::Affine3A::from_translation(params.get_float3("translate")?.into()) * trans;
        }
        if trans.matrix3.determinant() == 0.0 {
            log::warn!("{}: transform matrix is singular", params.name());
        }

        let surface = if params.contains_key("surface") {
            rsc.clone_surface(params.get_str("surface")?)?
        } else {
            let material = rsc.clone_material(params.get_str("material")?)?;
            Arc::new(Surface::new(
                material,
                None,
                None,
                Color::BLACK,
                None,
                false,
                None,
            ))
        };

        let primitive = rsc.clone_primitive(params.get_str("primitive")?)?;

        let res = Self::new(primitive, trans, surface);
        rsc.add_instance(name, res)?;

        params.check_unused_keys();

        Ok(())
    }
}

impl PrimitiveT for Instance {
    fn intersect_test(&self, ray: &Ray, t_max: f32) -> bool {
        let transformed_ray = ray.transformed_by(self.trans_inv);
        self.primitive.intersect_test(&transformed_ray, t_max)
    }

    fn intersect<'a>(&'a self, ray: &Ray, inter: &mut Intersection<'a>) -> bool {
        let transformed_ray = ray.transformed_by(self.trans_inv);
        if self.primitive.intersect(&transformed_ray, inter) {
            inter.instance = Some(self);

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

    fn sample<'a>(&'a self, rng: &mut Rng) -> (Intersection<'a>, f32) {
        let (mut inter, pdf) = self.primitive.sample(rng);
        inter.surface = Some(self.surface.as_ref());

        let original_area = inter.tangent.cross(inter.bitangent).length();

        inter.position = self.trans.transform_point3a(inter.position);
        inter.normal = self.trans.transform_normal3a(inter.normal);
        inter.bitangent = self.trans.transform_vector3a(inter.bitangent);
        inter.tangent = self.trans.transform_vector3a(inter.tangent);

        let transformed_area = inter.tangent.cross(inter.bitangent).length();

        (inter, pdf * original_area / transformed_area)
    }

    fn pdf(&self, inter: &Intersection<'_>) -> f32 {
        let trans_inv = self.trans.inverse();

        let tangent = trans_inv.transform_vector3a(inter.tangent);
        let bitangent = trans_inv.transform_vector3a(inter.bitangent);

        let original_area = tangent.cross(bitangent).length();
        let transformed_area = inter.tangent.cross(inter.bitangent).length();

        self.primitive.pdf(inter) * original_area / transformed_area
    }

    fn surface_area(&self, trans: Transform) -> f32 {
        let trans = trans * self.trans;
        self.primitive.surface_area(trans)
    }
}

pub struct InstancePtr(pub *const Instance);

impl std::hash::Hash for InstancePtr {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::ptr::hash(self.0, state);
    }
}
impl PartialEq for InstancePtr {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.0, other.0)
    }
}
impl Eq for InstancePtr {}

unsafe impl Send for InstancePtr {}
unsafe impl Sync for InstancePtr {}
