use std::sync::Arc;

use crate::{
    core::{color::Color, intersection::Intersection, rng::Rng},
    light::{Light, LightT},
    primitive::{Instance, PrimitiveT},
};

use super::{LightSamplerInputs, LightSamplerT};

pub struct UniformLightSampler {
    lights: Vec<Arc<Light>>,
    num_light_inv: f32,
    env_light_index: Option<usize>,
}

impl UniformLightSampler {
    pub fn new(lights: Vec<Arc<Light>>, env_light_index: Option<usize>) -> Self {
        let num_light_inv = 1.0 / lights.len() as f32;
        Self {
            lights,
            num_light_inv,
            env_light_index,
        }
    }
}

impl LightSamplerT for UniformLightSampler {
    fn sample_light(
        &self,
        inputs: &LightSamplerInputs,
        rng: &mut Rng,
    ) -> (glam::Vec3A, f32, Color, f32, bool) {
        let index = rng.uniform_1d() * self.lights.len() as f32;
        let index = (index as usize).min(self.lights.len() - 1);
        let light = self.lights[index].as_ref();
        let (dir, pdf, strength, dist) = light.sample(inputs.position, rng);
        let pdf = pdf * self.num_light_inv;
        (dir, pdf, strength, dist, light.is_delta())
    }

    fn pdf_shape_light(
        &self,
        inputs: &LightSamplerInputs,
        instance: &Instance,
        inter: &Intersection,
    ) -> f32 {
        let primitive_pdf = instance.pdf(inter);

        let light_vec = inter.position - inputs.position;
        let light_dist_sqr = light_vec.length_squared();
        let light_dir = light_vec / light_dist_sqr.sqrt();

        let cos = if inter.surface.unwrap().double_sided() {
            light_dir.dot(inter.normal).abs()
        } else {
            let cos = light_dir.dot(-inter.normal);
            if cos > 0.0 {
                cos
            } else {
                1.0
            }
        };

        let local_pdf = primitive_pdf * light_dist_sqr / cos.max(0.00001);

        local_pdf * self.num_light_inv
    }

    fn pdf_env_light(&self, _inputs: &LightSamplerInputs) -> f32 {
        if let Some(_) = self.env_light_index {
            self.num_light_inv
        } else {
            1.0
        }
    }

    fn num_lights(&self) -> usize {
        self.lights.len()
    }
}
