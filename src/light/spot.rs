use crate::core::{color::Color, loader::InputParams, rng::Rng, scene_resources::SceneResources};

use super::LightT;

pub struct SpotLight {
    position: glam::Vec3A,
    direction: glam::Vec3A,
    cos_inner_angle: f32,
    cos_outer_angle: f32,
    strength: Color,
}

impl SpotLight {
    pub fn new(
        position: glam::Vec3A,
        direction: glam::Vec3A,
        inner_angle: f32,
        outer_angle: f32,
        strength: Color,
    ) -> Self {
        let cos_inner_angle = inner_angle.cos();
        let cos_outer_angle = outer_angle.cos();
        Self {
            position,
            direction,
            cos_inner_angle,
            cos_outer_angle,
            strength,
        }
    }

    pub fn load(_rsc: &SceneResources, params: &mut InputParams) -> anyhow::Result<Self> {
        let position = params.get_float3("position")?.into();
        let direction = params.get_float3("direction")?.into();
        let inner_angle_deg = params.get_float_or("inner_angle", 0.0);
        let inner_angle = inner_angle_deg * std::f32::consts::PI / 180.0;
        let outer_angle_deg = params.get_float_or("outer_angle", 90.0);
        let outer_angle = outer_angle_deg * std::f32::consts::PI / 180.0;
        let strength = params.get_float3("strength")?.into();

        Ok(Self::new(
            position,
            direction,
            inner_angle,
            outer_angle,
            strength,
        ))
    }

    fn strength(&self, wi: glam::Vec3A) -> Color {
        let atten = ((self.direction.dot(-wi) - self.cos_outer_angle)
            / (self.cos_inner_angle - self.cos_outer_angle).max(0.0001))
        .clamp(0.0, 1.0);
        self.strength * atten
    }
}

impl LightT for SpotLight {
    fn sample(&self, position: glam::Vec3A, _rng: &mut Rng) -> (glam::Vec3A, f32, Color, f32) {
        let sample = self.position - position;
        let dist_sqr = sample.length_squared();
        let dist = dist_sqr.sqrt();
        let sample = sample / dist;
        (sample, 1.0, self.strength(sample) / dist_sqr, dist)
    }

    fn strength_dist_pdf(&self, position: glam::Vec3A, wi: glam::Vec3A) -> (Color, f32, f32) {
        let dir = self.position - position;
        let dist_sqr = dir.length_squared();
        let dist = dist_sqr.sqrt();
        let dir = dir / dist;
        if dir.dot(wi) >= 0.99 {
            (self.strength(dir) / dist_sqr, dist, 1.0)
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
