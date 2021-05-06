use cgmath::Transform;

#[derive(Debug, Copy, Clone)]
pub struct Ray {
    pub origin: cgmath::Point3<f32>,
    pub direction: cgmath::Vector3<f32>,
    pub t_min: f32,
    pub aux_ray: Option<AuxiliaryRay>,
}

#[derive(Debug, Copy, Clone)]
pub struct AuxiliaryRay {
    pub x_origin: cgmath::Point3<f32>,
    pub x_direction: cgmath::Vector3<f32>,
    pub y_origin: cgmath::Point3<f32>,
    pub y_direction: cgmath::Vector3<f32>,
}

impl Ray {
    pub const T_MIN_EPS: f32 = 0.0001;

    pub fn new(origin: cgmath::Point3<f32>, direction: cgmath::Vector3<f32>) -> Self {
        Self {
            origin,
            direction,
            t_min: Self::T_MIN_EPS,
            aux_ray: None,
        }
    }

    pub fn point_at(&self, t: f32) -> cgmath::Point3<f32> {
        self.origin + self.direction * t
    }

    pub fn transformed_by(self, trans: cgmath::Matrix4<f32>) -> Self {
        let origin = trans.transform_point(self.origin);
        let direction = trans.transform_vector(self.direction);
        Self {
            origin,
            direction,
            ..self
        }
    }
}

impl AuxiliaryRay {
    pub fn from_rays(ray_x: Ray, ray_y: Ray) -> Self {
        Self {
            x_origin: ray_x.origin,
            x_direction: ray_x.direction,
            y_origin: ray_y.origin,
            y_direction: ray_y.direction,
        }
    }

    pub fn point_x_at(&self, t: f32) -> cgmath::Point3<f32> {
        self.x_origin + self.x_direction * t
    }

    pub fn point_y_at(&self, t: f32) -> cgmath::Point3<f32> {
        self.y_origin + self.y_direction * t
    }
}
