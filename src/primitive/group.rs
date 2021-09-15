use crate::core::{
    bbox::Bbox,
    intersection::Intersection,
    material::Material,
    medium::Medium,
    primitive::{Aggregate, Primitive},
    ray::Ray,
};

pub struct Group {
    primitives: Vec<Box<dyn Primitive>>,
    bbox: Bbox,
}

impl Group {
    pub fn new(primitives: Vec<Box<dyn Primitive>>) -> Self {
        let bbox = primitives
            .iter()
            .map(|prim| prim.bbox())
            .fold(Bbox::empty(), |acc, curr_box| acc.merge(curr_box));
        Self { primitives, bbox }
    }
}

impl Primitive for Group {
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

    fn material(&self) -> Option<&dyn Material> {
        None
    }

    fn inside_medium(&self) -> Option<&dyn Medium> {
        None
    }
}

impl Aggregate for Group {}
