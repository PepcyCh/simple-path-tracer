use crate::core::ray::Ray;
use cgmath::{EuclideanSpace, Transform};

#[derive(Copy, Clone, Debug)]
pub struct Bbox {
    pub p_min: cgmath::Point3<f32>,
    pub p_max: cgmath::Point3<f32>,
}

impl Bbox {
    pub fn new(p_min: cgmath::Point3<f32>, p_max: cgmath::Point3<f32>) -> Self {
        Self { p_min, p_max }
    }

    pub fn from_points(points: &[cgmath::Point3<f32>]) -> Self {
        let mut p_min = points[0];
        let mut p_max = points[0];
        points.iter().skip(1).for_each(|p| {
            p_min = min_point3(p_min, *p);
            p_max = max_point3(p_max, *p);
        });
        Self { p_min, p_max }
    }

    pub fn empty() -> Self {
        Self {
            p_min: cgmath::Point3::new(f32::MAX, f32::MAX, f32::MAX),
            p_max: cgmath::Point3::new(f32::MIN, f32::MIN, f32::MIN),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.p_min.x > self.p_max.x || self.p_min.y > self.p_max.y || self.p_min.z > self.p_max.z
    }

    pub fn merge(mut self, another: Bbox) -> Self {
        self.p_min = min_point3(self.p_min, another.p_min);
        self.p_max = max_point3(self.p_max, another.p_max);
        self
    }

    pub fn transformed_by(mut self, trans: cgmath::Matrix4<f32>) -> Self {
        let p0 = trans.transform_point(cgmath::Point3::new(
            self.p_min.x,
            self.p_min.y,
            self.p_min.z,
        ));
        let p1 = trans.transform_point(cgmath::Point3::new(
            self.p_min.x,
            self.p_min.y,
            self.p_max.z,
        ));
        let p2 = trans.transform_point(cgmath::Point3::new(
            self.p_min.x,
            self.p_max.y,
            self.p_min.z,
        ));
        let p3 = trans.transform_point(cgmath::Point3::new(
            self.p_min.x,
            self.p_max.y,
            self.p_max.z,
        ));
        let p4 = trans.transform_point(cgmath::Point3::new(
            self.p_max.x,
            self.p_min.y,
            self.p_min.z,
        ));
        let p5 = trans.transform_point(cgmath::Point3::new(
            self.p_max.x,
            self.p_min.y,
            self.p_max.z,
        ));
        let p6 = trans.transform_point(cgmath::Point3::new(
            self.p_max.x,
            self.p_max.y,
            self.p_min.z,
        ));
        let p7 = trans.transform_point(cgmath::Point3::new(
            self.p_max.x,
            self.p_max.y,
            self.p_max.z,
        ));
        self.p_min = min_point3(
            min_point3(min_point3(p0, p1), min_point3(p2, p3)),
            min_point3(min_point3(p4, p5), min_point3(p6, p7)),
        );
        self.p_max = max_point3(
            max_point3(max_point3(p0, p1), max_point3(p2, p3)),
            max_point3(max_point3(p4, p5), max_point3(p6, p7)),
        );
        self
    }

    pub fn intersect_test(&self, ray: &Ray, t_max: f32) -> bool {
        if self.is_empty() {
            return false;
        }

        let x0 = (self.p_min.x - ray.origin.x) / ray.direction.x;
        let x1 = (self.p_max.x - ray.origin.x) / ray.direction.x;
        let (x0, x1) = (x0.min(x1), x0.max(x1));
        let y0 = (self.p_min.y - ray.origin.y) / ray.direction.y;
        let y1 = (self.p_max.y - ray.origin.y) / ray.direction.y;
        let (y0, y1) = (y0.min(y1), y0.max(y1));
        let z0 = (self.p_min.z - ray.origin.z) / ray.direction.z;
        let z1 = (self.p_max.z - ray.origin.z) / ray.direction.z;
        let (z0, z1) = (z0.min(z1), z0.max(z1));
        let t0 = x0.max(y0.max(z0));
        let t1 = x1.min(y1.min(z1));
        t0 <= t1 && t1 > ray.t_min && t0 < t_max
    }

    pub fn surface_area(&self) -> f32 {
        if self.is_empty() {
            0.0
        } else {
            let diff = self.p_max - self.p_min;
            diff.x * diff.y * diff.z
        }
    }

    pub fn centroid(&self) -> cgmath::Point3<f32> {
        self.p_min.midpoint(self.p_max)
    }
}

fn min_point3(a: cgmath::Point3<f32>, b: cgmath::Point3<f32>) -> cgmath::Point3<f32> {
    cgmath::Point3::new(a.x.min(b.x), a.y.min(b.y), a.z.min(b.z))
}

fn max_point3(a: cgmath::Point3<f32>, b: cgmath::Point3<f32>) -> cgmath::Point3<f32> {
    cgmath::Point3::new(a.x.max(b.x), a.y.max(b.y), a.z.max(b.z))
}
