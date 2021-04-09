use cgmath::Transform;

#[derive(Debug, Copy, Clone)]
pub struct Ray {
    pub origin: cgmath::Point3<f32>,
    pub direction: cgmath::Vector3<f32>,
    pub t_min: f32,
}

impl Ray {
    pub const T_MIN_EPS: f32 = 0.0001;

    pub fn new(origin: cgmath::Point3<f32>, direction: cgmath::Vector3<f32>) -> Self {
        Self {
            origin,
            direction,
            t_min: Self::T_MIN_EPS,
        }
    }

    pub fn point_at(&self, t: f32) -> cgmath::Point3<f32> {
        self.origin + self.direction * t
    }

    pub fn transformed_by(&self, trans: cgmath::Matrix4<f32>) -> Self {
        let origin = trans.transform_point(self.origin);
        let direction = trans.transform_vector(self.direction);
        Self {
            origin,
            direction,
            t_min: self.t_min,
        }
    }
}
