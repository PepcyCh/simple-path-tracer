use crate::core::{bbox::Bbox, intersection::Intersection, ray::Ray, sampler::Sampler, transform::Transform};

pub trait Primitive: Send + Sync {
    fn intersect_test(&self, ray: &Ray, t_max: f32) -> bool;

    fn intersect<'a>(&'a self, ray: &Ray, inter: &mut Intersection<'a>) -> bool;

    fn bbox(&self) -> Bbox;

    fn sample<'a>(&'a self, trans: Transform, sampler: &mut dyn Sampler) -> (Intersection<'a>, f32);

    /// sample pdf relative to area
    fn pdf(&self, trans: Transform, inter: &Intersection<'_>) -> f32;
}

pub trait Aggregate: Primitive {}
