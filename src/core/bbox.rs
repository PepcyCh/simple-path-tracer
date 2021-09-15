use crate::core::ray::Ray;

#[derive(Copy, Clone, Debug)]
pub struct Bbox {
    pub p_min: glam::Vec3A,
    pub p_max: glam::Vec3A,
}

impl Bbox {
    pub fn new(p_min: glam::Vec3A, p_max: glam::Vec3A) -> Self {
        Self { p_min, p_max }
    }

    pub fn from_points(points: &[glam::Vec3A]) -> Self {
        let mut p_min = points[0];
        let mut p_max = points[0];
        points.iter().skip(1).for_each(|p| {
            p_min = p_min.min(*p);
            p_max = p_max.max(*p);
        });
        Self { p_min, p_max }
    }

    pub fn empty() -> Self {
        Self {
            p_min: glam::Vec3A::new(f32::MAX, f32::MAX, f32::MAX),
            p_max: glam::Vec3A::new(f32::MIN, f32::MIN, f32::MIN),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.p_min.x > self.p_max.x || self.p_min.y > self.p_max.y || self.p_min.z > self.p_max.z
    }

    pub fn merge(mut self, another: Bbox) -> Self {
        self.p_min = self.p_min.min(another.p_min);
        self.p_max = self.p_max.max(another.p_max);
        self
    }

    pub fn transformed_by(mut self, trans: glam::Affine3A) -> Self {
        let p0 =
            trans.transform_point3a(glam::Vec3A::new(self.p_min.x, self.p_min.y, self.p_min.z));
        let p1 =
            trans.transform_point3a(glam::Vec3A::new(self.p_min.x, self.p_min.y, self.p_max.z));
        let p2 =
            trans.transform_point3a(glam::Vec3A::new(self.p_min.x, self.p_max.y, self.p_min.z));
        let p3 =
            trans.transform_point3a(glam::Vec3A::new(self.p_min.x, self.p_max.y, self.p_max.z));
        let p4 =
            trans.transform_point3a(glam::Vec3A::new(self.p_max.x, self.p_min.y, self.p_min.z));
        let p5 =
            trans.transform_point3a(glam::Vec3A::new(self.p_max.x, self.p_min.y, self.p_max.z));
        let p6 =
            trans.transform_point3a(glam::Vec3A::new(self.p_max.x, self.p_max.y, self.p_min.z));
        let p7 =
            trans.transform_point3a(glam::Vec3A::new(self.p_max.x, self.p_max.y, self.p_max.z));
        self.p_min = p0.min(p1).min(p2).min(p3).min(p4).min(p5).min(p6).min(p7);
        self.p_max = p0.max(p1).max(p2).max(p3).max(p4).max(p5).max(p6).max(p7);
        self
    }

    pub fn intersect_ray(&self, ray: &Ray) -> Option<(f32, f32)> {
        if self.is_empty() {
            return None;
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

        if t0 <= t1 {
            Some((t0, t1))
        } else {
            None
        }
    }

    pub fn intersect_test(&self, ray: &Ray, t_max: f32) -> bool {
        if let Some((t0, t1)) = self.intersect_ray(ray) {
            t1 > ray.t_min && t0 < t_max
        } else {
            false
        }
    }

    pub fn surface_area(&self) -> f32 {
        if self.is_empty() {
            0.0
        } else {
            let diff = self.p_max - self.p_min;
            diff.x * diff.y * diff.z
        }
    }

    pub fn centroid(&self) -> glam::Vec3A {
        (self.p_min + self.p_max) * 0.5
    }
}
