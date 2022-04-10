use crate::core::rng::Rng;

use super::{util, BxdfInputs, PndfAccel, PndfGaussTerm};

#[enum_dispatch::enum_dispatch(Microfacet)]
pub trait MicrofacetT {
    fn sample_half<'a>(&self, inputs: &'a BxdfInputs, rng: &mut Rng) -> (glam::Vec3A, f32);

    fn half_pdf(&self, wo: glam::Vec3A, half: glam::Vec3A) -> f32;

    fn ndf_visible(&self, wo: glam::Vec3A, wi: glam::Vec3A, half: glam::Vec3A) -> f32;
}

#[enum_dispatch::enum_dispatch]
pub enum Microfacet {
    GgxMicrofacet,
    PndfMicrofacet,
}

pub struct GgxMicrofacet {
    roughness_x: f32,
    roughness_y: f32,
}

impl GgxMicrofacet {
    pub fn new(roughness_x: f32, roughness_y: f32) -> Self {
        Self {
            roughness_x,
            roughness_y,
        }
    }
}

impl MicrofacetT for GgxMicrofacet {
    fn sample_half<'a>(&self, inputs: &'a BxdfInputs, rng: &mut Rng) -> (glam::Vec3A, f32) {
        util::ggx_smith_vndf_sample(
            inputs.wo,
            self.roughness_x,
            self.roughness_y,
            rng.uniform_2d(),
        )
    }

    fn half_pdf(&self, wo: glam::Vec3A, half: glam::Vec3A) -> f32 {
        util::ggx_smith_vndf_pdf(half, wo, self.roughness_x, self.roughness_y)
    }

    fn ndf_visible(&self, wo: glam::Vec3A, wi: glam::Vec3A, half: glam::Vec3A) -> f32 {
        let ndf = util::ggx_ndf_aniso(half, self.roughness_x, self.roughness_y);
        let visible =
            util::smith_separable_visible_aniso(wo, wi, self.roughness_x, self.roughness_y);
        ndf * visible
    }
}

pub struct PndfMicrofacet {
    u: glam::Vec2,
    sigma_p: f32,
    sigma_hx: f32,
    sigma_hy: f32,
    sigma_r: f32,
    terms: Vec<(PndfGaussTerm, f32)>,
    term_coe: f32,
    bvh: *const PndfAccel,
}

impl PndfMicrofacet {
    pub fn new(
        u: glam::Vec2,
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

impl MicrofacetT for PndfMicrofacet {
    fn sample_half<'a>(&self, inputs: &'a BxdfInputs, rng: &mut Rng) -> (glam::Vec3A, f32) {
        let bvh = unsafe { self.bvh.as_ref().unwrap() };

        let sigma_p_sqr = self.sigma_p * self.sigma_p;
        let sigma_p_sqr_inv = 1.0 / sigma_p_sqr;
        let sigma_h_sqr = self.sigma_hx * self.sigma_hy;
        let sigma_h_sqr_inv = 1.0 / sigma_h_sqr;
        let sigma_sqr_sum_inv = 1.0 / (sigma_p_sqr + sigma_h_sqr);

        let mut rand = rng.uniform_1d();
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
        let u = rng.gaussian_2d(0.0, sigma);
        let u = glam::Vec2::new(mu.x + u.0, mu.y + u.1);

        let s_mu = gaussian.s + gaussian.jacobian * (u - gaussian.u);
        let s = rng.gaussian_2d(0.0, self.sigma_r);
        let s = s_mu + glam::Vec2::new(s.0, s.1);
        let half = glam::Vec3A::new(s.x, s.y, (1.0 - s.length_squared()).clamp(0.0, 1.0).sqrt())
            .normalize();

        let pdf = bvh.calc(
            self.sigma_p,
            self.sigma_hx,
            self.sigma_hy,
            self.sigma_r,
            self.term_coe,
            self.u,
            s,
        );

        (half, pdf)
    }

    fn half_pdf(&self, wo: glam::Vec3A, half: glam::Vec3A) -> f32 {
        let bvh = unsafe { self.bvh.as_ref().unwrap() };
        let s = glam::Vec2::new(half.x, half.y);
        bvh.calc(
            self.sigma_p,
            self.sigma_hx,
            self.sigma_hy,
            self.sigma_r,
            self.term_coe,
            self.u,
            s,
        )
    }

    fn ndf_visible(&self, wo: glam::Vec3A, wi: glam::Vec3A, half: glam::Vec3A) -> f32 {
        let bvh = unsafe { self.bvh.as_ref().unwrap() };
        let s = glam::Vec2::new(half.x, half.y);
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
        pndf / half.z.max(0.0001) * visible
    }
}
