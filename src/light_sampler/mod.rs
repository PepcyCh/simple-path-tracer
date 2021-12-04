mod power_is;
mod uniform;

pub use power_is::*;
pub use uniform::*;

use crate::{core::rng::Rng, light::Light};

#[enum_dispatch::enum_dispatch(LightSampler)]
pub trait LightSamplerT: Send + Sync {
    fn sample_light(&self, rng: &mut Rng) -> (&Light, f32);

    fn num_lights(&self) -> usize;
}

#[enum_dispatch::enum_dispatch]
pub enum LightSampler {
    UniformLightSampler,
    PowerIsLightSampler,
}
