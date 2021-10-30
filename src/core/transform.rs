#[derive(Debug, Clone, Copy)]
pub struct Transform {
    trans: glam::Affine3A,
    trans_it: glam::Mat3A,
}

impl Transform {
    pub const IDENTITY: Transform = Transform {
        trans: glam::Affine3A::IDENTITY,
        trans_it: glam::Mat3A::IDENTITY,
    };

    pub fn new(trans: glam::Affine3A) -> Self {
        let trans_inv = trans.inverse();
        let trans_it = trans_inv.matrix3.transpose();
        Self {
            trans,
            trans_it,
        }
    }

    pub fn transform_point3a(&self, other: glam::Vec3A) -> glam::Vec3A {
        self.trans.transform_point3a(other)
    }

    pub fn transform_vector3a(&self, other: glam::Vec3A) -> glam::Vec3A {
        self.trans.transform_vector3a(other)
    }

    pub fn transform_normal3a(&self, other: glam::Vec3A) -> glam::Vec3A {
        (self.trans_it * other).normalize()
    }
}

impl std::ops::Mul for Transform {
    type Output = Transform;

    fn mul(self, rhs: Self) -> Self::Output {
        Transform {
            trans: self.trans * rhs.trans,
            trans_it: self.trans_it * rhs.trans_it,
        }
    }
}