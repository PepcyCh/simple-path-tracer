use crate::core::{bbox::Bbox, intersection::Intersection, ray::Ray, sampler::Sampler};

pub trait Primitive: Send + Sync {
    fn intersect_test(&self, ray: &Ray, t_max: f32) -> bool;

    fn intersect<'a>(&'a self, ray: &Ray, inter: &mut Intersection<'a>) -> bool;

    fn bbox(&self) -> Bbox;

    fn sample<'a>(&'a self, sampler: &mut dyn Sampler) -> (Intersection<'a>, f32);

    /// sample pdf relative to area
    fn pdf(&self, inter: &Intersection<'_>) -> f32;
}

pub trait Aggregate: Primitive {}
