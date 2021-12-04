use std::sync::Arc;

use crate::{core::rng::Rng, light::Light};

use super::LightSamplerT;

pub struct UniformLightSampler {
    lights: Vec<Arc<Light>>,
    num_light_inv: f32,
}

impl UniformLightSampler {
    pub fn new(lights: Vec<Arc<Light>>) -> Self {
        let num_light_inv = 1.0 / lights.len() as f32;
        Self {
            lights,
            num_light_inv,
        }
    }
}

impl LightSamplerT for UniformLightSampler {
    fn sample_light(&self, rng: &mut Rng) -> (&Light, f32) {
        let index = rng.uniform_1d() * self.lights.len() as f32;
        let index = (index as usize).min(self.lights.len() - 1);
        (self.lights[index].as_ref(), self.num_light_inv)
    }

    fn num_lights(&self) -> usize {
        self.lights.len()
    }
}
