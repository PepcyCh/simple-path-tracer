use crate::core::{
    bbox::Bbox, intersection::Intersection, material::Material, medium::Medium, ray::Ray,
};

pub trait Primitive: Send + Sync {
    fn intersect_test(&self, ray: &Ray, t_max: f32) -> bool;

    fn intersect<'a>(&'a self, ray: &Ray, inter: &mut Intersection<'a>) -> bool;

    fn bbox(&self) -> Bbox;

    fn material(&self) -> Option<&dyn Material>;

    fn inside_medium(&self) -> Option<&dyn Medium>;
}

pub trait Aggregate: Primitive {}
