use crate::core::color::Color;
use crate::core::intersection::Intersection;
use crate::core::material::Material;
use crate::core::primitive::Aggregate;
use crate::core::ray::Ray;
use crate::core::sampler::Sampler;
use cgmath::{Matrix, Matrix3, MetricSpace, Point3, Vector3};
use std::sync::Mutex;

pub struct Subsurface {
    albedo: Color,
    _ld: f32,
    ior: f32,
    d: Color,
    sampler: Box<Mutex<dyn Sampler>>,
    cdf_table: Vec<(f32, f32)>, // TODO - is there a better place for this table?
}

impl Subsurface {
    pub fn new(albedo: Color, ld: f32, ior: f32, sampler: Box<Mutex<dyn Sampler>>) -> Self {
        let cdf_table = (0..512)
            .map(|i| {
                let x = i as f32 / 512.0;
                let x = -2.0 * (1.0 - x).ln();
                let y = 1.0 - (-x).exp() * 0.25 - (-x / 3.0).exp() * 0.75;
                (x, y)
            })
            .collect();
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
            sampler,
            cdf_table,
        }
    }

    fn sp(&self, r: f32) -> Color {
        let exp1 = (-r / self.d).exp();
        let exp2 = (-r / self.d / 3.0).exp();
        (exp1 + exp2) / (8.0 * std::f32::consts::PI * self.d * r)
    }

    fn sample_r(&self, rand: f32) -> f32 {
        for i in 1..self.cdf_table.len() {
            if self.cdf_table[i].1 >= rand {
                let t = (rand - self.cdf_table[i - 1].1)
                    / (self.cdf_table[i].1 - self.cdf_table[i - 1].1);
                let x = self.cdf_table[i].0 * t + self.cdf_table[i - 1].0 * (1.0 - t);
                return x;
            }
        }
        -1.0
    }
}

impl Material for Subsurface {
    fn sample(&self, wo: Vector3<f32>) -> (Vector3<f32>, f32, Color) {
        let sample = {
            let mut sampler = self.sampler.lock().unwrap();
            sampler.cosine_weighted_on_hemisphere()
        };
        (
            sample,
            sample.z / std::f32::consts::PI,
            self.bsdf(wo, sample),
        )
    }

    fn bsdf(&self, wo: Vector3<f32>, wi: Vector3<f32>) -> Color {
        let fresnel_wo = crate::material::util::fresnel(self.ior, wo);
        let fresnel_wi = crate::material::util::fresnel(self.ior, wi);
        let value = (1.0 - fresnel_wo) * (1.0 - fresnel_wi) / std::f32::consts::PI;
        Color::new(value, value, value)
    }

    fn pdf(&self, _wo: Vector3<f32>, wi: Vector3<f32>) -> f32 {
        wi.z.max(0.0) / std::f32::consts::PI
    }

    fn is_delta(&self) -> bool {
        false
    }

    fn emissive(&self) -> Color {
        Color::BLACK
    }

    fn sample_sp(
        &self,
        p: Point3<f32>,
        wo: Vector3<f32>,
        normal_to_world: Matrix3<f32>,
        scene: &dyn Aggregate,
    ) -> (Point3<f32>, Vector3<f32>, Color) {
        let (mut rand_u, rand_x, rand_y) = {
            let mut sampler = self.sampler.lock().unwrap();
            let rand_u = sampler.uniform_1d();
            let (rand_x, rand_y) = sampler.uniform_2d();
            (rand_u, rand_x, rand_y)
        };

        // TODO - where to put codes about reflect

        // p for primitive
        let pt = normal_to_world * Vector3::unit_x();
        let pb = normal_to_world * Vector3::unit_y();
        let pn = normal_to_world * Vector3::unit_z();
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
        let r_max = self.cdf_table.last().unwrap().0 * sp_d;
        if sample_r < 0.0 {
            return (p, wo, Color::BLACK);
        }
        let sample_phi = 2.0 * std::f32::consts::PI * rand_y;
        let sample_phi_cos = sample_phi.cos();
        let sample_phi_sin = sample_phi.sin();
        let sample_l = (r_max * r_max + sample_r * sample_r).sqrt();

        let start_p: Point3<f32> =
            p + st * sample_phi_cos * sample_r + sb * sample_phi_sin * sample_r + sn * sample_l;
        let mut ray = Ray::new(start_p, -sn);
        let mut inter = Intersection::with_t_max(2.0 * sample_l);
        let mut intersects = vec![];
        loop {
            if scene.intersect(&ray, &mut inter) {
                // TODO - check if the intersected one is the same as self
                intersects.push((ray.point_at(inter.t), inter.normal));
                ray.t_min = inter.t + Ray::T_MIN_EPS;
            } else {
                break;
            }
        }

        if intersects.is_empty() {
            return (p, wo, Color::BLACK);
        }
        let sample_inter = ((rand_u * intersects.len() as f32) as usize).min(intersects.len() - 1);
        let (sample_p, sample_normal) = intersects[sample_inter];

        let sp = self.albedo * self.sp(sample_p.distance(p));

        let world_to_normal = normal_to_world.transpose();
        let offset = world_to_normal * (sample_p - p);
        let normal_local = world_to_normal * sample_normal;
        let r_xy = (offset.x * offset.x + offset.y * offset.y).sqrt();
        let r_yz = (offset.y * offset.y + offset.z * offset.z).sqrt();
        let r_zx = (offset.z * offset.z + offset.x * offset.x).sqrt();
        let pdf_xy = 0.5 * normal_local.z.abs() * self.sp(r_xy).avg();
        let pdf_yz = 0.25 * normal_local.x.abs() * self.sp(r_yz).avg();
        let pdf_zx = 0.25 * normal_local.y.abs() * self.sp(r_zx).avg();
        let pdf = (pdf_xy + pdf_yz + pdf_zx) * intersects.len() as f32;

        (sample_p, sample_normal, sp / pdf)
    }
}
