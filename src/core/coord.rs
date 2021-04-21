use cgmath::{InnerSpace, Matrix, Matrix3, Vector3};

#[derive(Copy, Clone)]
pub struct Coordinate {
    local_to_world: Matrix3<f32>,
    world_to_local: Matrix3<f32>,
}

impl Coordinate {
    pub fn from_z(z_world: Vector3<f32>) -> Self {
        let y_world = if z_world.y.abs() < 0.99 {
            cgmath::Vector3::unit_y()
        } else {
            cgmath::Vector3::unit_x()
        };
        let x_world = (y_world.cross(z_world)).normalize();
        let y_world = z_world.cross(x_world);

        let local_to_world = Matrix3::from_cols(x_world, y_world, z_world);
        let world_to_local = local_to_world.transpose();
        Self {
            local_to_world,
            world_to_local,
        }
    }

    pub fn to_local(&self, world: Vector3<f32>) -> Vector3<f32> {
        self.world_to_local * world
    }

    pub fn to_world(&self, local: Vector3<f32>) -> Vector3<f32> {
        self.local_to_world * local
    }
}
