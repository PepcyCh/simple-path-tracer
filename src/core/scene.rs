use std::{collections::HashMap, sync::Arc};

use anyhow::Context;

use crate::{camera::Camera, light::Light, light_sampler::LightSampler, primitive::Primitive};

pub struct Scene {
    cameras: HashMap<String, Arc<Camera>>,
    aggregate: Primitive,
    light_sampler: LightSampler,
    environment: Option<Arc<Light>>,
}

impl Scene {
    pub fn new(
        cameras: HashMap<String, Arc<Camera>>,
        aggregate: Primitive,
        light_sampler: LightSampler,
        environment: Option<Arc<Light>>,
    ) -> Self {
        Self {
            cameras,
            aggregate,
            light_sampler,
            environment,
        }
    }

    pub fn get_camera(&self, name: &Option<String>) -> Arc<Camera> {
        if let Some(name) = name {
            self.cameras
                .get(name)
                .context(format!("There is no camera names {}", name))
                .unwrap()
                .clone()
        } else if self.cameras.len() == 1 {
            self.cameras.values().next().unwrap().clone()
        } else {
            panic!("There are multiple cameras so a name must be given");
        }
    }

    pub fn aggregate(&self) -> &Primitive {
        &self.aggregate
    }

    pub fn light_sampler(&self) -> &LightSampler {
        &self.light_sampler
    }

    pub fn environment(&self) -> Option<&Light> {
        self.environment.as_ref().map(|env| env.as_ref())
    }
}
