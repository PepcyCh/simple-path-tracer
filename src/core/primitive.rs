use crate::core::bbox::Bbox;
use crate::core::intersection::Intersection;
use crate::core::material::Material;
use crate::core::ray::Ray;

pub trait Primitive {
    fn intersect_test(&self, ray: &Ray, t_max: f32) -> bool;

    fn intersect<'a>(&'a self, ray: &Ray, inter: &mut Intersection<'a>) -> bool;

    fn bbox(&self) -> Bbox;

    fn material(&self) -> Option<&dyn Material>;
}

pub trait Aggregate: Primitive {}
