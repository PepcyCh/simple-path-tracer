use std::{collections::HashMap, sync::Arc};

use crate::{
    core::{alias_table::AliasTable, color::Color, intersection::Intersection, rng::Rng},
    light::{Light, LightT},
    primitive::{Instance, InstancePtr, PrimitiveT},
};

use super::{LightSamplerInputs, LightSamplerT};

pub struct PowerIsLightSampler {
    lights: Vec<Arc<Light>>,
    alias_table: AliasTable,
    env_light_index: Option<usize>,
    instance_light_map: HashMap<InstancePtr, usize>,
}

impl PowerIsLightSampler {
    pub fn new(
        lights: Vec<Arc<Light>>,
        env_light_index: Option<usize>,
        instance_light_map: HashMap<InstancePtr, usize>,
    ) -> Self {
        let mut props = vec![0.0; lights.len()];
        let mut sum = 0.0;
        for (i, light) in lights.iter().enumerate() {
            props[i] = light.power();
            sum += props[i];
        }
        let sum_inv = 1.0 / sum;
        for prop in &mut props {
            *prop *= sum_inv;
        }
        let alias_table = AliasTable::new(props);

        Self {
            lights,
            alias_table,
            env_light_index,
            instance_light_map,
        }
    }
}

impl LightSamplerT for PowerIsLightSampler {
    fn sample_light(
        &self,
        inputs: &LightSamplerInputs,
        rng: &mut Rng,
    ) -> (glam::Vec3A, f32, Color, f32, bool) {
        let (index, pdf) = self.alias_table.sample(rng.uniform_1d());
        let light = self.lights[index].as_ref();
        let (dir, light_pdf, strength, dist) = light.sample(inputs.position, rng);
        let pdf = pdf * light_pdf;
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

        let instance_ptr = InstancePtr(instance as *const _);
        let light_index = self.instance_light_map[&instance_ptr];
        let alias_table_pdf = self.alias_table.probability(light_index);

        local_pdf * alias_table_pdf
    }

    fn pdf_env_light(&self, _inputs: &LightSamplerInputs) -> f32 {
        if let Some(env_light_index) = self.env_light_index {
            self.alias_table.probability(env_light_index)
        } else {
            1.0
        }
    }

    fn num_lights(&self) -> usize {
        self.lights.len()
    }
}
