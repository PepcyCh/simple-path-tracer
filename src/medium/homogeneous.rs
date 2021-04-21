use crate::core::color::Color;
use crate::core::medium::Medium;
use crate::core::sampler::Sampler;
use cgmath::{InnerSpace, Point3, Vector3};

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
}

impl Medium for Homogeneous {
    fn sample_transport(
        &self,
        position: Point3<f32>,
        wo: Vector3<f32>,
        t_max: f32,
        sampler: &mut dyn Sampler,
    ) -> (Point3<f32>, bool, Color) {
        let (rand_x, rand_y) = sampler.uniform_2d();
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
        let sample_position = position - wo * sample_t.min(t_max);

        if sample_t < t_max {
            let atten_pdf = (self.sigma_t * attenuation).avg();
            (
                sample_position,
                true,
                attenuation * self.sigma_s / atten_pdf,
            )
        } else {
            let atten_pdf = attenuation.avg();
            (sample_position, false, attenuation / atten_pdf)
        }
    }

    fn sample_phase(&self, wo: Vector3<f32>, sampler: &mut dyn Sampler) -> (Vector3<f32>, f32) {
        let (rand_x, rand_y) = sampler.uniform_2d();
        let cos_theta = crate::medium::util::henyey_greenstein_cdf_inverse(self.asymmetric, rand_x);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
        let phi = 2.0 * std::f32::consts::PI * rand_y;
        let wi = Vector3::new(sin_theta * phi.cos(), sin_theta * phi.sin(), cos_theta);
        let sample_direction = crate::medium::util::local_to_world(wo, wi);

        (
            sample_direction,
            crate::medium::util::henyey_greenstein(self.asymmetric, cos_theta),
        )
    }

    fn transport_attenuation(&self, dist: f32) -> Color {
        (-self.sigma_t * dist).exp()
    }

    fn phase(&self, wo: Vector3<f32>, wi: Vector3<f32>) -> f32 {
        let cos = wo.dot(wi);
        crate::medium::util::henyey_greenstein(self.asymmetric, cos)
    }
}
