use super::scatter::ScatterDirType;

#[derive(Copy, Clone)]
pub struct Coordinate {
    local_to_world: glam::Mat3A,
    world_to_local: glam::Mat3A,
    hemisphere: glam::Vec3A,
}

impl Coordinate {
    pub fn from_z(z_world: glam::Vec3A, hemisphere: glam::Vec3A) -> Self {
        let y_world = if z_world.y.abs() < 0.99 {
            glam::Vec3A::Y
        } else {
            glam::Vec3A::X
        };
        let x_world = (y_world.cross(z_world)).normalize();
        let y_world = z_world.cross(x_world);

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
