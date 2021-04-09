use crate::core::color::Color;
use crate::core::medium::Medium;
use crate::core::ray::Ray;
use crate::core::sampler::Sampler;
use cgmath::{Point3, Vector3};
use std::cell::RefCell;

pub struct Homogeneous {
    sigma_t: Color,
    sigma_s: Color,
    sampler: Box<RefCell<dyn Sampler>>,
}

impl Homogeneous {
    pub fn new(sigma_t: Color, sigma_s: Color, sampler: Box<RefCell<dyn Sampler>>) -> Self {
        Self {
            sigma_t,
            sigma_s,
            sampler,
        }
    }
}

impl Medium for Homogeneous {
    fn sample(
        &self,
        position: Point3<f32>,
        wo: Vector3<f32>,
        t_max: f32,
    ) -> (Point3<f32>, Vector3<f32>, bool, f32, Color) {
        let sample_sigma_t = {
            let rand = self.sampler.borrow_mut().uniform_1d();
            if rand <= 0.33 {
                self.sigma_t.r
            } else if rand <= 0.66 {
                self.sigma_t.g
            } else {
                self.sigma_t.b
            }
        };
        let sample_t = -(1.0 - self.sampler.borrow_mut().uniform_1d()).ln() / sample_sigma_t;
        let attenuation = (self.sigma_t * sample_t.min(t_max)).exp();
        let sample_position = position - wo * sample_t.min(t_max - Ray::T_MIN_EPS * 2.0);
        if sample_t < t_max {
            let sample_direction = self.sampler.borrow_mut().uniform_on_sphere();
            let density = self.sigma_t * attenuation;
            let atten_pdf = (density.r + density.g + density.b) / 3.0;
            (
                sample_position,
                sample_direction,
                true,
                0.25 / std::f32::consts::PI,
                attenuation * self.sigma_s / atten_pdf,
            )
        } else {
            let sample_direction = -wo;
            let density = attenuation;
            let atten_pdf = (density.r + density.g + density.b) / 3.0;
            (
                sample_position,
                sample_direction,
                false,
                1.0,
                attenuation / atten_pdf,
            )
        }
    }
}
