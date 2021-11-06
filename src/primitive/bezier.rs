use std::sync::Arc;

use anyhow::Context;

use crate::{
    core::{
        bbox::Bbox, intersection::Intersection, primitive::Primitive, ray::Ray, sampler::Sampler,
        scene::Scene,
    },
    loader::{self, JsonObject, LoadableSceneObject},
};

// Newton's iteration
#[cfg(feature = "bezier_ni")]
const NEWTON_ITERATION_MAX_TIMES: u32 = 16;
#[cfg(feature = "bezier_ni")]
const NEWTON_ITERATION_EPS: f32 = 0.000000001;
// Bezier clipping
#[cfg(not(feature = "bezier_ni"))]
const CLIPPING_MAX_TIMES: u32 = 16;
#[cfg(not(feature = "bezier_ni"))]
const CLIPPING_EPS: f32 = 0.00001;

pub struct CubicBezier {
    control_points: [[glam::Vec3A; 4]; 4],
    bbox: Bbox,
}

impl CubicBezier {
    pub fn new(control_points: [[glam::Vec3A; 4]; 4]) -> Self {
        let (p_min, p_max) = control_points
            .iter()
            .flatten()
            .fold((control_points[0][0], control_points[0][0]), |acc, curr| {
                (acc.0.min(*curr), acc.1.max(*curr))
            });
        let bbox = Bbox::new(p_min, p_max);

        Self {
            control_points,
            bbox,
        }
    }

    fn point_at(&self, u: f32, v: f32) -> glam::Vec3A {
        let bezier_u = cubic_bezier_at(u);
        let bezier_v = cubic_bezier_at(v);
        cubic_bezier_sum(&self.control_points, &bezier_u, &bezier_v)
    }

    fn tangent_at(&self, u: f32, v: f32) -> glam::Vec3A {
        let bezier_du = cubic_bezier_du_at(u);
        let bezier_v = cubic_bezier_at(v);
        cubic_bezier_sum(&self.control_points, &bezier_du, &bezier_v)
    }

    fn bitangent_at(&self, u: f32, v: f32) -> glam::Vec3A {
        let bezier_u = cubic_bezier_at(u);
        let bezier_dv = cubic_bezier_du_at(v);
        cubic_bezier_sum(&self.control_points, &bezier_u, &bezier_dv)
    }

    /// returns (u, v, t) of intersected point if exists, using Newton's iteration
    #[cfg(feature = "bezier_ni")]
    fn intersect_ray(&self, ray: &Ray) -> Option<(f32, f32, f32)> {
        if let Some((t0, t1)) = self.bbox.intersect_ray(ray) {
            let mut t = 0.5 * (t0 + t1);
            let mut u = 0.5;
            let mut v = 0.5;

            for _ in 0..NEWTON_ITERATION_MAX_TIMES {
                let point = self.point_at(u, v);
                let diff = ray.point_at(t) - point;

                if !t.is_finite() || !u.is_finite() || !v.is_finite() {
                    break;
                }

                if diff.length_squared() < NEWTON_ITERATION_EPS {
                    if u >= 0.0 && u <= 1.0 && v >= 0.0 && v <= 1.0 && t > ray.t_min {
                        return Some((u, v, t));
                    }
                    break;
                }

                let dpdu = self.tangent_at(u, v);
                let dpdv = self.bitangent_at(u, v);

                let n = dpdu.cross(dpdv);
                let det = ray.direction.dot(n);
                if det == 0.0 {
                    break;
                }
                let det = 1.0 / det;
                let dt = diff.dot(n) * det;
                let q = ray.direction.cross(diff);
                let du = -dpdv.dot(q) * det;
                let dv = dpdu.dot(q) * det;

                t -= dt;
                u -= du;
                v -= dv;
            }
        }

        None
    }

    /// returns (u, v, t) of intersected point if exists, using Bezier clipping
    #[cfg(not(feature = "bezier_ni"))]
    fn intersect_ray(&self, ray: &Ray) -> Option<(f32, f32, f32)> {
        let mut patch = [[glam::Vec2::new(0.0, 0.0); 4]; 4];
        let n1 = glam::Vec3A::new(-ray.direction.y, ray.direction.x, 0.0).normalize();
        let n2 = glam::Vec3A::new(0.0, -ray.direction.z, ray.direction.y).normalize();
        for i in 0..4 {
            for j in 0..4 {
                let diff = self.control_points[i][j] - ray.origin;
                patch[i][j] = glam::Vec2::new(diff.dot(n1), diff.dot(n2));
            }
        }
        let lu = ((patch[3][0] - patch[0][0]) + (patch[3][3] - patch[0][3])).normalize();
        let lv = ((patch[0][3] - patch[0][0]) + (patch[3][3] - patch[3][0])).normalize();
        let intersections = bezier_clipping(patch, lu, lv, (1.0, 0.0), (1.0, 0.0), true, None, 0);
        let mut t_min = std::f32::MAX;
        let mut result = None;
        for inter in intersections {
            let p = self.point_at(inter.x, inter.y);
            let diff = p - ray.origin;
            let cross = diff.cross(ray.direction);
            if cross.length_squared() < CLIPPING_EPS {
                let t = diff.dot(ray.direction) / ray.direction.length_squared();
                if t > ray.t_min && t < t_min {
                    t_min = t;
                    result = Some((inter.x, inter.y, t));
                }
            }
        }
        result
    }
}

impl Primitive for CubicBezier {
    fn intersect_test(&self, ray: &Ray, t_max: f32) -> bool {
        if let Some((_, _, t)) = self.intersect_ray(ray) {
            t > ray.t_min && t < t_max
        } else {
            false
        }
    }

    fn intersect<'a>(&'a self, ray: &Ray, inter: &mut Intersection<'a>) -> bool {
        if let Some((u, v, t)) = self.intersect_ray(ray) {
            if t > ray.t_min && t < inter.t {
                inter.t = t;
                inter.texcoords = glam::Vec2::new(u, v);
                inter.tangent = self.tangent_at(u, v);
                inter.bitangent = self.bitangent_at(u, v);
                inter.normal = (inter.tangent.cross(inter.bitangent)).normalize();
                inter.primitive = Some(self);
                return true;
            }
        }
        false
    }

    fn bbox(&self) -> Bbox {
        self.bbox
    }

    fn sample<'a>(&'a self, _sampler: &mut dyn Sampler) -> (Intersection<'a>, f32) {
        unimplemented!("<CubicBezier as Primitive>::sample() not supported yet")
    }

    fn pdf(&self, _inter: &Intersection<'_>) -> f32 {
        unimplemented!("<CubicBezier as Primitive>::pdf() not supported yet")
    }
}

fn cubic_bezier_at(u: f32) -> [f32; 4] {
    let iu = 1.0 - u;
    [iu * iu * iu, 3.0 * iu * iu * u, 3.0 * u * u * iu, u * u * u]
}

fn cubic_bezier_du_at(u: f32) -> [f32; 4] {
    let iu = 1.0 - u;
    [
        -3.0 * iu * iu,
        3.0 * iu * iu - 6.0 * iu * u,
        6.0 * u * iu - 3.0 * u * u,
        3.0 * u * u,
    ]
}

fn cubic_bezier_sum(
    points: &[[glam::Vec3A; 4]; 4],
    basic_u: &[f32; 4],
    basic_v: &[f32; 4],
) -> glam::Vec3A {
    let mut result = glam::Vec3A::ZERO;
    for i in 0..4 {
        for j in 0..4 {
            result += basic_u[j] * basic_v[i] * points[i][j];
        }
    }
    result
}

#[cfg(not(feature = "bezier_ni"))]
fn bezier_clipping(
    patch: [[glam::Vec2; 4]; 4],
    lu: glam::Vec2,
    lv: glam::Vec2,
    affine_u: (f32, f32),
    affine_v: (f32, f32),
    real_u: bool,
    mut calculated: Option<f32>,
    times: u32,
) -> Vec<glam::Vec2> {
    if times == CLIPPING_MAX_TIMES {
        let u = 0.5 * affine_u.0 + affine_u.1;
        let v = if let Some(calculated) = calculated {
            calculated
        } else {
            0.5 * affine_v.0 + affine_v.1
        };
        return if real_u {
            vec![glam::Vec2::new(u, v)]
        } else {
            vec![glam::Vec2::new(v, u)]
        };
    }

    let mut upper_points = [0.0; 4];
    let mut lower_points = [0.0; 4];

    for i in 0..4 {
        for j in 0..4 {
            let dist = patch[i][j].x * lu.y - patch[i][j].y * lu.x;
            if i == 0 || dist > upper_points[j] {
                upper_points[j] = dist;
            }
            if i == 0 || dist < lower_points[j] {
                lower_points[j] = dist;
            }
        }
    }

    let pairs = [(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)];
    let mut u_min = if upper_points[0] >= 0.0 && lower_points[0] <= 0.0 {
        0.0_f32
    } else {
        1.0_f32
    };
    let mut u_max = if upper_points[3] >= 0.0 && lower_points[3] <= 0.0 {
        1.0_f32
    } else {
        0.0_f32
    };
    for pair in &pairs {
        if upper_points[pair.0] * upper_points[pair.1] <= 0.0 {
            let diff = upper_points[pair.1] - upper_points[pair.0];
            if diff == 0.0 {
                u_min = u_min.min(pair.0 as f32 / 3.0);
                u_max = u_max.max(pair.1 as f32 / 3.0);
            } else {
                let k = (pair.1 - pair.0) as f32 / 3.0 / diff;
                let b = pair.0 as f32 / 3.0 - k * upper_points[pair.0];
                u_min = u_min.min(b);
                u_max = u_max.max(b);
            }
        }

        if lower_points[pair.0] * lower_points[pair.1] <= 0.0 {
            let diff = lower_points[pair.1] - lower_points[pair.0];
            if diff == 0.0 {
                u_min = u_min.min(pair.0 as f32 / 3.0);
                u_max = u_max.max(pair.1 as f32 / 3.0);
            } else {
                let k = (pair.1 - pair.0) as f32 / 3.0 / diff;
                let b = pair.1 as f32 / 3.0 - k * lower_points[pair.1];
                u_min = u_min.min(b);
                u_max = u_max.max(b);
            }
        }
    }

    if u_max < u_min {
        return vec![];
    }

    let swap = calculated.is_none();
    if u_max - u_min > 0.8 {
        let (new_u0_l, new_u0_r) = clip_bezier_at_midpoint(patch[0]);
        let (new_u1_l, new_u1_r) = clip_bezier_at_midpoint(patch[1]);
        let (new_u2_l, new_u2_r) = clip_bezier_at_midpoint(patch[2]);
        let (new_u3_l, new_u3_r) = clip_bezier_at_midpoint(patch[3]);
        if swap {
            let new_patch_l = [
                [new_u0_l[0], new_u1_l[0], new_u2_l[0], new_u3_l[0]],
                [new_u0_l[1], new_u1_l[1], new_u2_l[1], new_u3_l[1]],
                [new_u0_l[2], new_u1_l[2], new_u2_l[2], new_u3_l[2]],
                [new_u0_l[3], new_u1_l[3], new_u2_l[3], new_u3_l[3]],
            ];
            let new_patch_r = [
                [new_u0_r[0], new_u1_r[0], new_u2_r[0], new_u3_r[0]],
                [new_u0_r[1], new_u1_r[1], new_u2_r[1], new_u3_r[1]],
                [new_u0_r[2], new_u1_r[2], new_u2_r[2], new_u3_r[2]],
                [new_u0_r[3], new_u1_r[3], new_u2_r[3], new_u3_r[3]],
            ];

            let mut results = vec![];
            results.append(&mut bezier_clipping(
                new_patch_l,
                lv,
                lu,
                affine_v,
                (affine_u.0 * 0.5, affine_u.1),
                !real_u,
                None,
                times + 1,
            ));
            results.append(&mut bezier_clipping(
                new_patch_r,
                lv,
                lu,
                affine_v,
                (affine_u.0 * 0.5, affine_u.0 * 0.5 + affine_u.1),
                !real_u,
                None,
                times + 1,
            ));
            results
        } else {
            let new_patch_l = [new_u0_l, new_u1_l, new_u2_l, new_u3_l];
            let new_patch_r = [new_u0_r, new_u1_r, new_u2_r, new_u3_r];

            let mut results = vec![];
            results.append(&mut bezier_clipping(
                new_patch_l,
                lu,
                lv,
                (affine_u.0 * 0.5, affine_u.1),
                affine_v,
                real_u,
                calculated,
                times + 1,
            ));
            results.append(&mut bezier_clipping(
                new_patch_r,
                lu,
                lv,
                (affine_u.0 * 0.5, affine_u.0 * 0.5 + affine_u.1),
                affine_v,
                real_u,
                calculated,
                times + 1,
            ));
            results
        }
    } else {
        let u_len = u_max - u_min;
        let stop = u_len * affine_u.0 < CLIPPING_EPS;
        if stop {
            let u = 0.5 * (u_max + u_min) * affine_u.0 + affine_u.1;
            if let Some(calculated) = calculated {
                return if real_u {
                    vec![glam::Vec2::new(u, calculated)]
                } else {
                    vec![glam::Vec2::new(calculated, u)]
                };
            }
            calculated = Some(u);
        }

        let new_u0 = clip_bezier_by(patch[0], u_min, u_max);
        let new_u1 = clip_bezier_by(patch[1], u_min, u_max);
        let new_u2 = clip_bezier_by(patch[2], u_min, u_max);
        let new_u3 = clip_bezier_by(patch[3], u_min, u_max);
        if swap {
            let new_patch = [
                [new_u0[0], new_u1[0], new_u2[0], new_u3[0]],
                [new_u0[1], new_u1[1], new_u2[1], new_u3[1]],
                [new_u0[2], new_u1[2], new_u2[2], new_u3[2]],
                [new_u0[3], new_u1[3], new_u2[3], new_u3[3]],
            ];
            bezier_clipping(
                new_patch,
                lv,
                lu,
                affine_v,
                (affine_u.0 * u_len, affine_u.0 * u_min + affine_u.1),
                !real_u,
                calculated,
                times + 1,
            )
        } else {
            let new_patch = [new_u0, new_u1, new_u2, new_u3];
            bezier_clipping(
                new_patch,
                lu,
                lv,
                (affine_u.0 * u_len, affine_u.0 * u_min + affine_u.1),
                affine_v,
                real_u,
                calculated,
                times + 1,
            )
        }
    }
}

#[cfg(not(feature = "bezier_ni"))]
fn clip_bezier_by(points: [glam::Vec2; 4], u_min: f32, u_max: f32) -> [glam::Vec2; 4] {
    let bezier_u_min = cubic_bezier_at(u_min);
    let p_min = points[0] * bezier_u_min[0]
        + points[1] * bezier_u_min[1]
        + points[2] * bezier_u_min[2]
        + points[3] * bezier_u_min[3];
    let bezier_du_min = cubic_bezier_du_at(u_min);
    let d_min = points[0] * bezier_du_min[0]
        + points[1] * bezier_du_min[1]
        + points[2] * bezier_du_min[2]
        + points[3] * bezier_du_min[3];
    let d_min = d_min * (u_max - u_min);

    let bezier_u_max = cubic_bezier_at(u_max);
    let p_max = points[0] * bezier_u_max[0]
        + points[1] * bezier_u_max[1]
        + points[2] * bezier_u_max[2]
        + points[3] * bezier_u_max[3];
    let bezier_du_max = cubic_bezier_du_at(u_max);
    let d_max = points[0] * bezier_du_max[0]
        + points[1] * bezier_du_max[1]
        + points[2] * bezier_du_max[2]
        + points[3] * bezier_du_max[3];
    let d_max = d_max * (u_max - u_min);

    let p1 = p_min + d_min / 3.0;
    let p2 = p_max - d_max / 3.0;

    [p_min, p1, p2, p_max]
}

#[cfg(not(feature = "bezier_ni"))]
fn clip_bezier_at_midpoint(points: [glam::Vec2; 4]) -> ([glam::Vec2; 4], [glam::Vec2; 4]) {
    let bezier_u_mid = cubic_bezier_at(0.5);
    let p_mid = points[0] * bezier_u_mid[0]
        + points[1] * bezier_u_mid[1]
        + points[2] * bezier_u_mid[2]
        + points[3] * bezier_u_mid[3];
    let bezier_du_mid = cubic_bezier_du_at(0.5);
    let d_mid = points[0] * bezier_du_mid[0]
        + points[1] * bezier_du_mid[1]
        + points[2] * bezier_du_mid[2]
        + points[3] * bezier_du_mid[3];
    let d_mid = d_mid * 0.5 / 3.0;

    (
        [
            points[0],
            (points[0] + points[1]) * 0.5,
            p_mid - d_mid,
            p_mid,
        ],
        [
            p_mid,
            p_mid + d_mid,
            (points[2] + points[3]) * 0.5,
            points[3],
        ],
    )
}

impl LoadableSceneObject for CubicBezier {
    fn load(
        scene: &mut Scene,
        _path: &std::path::PathBuf,
        json_value: &JsonObject,
    ) -> anyhow::Result<()> {
        let name = loader::get_str_field(json_value, "primitive-cubic_bezier", "name")?;
        let env = format!("primitive-cubic_bezier({})", name);
        if scene.primitives.contains_key(name) {
            anyhow::bail!(format!("{}: name is duplicated", env));
        }

        let cp_value = json_value
            .get("control_points")
            .context(format!("{}: no 'control_points' field", env))?;
        let error_info = format!("{}: 'control_points' should be a 4x4 array of float3", env);
        let cp_arr = cp_value.as_array().context(error_info.clone())?;
        if cp_arr.len() != 4 {
            anyhow::bail!(error_info.clone());
        }
        let mut control_points = [[glam::Vec3A::new(0.0, 0.0, 0.0); 4]; 4];
        for i in 0..4 {
            let cp_row_arr = cp_arr[i].as_array().context(error_info.clone())?;
            if cp_row_arr.len() != 4 {
                anyhow::bail!(error_info.clone());
            }
            for j in 0..4 {
                let cp_point_arr = cp_row_arr[j].as_array().context(error_info.clone())?;
                if cp_point_arr.len() != 3 {
                    anyhow::bail!(error_info.clone());
                }
                control_points[i][j] = glam::Vec3A::new(
                    cp_point_arr[0].as_f64().context(error_info.clone())? as f32,
                    cp_point_arr[1].as_f64().context(error_info.clone())? as f32,
                    cp_point_arr[2].as_f64().context(error_info.clone())? as f32,
                );
            }
        }

        let bezier = CubicBezier::new(control_points);
        scene.primitives.insert(name.to_owned(), Arc::new(bezier));

        Ok(())
    }
}
