mod jittered;
mod random;
mod recurrence;

pub use jittered::*;
pub use random::*;
pub use recurrence::*;

use crate::core::{loader::InputParams, rng::Rng};

#[enum_dispatch::enum_dispatch(PixelSampler)]
pub trait PixelSamplerT: Send + Sync + Clone + Copy {
    fn spp(&self) -> u32;

    fn start_pixel(&mut self);

    fn next_sample(&mut self, rng: &mut Rng) -> Option<(f32, f32)>;
}

#[enum_dispatch::enum_dispatch]
#[derive(Clone, Copy)]
pub enum PixelSampler {
    RandomSampler,
    JitteredSampler,
    AdditiveRecurrenceSampler,
}

pub fn create_sampler_from_params(params: &mut InputParams) -> anyhow::Result<PixelSampler> {
    params.set_name("sampler".into());
    let ty = params.get_str("type")?;
    params.set_name(format!("sampler-{}", ty).into());

    let res = match ty.as_str() {
        "random" => RandomSampler::load(params)?.into(),
        "jittered" => JitteredSampler::load(params)?.into(),
        "recurrence" => AdditiveRecurrenceSampler::load(params)?.into(),
        _ => anyhow::bail!(format!("{}: unknown type '{}'", params.name(), ty)),
    };

    params.check_unused_keys();

    Ok(res)
}
