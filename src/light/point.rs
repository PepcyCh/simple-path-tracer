use crate::core::{color::Color, loader::InputParams, rng::Rng, scene_resources::SceneResources};

use super::LightT;

pub struct PointLight {
    position: glam::Vec3A,
    strength: Color,
}

impl PointLight {
    pub fn new(position: glam::Vec3A, strength: Color) -> Self {
        Self { position, strength }
    }

    pub fn load(_rsc: &SceneResources, params: &mut InputParams) -> anyhow::Result<Self> {
        let position = params.get_float3("position")?.into();
        let strength = params.get_float3("strength")?.into();

        Ok(Self::new(position, strength))
    }
}

impl LightT for PointLight {
    fn sample(&self, position: glam::Vec3A, _rng: &mut Rng) -> (glam::Vec3A, f32, Color, f32) {
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

    fn power(&self) -> f32 {
        self.strength.luminance()
    }
}
