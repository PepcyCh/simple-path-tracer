use crate::core::{
    bbox::Bbox, intersection::Intersection, material::Material, medium::Medium,
    primitive::Primitive, ray::Ray,
};

pub struct Transform {
    primitive: Box<dyn Primitive>,
    trans: glam::Affine3A,
    trans_inv: glam::Affine3A,
    trans_it: glam::Mat3A,
    bbox: Bbox,
}

impl Transform {
    pub fn new(primitive: Box<dyn Primitive>, trans: glam::Affine3A) -> Self {
        let trans_inv = trans.inverse();
        let trans_it = trans_inv.matrix3.transpose();
        let bbox = primitive.bbox().transformed_by(trans);
        Self {
            primitive,
            trans,
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
            inter.normal = (self.trans_it * inter.normal).normalize();
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

    fn material(&self) -> Option<&dyn Material> {
        self.primitive.material()
    }

    fn inside_medium(&self) -> Option<&dyn Medium> {
        self.primitive.inside_medium()
    }
}
