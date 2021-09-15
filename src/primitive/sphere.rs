use std::sync::Arc;

use crate::core::{
    bbox::Bbox, intersection::Intersection, material::Material, medium::Medium,
    primitive::Primitive, ray::Ray,
};

pub struct Sphere {
    center: glam::Vec3A,
    radius: f32,
    material: Arc<dyn Material>,
    inside_medium: Option<Arc<dyn Medium>>,
    bbox: Bbox,
}

impl Sphere {
    pub fn new(
        center: glam::Vec3A,
        radius: f32,
        material: Arc<dyn Material>,
        inside_medium: Option<Arc<dyn Medium>>,
    ) -> Self {
        let delta = glam::Vec3A::new(radius, radius, radius);
        let bbox = Bbox::new(center - delta, center + delta);
        Self {
            center,
            radius,
            material,
            inside_medium,
            bbox,
        }
    }

    fn intersect_ray(&self, ray: &Ray) -> Option<(f32, f32)> {
        let oc = ray.origin - self.center;
        let a = ray.direction.length_squared();
        let b = ray.direction.dot(oc);
        let c = oc.length_squared() - self.radius * self.radius;
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
                let sin_theta = (1.0 - norm.y * norm.y).sqrt();
                inter.normal = norm;
                if sin_theta != 0.0 {
                    inter.bitangent = norm * (-norm.y / sin_theta);
                    inter.bitangent.y = sin_theta;
                    inter.tangent = inter.bitangent.cross(inter.normal);
                } else if norm.y > 0.0 {
                    inter.bitangent = glam::Vec3A::X;
                    inter.tangent = glam::Vec3A::Z;
                } else {
                    inter.bitangent = -glam::Vec3A::X;
                    inter.tangent = -glam::Vec3A::Z;
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

    fn inside_medium(&self) -> Option<&dyn Medium> {
        self.inside_medium.as_ref().map(|rc| rc.as_ref())
    }
}

fn sphere_normal_to_texcoords(p: glam::Vec3A) -> glam::Vec2 {
    let theta = p.y.acos();
    let phi = p.x.atan2(p.z) + std::f32::consts::PI;
    glam::Vec2::new(
        phi * 0.5 * std::f32::consts::FRAC_1_PI,
        theta * std::f32::consts::FRAC_1_PI,
    )
}
