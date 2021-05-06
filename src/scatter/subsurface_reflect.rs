use crate::core::{color::Color, scatter::ScatterType};
use crate::core::coord::Coordinate;
use crate::core::intersection::Intersection;
use crate::core::primitive::Aggregate;
use crate::core::ray::Ray;
use crate::core::sampler::Sampler;
use crate::core::scatter::{Scatter, SsReflect};
use cgmath::{MetricSpace, Point3, Vector3};

pub struct SubsurfaceReflect {
    albedo: Color,
    _ld: f32,
    ior: f32,
    d: Color,
}

impl SubsurfaceReflect {
    pub fn new(albedo: Color, ld: f32, ior: f32) -> Self {
        let d = Color::new(
            ld / (3.5 + 100.0 * (albedo.r - 0.33).powi(4)),
            ld / (3.5 + 100.0 * (albedo.g - 0.33).powi(4)),
            ld / (3.5 + 100.0 * (albedo.b - 0.33).powi(4)),
        );
        Self {
            albedo,
            _ld: ld,
            ior,
            d,
        }
    }

    fn sp(&self, r: f32) -> Color {
        let exp1 = (-r / self.d).exp();
        let exp2 = (-r / self.d / 3.0).exp();
        (exp1 + exp2) / (8.0 * std::f32::consts::PI * self.d * r)
    }

    fn sample_r(&self, rand: f32) -> f32 {
        for i in 1..CDF_TABLE.len() {
            if CDF_TABLE[i].1 >= rand {
                let t = (rand - CDF_TABLE[i - 1].1) / (CDF_TABLE[i].1 - CDF_TABLE[i - 1].1);
                let x = CDF_TABLE[i].0 * t + CDF_TABLE[i - 1].0 * (1.0 - t);
                return x;
            }
        }
        -1.0
    }
}

impl Scatter for SubsurfaceReflect {
    fn sample_pi(
        &self,
        po: Point3<f32>,
        _wo: Vector3<f32>,
        coord_po: Coordinate,
        sampler: &mut dyn Sampler,
        scene: &dyn Aggregate,
    ) -> (Point3<f32>, Coordinate, f32, Color) {
        let mut rand_u = sampler.uniform_1d();
        let (rand_x, rand_y) = sampler.uniform_2d();

        // p for primitive
        let pt = coord_po.to_world(Vector3::unit_x());
        let pb = coord_po.to_world(Vector3::unit_y());
        let pn = coord_po.to_world(Vector3::unit_z());
        // s for sampled
        let (st, sb, sn) = if rand_u < 0.5 {
            rand_u = rand_u * 2.0;
            (pt, pb, pn)
        } else if rand_u < 0.75 {
            rand_u = rand_u * 4.0 - 2.0;
            (pb, pn, pt)
        } else {
            rand_u = rand_u * 4.0 - 3.0;
            (pn, pt, pb)
        };

        let sp_d = if rand_u < 1.0 / 3.0 {
            rand_u = 3.0 * rand_u;
            self.d.r
        } else if rand_u < 2.0 / 3.0 {
            rand_u = 3.0 * rand_u - 1.0;
            self.d.g
        } else {
            rand_u = 3.0 * rand_u - 2.0;
            self.d.b
        };
        let sample_r = self.sample_r(rand_x) * sp_d;
        let r_max = CDF_TABLE.last().unwrap().0 * sp_d;
        if sample_r < 0.0 {
            return (po, coord_po, 1.0, Color::BLACK);
        }
        let pihi = 2.0 * std::f32::consts::PI * rand_y;
        let pihi_cos = pihi.cos();
        let pihi_sin = pihi.sin();
        let sample_l = (r_max * r_max + sample_r * sample_r).sqrt();

        let start_p: Point3<f32> =
            po + st * pihi_cos * sample_r + sb * pihi_sin * sample_r + sn * sample_l;
        let mut ray = Ray::new(start_p, -sn);
        let mut inter = Intersection::with_t_max(2.0 * sample_l);
        let mut intersects = vec![];
        loop {
            if scene.intersect(&ray, &mut inter) {
                // TODO - check if the intersected one is the same as self
                inter.apply_normal_map();
                intersects.push((ray.point_at(inter.t), inter.normal, inter.shade_normal));
                ray.t_min = inter.t + Ray::T_MIN_EPS;
            } else {
                break;
            }
        }

        if intersects.is_empty() {
            return (po, coord_po, 1.0, Color::BLACK);
        }
        let sample_inter = ((rand_u * intersects.len() as f32) as usize).min(intersects.len() - 1);
        let (pi, sample_normal, sample_shade_normal) = intersects[sample_inter];

        let sp = self.albedo * self.sp(pi.distance(po));

        let offset = coord_po.to_local(pi - po);
        let normal_local = coord_po.to_local(sample_normal);
        let r_xy = (offset.x * offset.x + offset.y * offset.y).sqrt();
        let r_yz = (offset.y * offset.y + offset.z * offset.z).sqrt();
        let r_zx = (offset.z * offset.z + offset.x * offset.x).sqrt();
        let pdf_xy = 0.5 * normal_local.z.abs() * self.sp(r_xy).avg();
        let pdf_yz = 0.25 * normal_local.x.abs() * self.sp(r_yz).avg();
        let pdf_zx = 0.25 * normal_local.y.abs() * self.sp(r_zx).avg();
        let pdf = (pdf_xy + pdf_yz + pdf_zx) / intersects.len() as f32;

        (pi, Coordinate::from_z(sample_shade_normal, sample_normal), pdf, sp)
    }

    fn sample_wi(
        &self,
        po: Point3<f32>,
        wo: Vector3<f32>,
        pi: Point3<f32>,
        sampler: &mut dyn Sampler,
    ) -> (Vector3<f32>, f32, Color, ScatterType) {
        let mut wi = sampler.cosine_weighted_on_hemisphere();
        if wo.z < 0.0 {
            wi.z = -wi.z;
        }
        (
            wi,
            wi.z.abs() / std::f32::consts::PI,
            self.bxdf(po, wo, pi, wi),
            ScatterType::lambert_reflect(),
        )
    }

    fn pdf(&self, _po: Point3<f32>, wo: Vector3<f32>, _pi: Point3<f32>, wi: Vector3<f32>) -> f32 {
        if wo.z * wi.z >= 0.0 {
            wi.z.abs() / std::f32::consts::PI
        } else {
            1.0
        }
    }

    fn bxdf(
        &self,
        _po: Point3<f32>,
        wo: Vector3<f32>,
        _pi: Point3<f32>,
        wi: Vector3<f32>,
    ) -> Color {
        let fresnel_wo = crate::scatter::util::fresnel(self.ior, wo);
        let fresnel_wi = crate::scatter::util::fresnel(self.ior, wi);
        let value = (1.0 - fresnel_wo) * (1.0 - fresnel_wi) / std::f32::consts::PI;
        Color::new(value, value, value)
    }

    fn is_delta(&self) -> bool {
        false
    }
}

impl SsReflect for SubsurfaceReflect {}

lazy_static! {
    static ref CDF_TABLE: Vec<(f32, f32)> = (0..512)
        .map(|i| {
            let x = i as f32 / 512.0;
            let x = -2.0 * (1.0 - x).ln();
            let y = 1.0 - (-x).exp() * 0.25 - (-x / 3.0).exp() * 0.75;
            (x, y)
        })
        .collect();
}
