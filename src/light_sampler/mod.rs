mod power_is;
mod uniform;

pub use power_is::*;
pub use uniform::*;

use crate::{
    core::{color::Color, intersection::Intersection, rng::Rng},
    primitive::Instance,
};

pub struct LightSamplerInputs {
    pub position: glam::Vec3A,
    pub normal: glam::Vec3A,
}

#[enum_dispatch::enum_dispatch(LightSampler)]
pub trait LightSamplerT: Send + Sync {
    fn sample_light(
        &self,
        inputs: &LightSamplerInputs,
        rng: &mut Rng,
    ) -> (glam::Vec3A, f32, Color, f32, bool);

    fn pdf_shape_light(
        &self,
        inputs: &LightSamplerInputs,
        instance: &Instance,
        inter: &Intersection,
    ) -> f32;

    fn pdf_env_light(&self, inputs: &LightSamplerInputs) -> f32;

    fn num_lights(&self) -> usize;
}

#[enum_dispatch::enum_dispatch]
pub enum LightSampler {
    UniformLightSampler,
    PowerIsLightSampler,
}
