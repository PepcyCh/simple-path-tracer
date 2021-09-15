#[derive(Debug, Copy, Clone)]
pub struct Ray {
    pub origin: glam::Vec3A,
    pub direction: glam::Vec3A,
    pub t_min: f32,
    pub aux_ray: Option<AuxiliaryRay>,
}

#[derive(Debug, Copy, Clone)]
pub struct AuxiliaryRay {
    pub x_origin: glam::Vec3A,
    pub x_direction: glam::Vec3A,
    pub y_origin: glam::Vec3A,
    pub y_direction: glam::Vec3A,
}

impl Ray {
    pub const T_MIN_EPS: f32 = 0.0001;

    pub fn new(origin: glam::Vec3A, direction: glam::Vec3A) -> Self {
        Self {
            origin,
            direction,
            t_min: Self::T_MIN_EPS,
            aux_ray: None,
        }
    }

    pub fn point_at(&self, t: f32) -> glam::Vec3A {
        self.origin + self.direction * t
    }

    pub fn transformed_by(self, trans: glam::Affine3A) -> Self {
        let origin = trans.transform_point3a(self.origin);
        let direction = trans.transform_vector3a(self.direction);
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

    pub fn point_x_at(&self, t: f32) -> glam::Vec3A {
        self.x_origin + self.x_direction * t
    }

    pub fn point_y_at(&self, t: f32) -> glam::Vec3A {
        self.y_origin + self.y_direction * t
    }
}
