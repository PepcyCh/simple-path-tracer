use crate::core::{
    bbox::Bbox, intersection::Intersection, loader::InputParams, ray::Ray, rng::Rng, scene::Scene,
};

use super::{BasicPrimitiveRef, PrimitiveT};

pub struct Sphere {
    center: glam::Vec3A,
    radius: f32,
    bbox: Bbox,
}

impl Sphere {
    pub fn new(center: glam::Vec3A, radius: f32) -> Self {
        let delta = glam::Vec3A::new(radius, radius, radius);
        let bbox = Bbox::new(center - delta, center + delta);
        Self {
            center,
            radius,
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

    pub fn load(_scene: &Scene, params: &mut InputParams) -> anyhow::Result<Self> {
        let center = params.get_float3_or("center", [0.0, 0.0, 0.0]);

        let radius = params.get_float("radius")?;

        Ok(Sphere::new(center.into(), radius))
    }
}

impl PrimitiveT for Sphere {
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
                inter.primitive = Some(BasicPrimitiveRef::Sphere(self));
                return true;
            }
        }
        false
    }

    fn bbox(&self) -> Bbox {
        self.bbox
    }

    fn sample<'a>(&'a self, sampler: &mut Rng) -> (Intersection<'a>, f32) {
        let norm = sampler.uniform_on_sphere();
        let pos = self.center + norm * self.radius;

        let mut inter = Intersection {
            position: pos,
            normal: norm,
            texcoords: sphere_normal_to_texcoords(norm),
            primitive: Some(BasicPrimitiveRef::Sphere(self)),
            ..Default::default()
        };

        let sin_theta = (1.0 - norm.y * norm.y).sqrt();
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

        (inter, 0.25 * std::f32::consts::FRAC_1_PI)
    }

    fn pdf(&self, _inter: &Intersection<'_>) -> f32 {
        0.25 * std::f32::consts::FRAC_1_PI
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
