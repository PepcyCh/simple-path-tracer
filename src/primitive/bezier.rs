use std::sync::Arc;

use cgmath::{ElementWise, InnerSpace, Point3, Vector3};

use crate::core::bbox::Bbox;
use crate::core::intersection::Intersection;
use crate::core::material::Material;
use crate::core::primitive::Primitive;
use crate::core::ray::Ray;

pub struct CubicBezier {
    control_points: [[Point3<f32>; 4]; 4],
    material: Arc<dyn Material>,
    bbox: Bbox,
}

impl CubicBezier {
    pub fn new(control_points: [[Point3<f32>; 4]; 4], material: Arc<dyn Material>) -> Self {
        let (p_min, p_max) = control_points.iter().flatten().fold(
            (control_points[0][0], control_points[0][0]),
            |acc, curr| {
                (
                    Point3::new(
                        acc.0.x.min(curr.x),
                        acc.0.y.min(curr.y),
                        acc.0.z.min(curr.z),
                    ),
                    Point3::new(
                        acc.1.x.max(curr.x),
                        acc.1.y.max(curr.y),
                        acc.1.z.max(curr.z),
                    ),
                )
            },
        );
        let bbox = Bbox::new(p_min, p_max);

        Self {
            control_points,
            material,
            bbox,
        }
    }

    fn point_at(&self, u: f32, v: f32) -> Point3<f32> {
        let bezier_u = cubic_bezier_at(u);
        let bezier_v = cubic_bezier_at(v);
        cubic_bezier_sum(&self.control_points, &bezier_u, &bezier_v)
    }

    fn tangent_at(&self, u: f32, v: f32) -> Vector3<f32> {
        let bezier_du = cubic_bezier_du_at(u);
        let bezier_v = cubic_bezier_at(v);
        point_as_vector(cubic_bezier_sum(
            &self.control_points,
            &bezier_du,
            &bezier_v,
        ))
    }

    fn bitangent_at(&self, u: f32, v: f32) -> Vector3<f32> {
        let bezier_u = cubic_bezier_at(u);
        let bezier_dv = cubic_bezier_du_at(v);
        point_as_vector(cubic_bezier_sum(
            &self.control_points,
            &bezier_u,
            &bezier_dv,
        ))
    }

    fn intersect_ray(&self, ray: &Ray) -> Option<(f32, f32, f32)> {
        // Newton's iteration
        let max_iteration_times = 32;
        if let Some((t0, t1)) = self.bbox.intersect_ray(ray) {
            let mut t = 0.5 * (t0 + t1);
            let mut u = 0.5;
            let mut v = 0.5;

            for _ in 0..max_iteration_times {
                let point = self.point_at(u, v);
                let diff = ray.point_at(t) - point;

                if !t.is_finite() || !u.is_finite() || !v.is_finite() {
                    break;
                }

                if diff.magnitude2() < 0.000000001 {
                    if u >= 0.0 && u <= 1.0 && v >= 0.0 && v <= 1.0 && t > ray.t_min {
                        return Some((u, v, t));
                    }
                    break;
                }

                let dpdu = self.tangent_at(u, v);
                let dpdv = self.bitangent_at(u, v);

                let n = dpdu.cross(dpdv);
                let det = ray.direction.dot(n);
                if det == 0.0 {
                    break;
                }
                let det = 1.0 / det;
                let dt = diff.dot(n) * det;
                let q = ray.direction.cross(diff);
                let du = -dpdv.dot(q) * det;
                let dv = dpdu.dot(q) * det;

                t -= dt;
                u -= du;
                v -= dv;
            }
        }

        None
    }
}

impl Primitive for CubicBezier {
    fn intersect_test(&self, ray: &Ray, t_max: f32) -> bool {
        if let Some((_, _, t)) = self.intersect_ray(ray) {
            t < t_max
        } else {
            false
        }
    }

    fn intersect<'a>(&'a self, ray: &Ray, inter: &mut Intersection<'a>) -> bool {
        if let Some((u, v, t)) = self.intersect_ray(ray) {
            if t < inter.t {
                inter.t = t;
                inter.texcoords = cgmath::Point2::new(u, v);
                inter.tangent = self.tangent_at(u, v);
                inter.bitangent = self.bitangent_at(u, v);
                inter.normal = (inter.tangent.cross(inter.bitangent)).normalize();
                if ray.direction.dot(inter.normal) > 0.0 {
                    inter.normal = inter.normal;
                }
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

    fn inside_medium(&self) -> Option<&dyn crate::core::medium::Medium> {
        None
    }
}

fn cubic_bezier_at(u: f32) -> [f32; 4] {
    let iu = 1.0 - u;
    [iu * iu * iu, 3.0 * iu * iu * u, 3.0 * u * u * iu, u * u * u]
}

fn cubic_bezier_du_at(u: f32) -> [f32; 4] {
    let iu = 1.0 - u;
    [
        -3.0 * iu * iu,
        3.0 * iu * iu - 6.0 * iu * u,
        6.0 * u * iu - 3.0 * u * u,
        3.0 * u * u,
    ]
}

fn cubic_bezier_sum(
    points: &[[Point3<f32>; 4]; 4],
    basic_u: &[f32; 4],
    basic_v: &[f32; 4],
) -> Point3<f32> {
    let mut result = Point3::new(0.0, 0.0, 0.0);
    for i in 0..4 {
        for j in 0..4 {
            result = result.add_element_wise(basic_u[j] * basic_v[i] * points[i][j]);
        }
    }
    result
}

fn point_as_vector(p: Point3<f32>) -> Vector3<f32> {
    Vector3::new(p.x, p.y, p.z)
}
