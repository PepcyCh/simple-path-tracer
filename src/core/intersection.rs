use crate::{
    core::{ray::Ray, surface::Surface},
    primitive::BasicPrimitiveRef,
};

pub struct Intersection<'a> {
    pub t: f32,
    pub position: glam::Vec3A,
    /// tangent - dpdu
    pub tangent: glam::Vec3A,
    /// bitangent - dpdv
    pub bitangent: glam::Vec3A,
    pub normal: glam::Vec3A,
    pub texcoords: glam::Vec2,
    pub primitive: Option<BasicPrimitiveRef<'a>>,
    pub surface: Option<&'a Surface>,
    pub duvdx: glam::Vec2,
    pub duvdy: glam::Vec2,
}

impl Intersection<'_> {
    pub fn with_t_max(t_max: f32) -> Self {
        Self {
            t: t_max,
            ..Default::default()
        }
    }

    pub fn calc_differential(&mut self, ray: &Ray) {
        if let Some(aux_ray) = &ray.aux_ray {
            let p = ray.point_at(self.t);

            let d = p.dot(self.normal);
            let tx =
                (d - aux_ray.x_origin.dot(self.normal)) / (aux_ray.x_direction.dot(self.normal));
            let px = aux_ray.point_x_at(tx);
            let ty =
                (d - aux_ray.y_origin.dot(self.normal)) / (aux_ray.y_direction.dot(self.normal));
            let py = aux_ray.point_y_at(ty);

            let dpdx = px - p;
            let dpdy = py - p;
            let mut bx = glam::Vec2::ZERO;
            let mut by = glam::Vec2::ZERO;
            let mut a = glam::Mat2::ZERO;
            if self.normal.x.abs() >= self.normal.y.abs()
                && self.normal.x.abs() >= self.normal.z.abs()
            {
                bx.x = dpdx.y;
                bx.y = dpdx.z;
                by.x = dpdy.y;
                by.y = dpdy.z;
                a.col_mut(0).x = self.tangent.y;
                a.col_mut(0).y = self.tangent.z;
                a.col_mut(1).x = self.bitangent.y;
                a.col_mut(1).y = self.bitangent.z;
            } else if self.normal.y.abs() >= self.normal.z.abs() {
                bx.x = dpdx.z;
                bx.y = dpdx.x;
                by.x = dpdy.z;
                by.y = dpdy.x;
                a.col_mut(0).x = self.tangent.z;
                a.col_mut(0).y = self.tangent.x;
                a.col_mut(1).x = self.bitangent.z;
                a.col_mut(1).y = self.bitangent.x;
            } else {
                bx.x = dpdx.x;
                bx.y = dpdx.y;
                by.x = dpdy.x;
                by.y = dpdy.y;
                a.col_mut(0).x = self.tangent.x;
                a.col_mut(0).y = self.tangent.y;
                a.col_mut(1).x = self.bitangent.x;
                a.col_mut(1).y = self.bitangent.y;
            }

            if let Some((x1, x2)) = solve_linear_system_2x2(a, bx) {
                self.duvdx = glam::Vec2::new(x1, x2);
            }
            if let Some((x1, x2)) = solve_linear_system_2x2(a, by) {
                self.duvdy = glam::Vec2::new(x1, x2);
            }
        }
    }
}

impl Default for Intersection<'_> {
    fn default() -> Self {
        Self {
            t: f32::MAX,
            position: glam::Vec3A::ZERO,
            tangent: glam::Vec3A::X,
            bitangent: glam::Vec3A::Y,
            normal: glam::Vec3A::Z,
            texcoords: glam::Vec2::ZERO,
            primitive: None,
            surface: None,
            duvdx: glam::Vec2::ZERO,
            duvdy: glam::Vec2::ZERO,
        }
    }
}

fn solve_linear_system_2x2(a: glam::Mat2, b: glam::Vec2) -> Option<(f32, f32)> {
    let det = a.determinant();
    if det != 0.0 {
        let temp = b.y * a.col(0).x - b.x * a.col(0).y;
        let x2 = temp / det;
        let x1 = if a.col(0).x.abs() > a.col(0).y.abs() {
            (b.x - a.col(1).x * x2) / a.col(0).x
        } else {
            (b.y - a.col(1).y * x2) / a.col(0).y
        };
        Some((x1, x2))
    } else {
        None
    }
}
