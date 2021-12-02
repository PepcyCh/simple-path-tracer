use crate::core::{color::Color, loader::InputParams, rng::Rng, scene_resources::SceneResources};

use super::{util, MediumT};

pub struct Homogeneous {
    sigma_t: Color,
    sigma_s: Color,
    asymmetric: f32,
}

impl Homogeneous {
    pub fn new(sigma_a: Color, sigma_s: Color, asymmetric: f32) -> Self {
        let sigma_t = sigma_a + sigma_s;
        Self {
            sigma_t,
            sigma_s,
            asymmetric,
        }
    }

    pub fn load(_rsc: &SceneResources, params: &mut InputParams) -> anyhow::Result<Self> {
        let sigma_a = params.get_float3("sigma_a")?.into();
        let sigma_s = params.get_float3("sigma_a")?.into();
        let asymmetric = params.get_float("asymmetric")?;

        Ok(Self::new(sigma_a, sigma_s, asymmetric))
    }
}

impl MediumT for Homogeneous {
    fn sample_pi(
        &self,
        po: glam::Vec3A,
        wo: glam::Vec3A,
        t_max: f32,
        rng: &mut Rng,
    ) -> (glam::Vec3A, bool, Color) {
        let (rand_x, rand_y) = rng.uniform_2d();
        let sample_sigma_t = {
            if rand_x < 1.0 / 3.0 {
                self.sigma_t.r
            } else if rand_x < 2.0 / 3.0 {
                self.sigma_t.g
            } else {
                self.sigma_t.b
            }
        };
        let sample_t = -(1.0 - rand_y).ln() / sample_sigma_t;

        let attenuation = (-self.sigma_t * sample_t.min(t_max)).exp();
        let pi = po - wo * sample_t.min(t_max);

        if sample_t < t_max {
            let atten_pdf = (self.sigma_t * attenuation).avg();
            (pi, true, attenuation * self.sigma_s / atten_pdf)
        } else {
            let atten_pdf = attenuation.avg();
            (pi, false, attenuation / atten_pdf)
        }
    }

    fn sample_wi(&self, wo: glam::Vec3A, rng: &mut Rng) -> (glam::Vec3A, f32) {
        let (rand_x, rand_y) = rng.uniform_2d();
        let cos_theta = util::henyey_greenstein_cdf_inverse(self.asymmetric, rand_x);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
        let phi = 2.0 * std::f32::consts::PI * rand_y;
        let wi = glam::Vec3A::new(sin_theta * phi.cos(), sin_theta * phi.sin(), cos_theta);
        let wi = util::local_to_world(wo, wi);

        (wi, util::henyey_greenstein(self.asymmetric, cos_theta))
    }

    fn transport_attenuation(&self, dist: f32) -> Color {
        (-self.sigma_t * dist).exp()
    }

    fn phase(&self, wo: glam::Vec3A, wi: glam::Vec3A) -> f32 {
        let cos = wo.dot(wi);
        util::henyey_greenstein(self.asymmetric, cos)
    }
}
