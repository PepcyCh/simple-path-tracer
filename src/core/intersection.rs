use crate::core::primitive::Primitive;
use crate::core::ray::Ray;
use cgmath::{EuclideanSpace, InnerSpace, SquareMatrix, Zero};

pub struct Intersection<'a> {
    pub t: f32,
    /// tangent - dpdu
    pub tangent: cgmath::Vector3<f32>,
    /// bitangent - dpdv
    pub bitangent: cgmath::Vector3<f32>,
    pub normal: cgmath::Vector3<f32>,
    /// shade_normal - normal from normal map (in world space)
    pub shade_normal: cgmath::Vector3<f32>,
    pub texcoords: cgmath::Point2<f32>,
    pub primitive: Option<&'a dyn Primitive>,
    pub duvdx: cgmath::Vector2<f32>,
    pub duvdy: cgmath::Vector2<f32>,
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
            let mut bx = cgmath::Vector2::zero();
            let mut by = cgmath::Vector2::zero();
            let mut a = cgmath::Matrix2::zero();
            if self.normal.x.abs() >= self.normal.y.abs()
                && self.normal.x.abs() >= self.normal.z.abs()
            {
                bx.x = dpdx.y;
                bx.y = dpdx.z;
                by.x = dpdy.y;
                by.y = dpdy.z;
                a.x.x = self.tangent.y;
                a.x.y = self.tangent.z;
                a.y.x = self.bitangent.y;
                a.y.y = self.bitangent.z;
            } else if self.normal.y.abs() >= self.normal.z.abs() {
                bx.x = dpdx.z;
                bx.y = dpdx.x;
                by.x = dpdy.z;
                by.y = dpdy.x;
                a.x.x = self.tangent.z;
                a.x.y = self.tangent.x;
                a.y.x = self.bitangent.z;
                a.y.y = self.bitangent.x;
            } else {
                bx.x = dpdx.x;
                bx.y = dpdx.y;
                by.x = dpdy.x;
                by.y = dpdy.y;
                a.x.x = self.tangent.x;
                a.x.y = self.tangent.y;
                a.y.x = self.bitangent.x;
                a.y.y = self.bitangent.y;
            }

            if let Some((x1, x2)) = solve_linear_system_2x2(a, bx) {
                self.duvdx = cgmath::Vector2::new(x1, x2);
            }
            if let Some((x1, x2)) = solve_linear_system_2x2(a, by) {
                self.duvdy = cgmath::Vector2::new(x1, x2);
            }
        }
    }

    pub fn apply_normal_map(&mut self) {
        if let Some(prim) = self.primitive {
            if let Some(mat) = prim.material() {
                let shade_normal_local = mat.apply_normal_map(self);
                self.shade_normal = (shade_normal_local.x * self.tangent.normalize()
                    + shade_normal_local.y * self.bitangent.normalize()
                    + shade_normal_local.z * self.normal)
                    .normalize();
            }
        }
    }
}

impl Default for Intersection<'_> {
    fn default() -> Self {
        Self {
            t: f32::MAX,
            tangent: cgmath::Vector3::unit_x(),
            bitangent: cgmath::Vector3::unit_y(),
            normal: cgmath::Vector3::unit_z(),
            shade_normal: cgmath::Vector3::unit_z(),
            texcoords: cgmath::Point2::new(0.0, 0.0),
            primitive: None,
            duvdx: cgmath::Vector2::zero(),
            duvdy: cgmath::Vector2::zero(),
        }
    }
}

fn solve_linear_system_2x2(a: cgmath::Matrix2<f32>, b: cgmath::Vector2<f32>) -> Option<(f32, f32)> {
    let det = a.determinant();
    if det != 0.0 {
        let temp = b.y * a.x.x - b.x * a.x.y;
        let x2 = temp / det;
        let x1 = if a.x.x.abs() > a.x.y.abs() {
            (b.x - a.y.x * x2) / a.x.x
        } else {
            (b.y - a.y.y * x2) / a.x.y
        };
        Some((x1, x2))
    } else {
        None
    }
}
