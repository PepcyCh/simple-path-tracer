use crate::core::{color::Color, loader::InputParams, rng::Rng, scene_resources::SceneResources};

use super::LightT;

pub struct DirLight {
    direction: glam::Vec3A,
    strength: Color,
}

impl DirLight {
    pub fn new(direction: glam::Vec3A, strength: Color) -> Self {
        Self {
            direction: direction.normalize(),
            strength,
        }
    }

    pub fn load(_rsc: &SceneResources, params: &mut InputParams) -> anyhow::Result<Self> {
        let direction = params.get_float3("direction")?.into();
        let strength = params.get_float3("strength")?.into();

        Ok(Self::new(direction, strength))
    }
}

impl LightT for DirLight {
    fn sample(&self, _position: glam::Vec3A, _rng: &mut Rng) -> (glam::Vec3A, f32, Color, f32) {
        (-self.direction, 1.0, self.strength, f32::MAX)
    }

    fn strength_dist_pdf(&self, _position: glam::Vec3A, wi: glam::Vec3A) -> (Color, f32, f32) {
        if wi.dot(self.direction) <= -0.99 {
            (self.strength, f32::MAX, 1.0)
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
