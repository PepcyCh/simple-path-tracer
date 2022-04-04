use crate::scatter::ScatterDirType;

#[derive(Copy, Clone)]
pub struct Coordinate {
    local_to_world: glam::Mat3A,
    world_to_local: glam::Mat3A,
    hemisphere: glam::Vec3A,
}

impl Coordinate {
    pub fn from_tangent_normal(t: glam::Vec3A, n: glam::Vec3A, hemisphere: glam::Vec3A) -> Self {
        let z_world = n;
        let y_world = z_world.cross(t).normalize();
        let x_world = y_world.cross(z_world);

        let local_to_world = glam::Mat3A::from_cols(x_world, y_world, z_world);
        let world_to_local = local_to_world.transpose();
        Self {
            local_to_world,
            world_to_local,
            hemisphere,
        }
    }

    pub fn from_z(z_world: glam::Vec3A, hemisphere: glam::Vec3A) -> Self {
        let sign = if z_world.z >= 0.0 { 1.0 } else { -1.0 };
        let a = -1.0 / (sign + z_world.z);
        let b = z_world.x * z_world.y * a;
        let x_world = glam::Vec3A::new(
            1.0 + sign * z_world.x * z_world.x * a,
            sign * b,
            -sign * z_world.x,
        );
        let y_world = glam::Vec3A::new(b, sign + z_world.y * z_world.y * a, -z_world.y);

        let local_to_world = glam::Mat3A::from_cols(x_world, y_world, z_world);
        let world_to_local = local_to_world.transpose();
        Self {
            local_to_world,
            world_to_local,
            hemisphere,
        }
    }

    pub fn to_local(&self, world: glam::Vec3A) -> glam::Vec3A {
        self.world_to_local * world
    }

    pub fn to_world(&self, local: glam::Vec3A) -> glam::Vec3A {
        self.local_to_world * local
    }

    pub fn in_expected_hemisphere(&self, dir: glam::Vec3A, ty: ScatterDirType) -> bool {
        if ty == ScatterDirType::Reflect {
            dir.dot(self.hemisphere) >= 0.0
        } else {
            dir.dot(self.hemisphere) <= 0.0
        }
    }
}
