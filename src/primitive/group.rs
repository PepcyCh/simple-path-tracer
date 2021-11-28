use std::sync::Arc;

use crate::core::{bbox::Bbox, intersection::Intersection, ray::Ray, rng::Rng};

use super::PrimitiveT;

pub struct Group<P: PrimitiveT> {
    primitives: Vec<Arc<P>>,
    bbox: Bbox,
}

impl<P: PrimitiveT> Group<P> {
    pub fn new(primitives: Vec<Arc<P>>) -> Self {
        let bbox = primitives
            .iter()
            .map(|prim| prim.bbox())
            .fold(Bbox::empty(), |acc, curr_box| acc.merge(curr_box));
        Self { primitives, bbox }
    }
}

impl<P: PrimitiveT> PrimitiveT for Group<P> {
    fn intersect_test(&self, ray: &Ray, t_max: f32) -> bool {
        for prim in &self.primitives {
            if prim.intersect_test(ray, t_max) {
                return true;
            }
        }
        false
    }

    fn intersect<'a>(&'a self, ray: &Ray, inter: &mut Intersection<'a>) -> bool {
        let mut result = false;
        for prim in &self.primitives {
            result |= prim.intersect(ray, inter);
        }
        result
    }

    fn bbox(&self) -> Bbox {
        self.bbox
    }

    fn sample<'a>(&'a self, sampler: &mut Rng) -> (Intersection<'a>, f32) {
        let index = sampler.uniform_1d() * self.primitives.len() as f32;
        let index = (index as usize).min(self.primitives.len() - 1);
        let (inter, pdf) = self.primitives[index].sample(sampler);
        (inter, pdf / self.primitives.len() as f32)
    }

    fn pdf(&self, inter: &Intersection<'_>) -> f32 {
        inter.primitive.unwrap().pdf(inter) / self.primitives.len() as f32
    }
}
