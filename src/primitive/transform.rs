use crate::core::bbox::Bbox;
use crate::core::intersection::Intersection;
use crate::core::material::Material;
use crate::core::medium::Medium;
use crate::core::primitive::Primitive;
use crate::core::ray::Ray;
use cgmath::{InnerSpace, Matrix, SquareMatrix};

pub struct Transform {
    primitive: Box<dyn Primitive>,
    _trans: cgmath::Matrix4<f32>,
    trans_inv: cgmath::Matrix4<f32>,
    trans_it: cgmath::Matrix4<f32>,
    bbox: Bbox,
}

impl Transform {
    pub fn new(primitive: Box<dyn Primitive>, trans: cgmath::Matrix4<f32>) -> Self {
        let trans_inv = trans.invert().unwrap();
        let trans_it = trans_inv.transpose();
        let bbox = primitive.bbox().transformed_by(trans);
        Self {
            primitive,
            _trans: trans,
            trans_inv,
            trans_it,
            bbox,
        }
    }
}

impl Primitive for Transform {
    fn intersect_test(&self, ray: &Ray, t_max: f32) -> bool {
        let transformed_ray = ray.transformed_by(self.trans_inv);
        self.primitive.intersect_test(&transformed_ray, t_max)
    }

    fn intersect<'a>(&'a self, ray: &Ray, inter: &mut Intersection<'a>) -> bool {
        let transformed_ray = ray.transformed_by(self.trans_inv);
        if self.primitive.intersect(&transformed_ray, inter) {
            inter.normal =
                cgmath::Transform::transform_vector(&self.trans_it, inter.normal).normalize();
            true
        } else {
            false
        }
    }

    fn bbox(&self) -> Bbox {
        self.bbox
    }

    fn material(&self) -> Option<&dyn Material> {
        self.primitive.material()
    }

    fn inside_medium(&self) -> Option<&dyn Medium> {
        self.primitive.inside_medium()
    }
}
