use std::sync::Arc;

use crate::{
    core::{alias_table::AliasTable, rng::Rng},
    light::{Light, LightT},
};

use super::LightSamplerT;

pub struct PowerIsLightSampler {
    lights: Vec<Arc<Light>>,
    alias_table: AliasTable,
}

impl PowerIsLightSampler {
    pub fn new(lights: Vec<Arc<Light>>) -> Self {
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
        }
    }
}

impl LightSamplerT for PowerIsLightSampler {
    fn sample_light(&self, rng: &mut Rng) -> (&Light, f32) {
        let (index, pdf) = self.alias_table.sample(rng.uniform_1d());
        (self.lights[index].as_ref(), pdf)
    }
}
