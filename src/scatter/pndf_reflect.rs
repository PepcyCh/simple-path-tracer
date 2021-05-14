use crate::core::color::Color;
use crate::core::sampler::Sampler;
use crate::core::scatter::{Reflect, Scatter, ScatterType};
use crate::scatter::PndfAccel;
use cgmath::{InnerSpace, Point3, Vector2, Vector3};

pub struct PndfReflect {
    albedo: Color,
    u: Vector2<f32>,
    sigma_p: f32,
    sigma_hx: f32,
    sigma_hy: f32,
    sigma_r: f32,
    // TODO - use ptr
    bvh: *const PndfAccel,
}

impl PndfReflect {
    pub fn new(
        albedo: Color,
        u: Vector2<f32>,
        sigma_p: f32,
        sigma_hx: f32,
        sigma_hy: f32,
        sigma_r: f32,
        bvh: *const PndfAccel,
    ) -> Self {
        Self {
            albedo,
            u,
            sigma_p,
            sigma_hx,
            sigma_hy,
            sigma_r,
            bvh,
        }
    }
}

impl Scatter for PndfReflect {
    fn sample_wi(
        &self,
        _po: Point3<f32>,
        wo: Vector3<f32>,
        _pi: Point3<f32>,
        sampler: &mut dyn Sampler,
    ) -> (cgmath::Vector3<f32>, f32, Color, ScatterType) {
        let bvh = unsafe { self.bvh.as_ref().unwrap() };

        let u = sampler.gaussian_2d(0.0, self.sigma_p);
        let u = Vector2::new(u.0 + self.u.x, u.1 + self.u.y);
        let delta_u = sampler.gaussian_2d(0.0, 1.0);
        let term_u = u + Vector2::new(delta_u.0 * self.sigma_hx, delta_u.1 * self.sigma_hy);
        let gaussian = bvh.find_term(term_u);
        let s = sampler.gaussian_2d(0.0, self.sigma_r);
        let s = Vector2::new(s.0 + gaussian.s.x, s.1 + gaussian.s.y);
        let half = Vector3::new(s.x, s.y, (1.0 - s.magnitude2()).clamp(0.0, 1.0).sqrt());

        let wi = crate::scatter::util::reflect_n(wo, half);
        if wo.dot(wi) >= 0.0 {
            let pndf = bvh.calc(
                self.sigma_p,
                self.sigma_hx,
                self.sigma_hy,
                self.sigma_r,
                self.u,
                s,
            );
            let pdf = pndf * half.z / (4.0 * wo.dot(half).abs());
            let bxdf = self.albedo * pndf;
            (wi, pdf, bxdf, ScatterType::glossy_reflect())
        } else {
            (wo, 1.0, Color::BLACK, ScatterType::glossy_reflect())
        }
    }

    fn pdf(&self, _po: Point3<f32>, wo: Vector3<f32>, _pi: Point3<f32>, wi: Vector3<f32>) -> f32 {
        if wo.dot(wi) >= 0.0 {
            let bvh = unsafe { self.bvh.as_ref().unwrap() };
            let half = crate::scatter::util::half_from_reflect(wo, wi);
            let s = Vector2::new(half.x, half.y);
            let pndf = bvh.calc(
                self.sigma_p,
                self.sigma_hx,
                self.sigma_hy,
                self.sigma_r,
                self.u,
                s,
            );
            pndf * half.z / (4.0 * wo.dot(half).abs())
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
        if wo.dot(wi) >= 0.0 {
            let bvh = unsafe { self.bvh.as_ref().unwrap() };
            let half = crate::scatter::util::half_from_reflect(wo, wi);
            let s = Vector2::new(half.x, half.y);
            let pndf = bvh.calc(
                self.sigma_p,
                self.sigma_hx,
                self.sigma_hy,
                self.sigma_r,
                self.u,
                s,
            );
            self.albedo * pndf
        } else {
            Color::BLACK
        }
    }

    fn is_delta(&self) -> bool {
        false
    }
}

impl Reflect for PndfReflect {}
