use std::sync::Arc;

use crate::{
    core::{color::Color, light::Light, sampler::Sampler, scene::Scene},
    loader::{self, JsonObject, Loadable},
};

pub struct PointLight {
    position: glam::Vec3A,
    strength: Color,
}

impl PointLight {
    pub fn new(position: glam::Vec3A, strength: Color) -> Self {
        Self { position, strength }
    }
}

impl Light for PointLight {
    fn sample(
        &self,
        position: glam::Vec3A,
        _sampler: &mut dyn Sampler,
    ) -> (glam::Vec3A, f32, Color, f32) {
        let sample = self.position - position;
        let dist_sqr = sample.length_squared();
        let dist = dist_sqr.sqrt();
        let sample = sample / dist;
        (sample, 1.0, self.strength / dist_sqr, dist)
    }

    fn strength_dist_pdf(&self, position: glam::Vec3A, wi: glam::Vec3A) -> (Color, f32, f32) {
        let dir = self.position - position;
        let dist_sqr = dir.length_squared();
        let dist = dist_sqr.sqrt();
        let dir = dir / dist;
        if dir.dot(wi) >= 0.99 {
            (self.strength / dist_sqr, dist, 1.0)
        } else {
            (Color::BLACK, f32::MAX, 0.0)
        }
    }

    fn is_delta(&self) -> bool {
        true
    }
}

impl Loadable for PointLight {
    fn load(
        scene: &mut Scene,
        _path: &std::path::PathBuf,
        json_value: &JsonObject,
    ) -> anyhow::Result<()> {
        let env = "light-point";

        let position = loader::get_float_array3_field(json_value, &env, "position")?;
        let strength = loader::get_float_array3_field(json_value, &env, "strength")?;

        let light = PointLight::new(position.into(), strength.into());
        scene.lights.push(Arc::new(light));

        Ok(())
    }
}
