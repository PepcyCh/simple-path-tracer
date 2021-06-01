use crate::core::color::Color;
use crate::core::sampler::Sampler;
use crate::core::scatter::{Reflect, Scatter, ScatterType};
use crate::scatter::{PndfAccel, PndfGaussTerm};
use cgmath::{InnerSpace, Point3, Vector2, Vector3};

pub struct PndfReflect {
    albedo: Color,
    u: Vector2<f32>,
    sigma_p: f32,
    sigma_hx: f32,
    sigma_hy: f32,
    sigma_r: f32,
    terms: Vec<(PndfGaussTerm, f32)>,
    term_coe: f32,
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
        let bvh_ref = unsafe { bvh.as_ref().unwrap() };
        let (mut terms, sum) = bvh_ref.find_terms(u, sigma_p, sigma_hx, sigma_hy);
        let sum_inv = 1.0 / sum;
        let term_coe = sum_inv / (2.0 * std::f32::consts::PI * sigma_r * sigma_r);
        for (_, value) in &mut terms {
            *value *= sum_inv;
        }

        Self {
            albedo,
            u,
            sigma_p,
            sigma_hx,
            sigma_hy,
            sigma_r,
            terms,
            term_coe,
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

        let sigma_p_sqr = self.sigma_p * self.sigma_p;
        let sigma_p_sqr_inv = 1.0 / sigma_p_sqr;
        let sigma_h_sqr = self.sigma_hx * self.sigma_hy;
        let sigma_h_sqr_inv = 1.0 / sigma_h_sqr;
        let sigma_sqr_sum_inv = 1.0 / (sigma_p_sqr + sigma_h_sqr);

        let mut rand = sampler.uniform_1d();
        let mut gaussian = self.terms.last().unwrap().0;
        for (term, value) in &self.terms {
            rand -= value;
            if rand <= 0.0 {
                gaussian = *term;
                break;
            }
        }

        let mu = sigma_sqr_sum_inv * (sigma_h_sqr * self.u + sigma_p_sqr * gaussian.u);
        let sigma = 1.0 / (sigma_p_sqr_inv + sigma_h_sqr_inv).sqrt();
        let u = sampler.gaussian_2d(0.0, sigma);
        let u = Vector2::new(mu.x + u.0, mu.y + u.1);

        let s_mu = gaussian.s + gaussian.jacobian * (u - gaussian.u);
        let s = sampler.gaussian_2d(0.0, self.sigma_r);
        let s = s_mu + Vector2::new(s.0, s.1);
        let half =
            Vector3::new(s.x, s.y, (1.0 - s.magnitude2()).clamp(0.0, 1.0).sqrt()).normalize();

        let wi = crate::scatter::util::reflect_n(wo, half);
        if wo.z * wi.z >= 0.0 {
            let pndf = bvh.calc(
                self.sigma_p,
                self.sigma_hx,
                self.sigma_hy,
                self.sigma_r,
                self.term_coe,
                self.u,
                s,
            );
            let visible = 0.25 / (wi.z * wo.z).max(0.0001);
            // let visible = 0.25;
            let pdf = pndf / (4.0 * wo.dot(half).abs());
            let bxdf = self.albedo * pndf / half.z.max(0.0001) * visible;
            (wi, pdf, bxdf, ScatterType::glossy_reflect())
        } else {
            (wo, 1.0, Color::BLACK, ScatterType::glossy_reflect())
        }
    }

    fn pdf(&self, _po: Point3<f32>, wo: Vector3<f32>, _pi: Point3<f32>, wi: Vector3<f32>) -> f32 {
        if wo.z * wi.z >= 0.0 {
            let bvh = unsafe { self.bvh.as_ref().unwrap() };
            let half = crate::scatter::util::half_from_reflect(wo, wi);
            let s = Vector2::new(half.x, half.y);
            let pndf = bvh.calc(
                self.sigma_p,
                self.sigma_hx,
                self.sigma_hy,
                self.sigma_r,
                self.term_coe,
                self.u,
                s,
            );
            pndf / (4.0 * wo.dot(half).abs())
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
        if wo.z * wi.z >= 0.0 {
            let bvh = unsafe { self.bvh.as_ref().unwrap() };
            let half = crate::scatter::util::half_from_reflect(wo, wi);
            let s = Vector2::new(half.x, half.y);
            let pndf = bvh.calc(
                self.sigma_p,
                self.sigma_hx,
                self.sigma_hy,
                self.sigma_r,
                self.term_coe,
                self.u,
                s,
            );
            let visible = 0.25 / (wi.z * wo.z).max(0.0001);
            // let visible = 0.25;
            self.albedo * pndf / half.z.max(0.0001) * visible
        } else {
            Color::BLACK
        }
    }

    fn is_delta(&self) -> bool {
        false
    }
}

impl Reflect for PndfReflect {}
