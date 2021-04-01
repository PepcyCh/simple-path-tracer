use crate::core::bbox::Bbox;
use crate::core::intersection::Intersection;
use crate::core::material::Material;
use crate::core::primitive::Primitive;
use crate::core::ray::Ray;
use cgmath::InnerSpace;
use std::rc::Rc;

pub struct Sphere {
    center: cgmath::Point3<f32>,
    radius: f32,
    material: Rc<dyn Material>,
    bbox: Bbox,
}

impl Sphere {
    pub fn new(center: cgmath::Point3<f32>, radius: f32, material: Rc<dyn Material>) -> Self {
        let delta = cgmath::Vector3::new(radius, radius, radius);
        let bbox = Bbox::new(center - delta, center + delta);
        Self {
            center,
            radius,
            material,
            bbox,
        }
    }

    fn intersect_ray(&self, ray: &Ray) -> Option<(f32, f32)> {
        let oc = ray.origin - self.center;
        let a = ray.direction.magnitude2();
        let b = ray.direction.dot(oc);
        let c = oc.magnitude2() - self.radius * self.radius;
        let delta = b * b - a * c;
        if delta >= 0.0 {
            let delta = delta.sqrt();
            let min = (-b - delta) / a;
            let max = (-b + delta) / a;
            Some((min, max))
        } else {
            None
        }
    }
}

impl Primitive for Sphere {
    fn intersect_test(&self, ray: &Ray, t_max: f32) -> bool {
        if let Some((min, max)) = self.intersect_ray(ray) {
            min < t_max && max > ray.t_min
        } else {
            false
        }
    }

    fn intersect<'a>(&'a self, ray: &Ray, inter: &mut Intersection<'a>) -> bool {
        if let Some((min, max)) = self.intersect_ray(ray) {
            let t = if min < ray.t_min { max } else { min };
            if ray.t_min < t && t < inter.t {
                inter.t = t;
                let norm = (ray.point_at(t) - self.center) / self.radius;
                inter.normal = norm;
                if inter.normal.dot(ray.direction) > 0.0 {
                    inter.normal = -inter.normal;
                }
                inter.texcoords = sphere_normal_to_texcoords(norm);
                inter.primitive = Some(self);
                return true;
            }
        }
        false
    }

    fn bbox(&self) -> Bbox {
        self.bbox
    }

    fn material(&self) -> Option<&dyn Material> {
        Some(&*self.material)
    }
}

fn sphere_normal_to_texcoords(p: cgmath::Vector3<f32>) -> cgmath::Point2<f32> {
    let theta = p.y.acos();
    let sin_theta = (1.0 - p.y * p.y).sqrt();
    let phi = (p.z / sin_theta).acos();
    cgmath::Point2::new(
        phi / (2.0 * std::f32::consts::PI),
        theta / std::f32::consts::PI,
    )
}
