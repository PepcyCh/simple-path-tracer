mod jittered;
mod random;

pub use jittered::*;
pub use random::*;

use crate::core::sampler::Sampler;

pub fn sampler_from(ty: &str, sample_counts: u32) -> Box<dyn Sampler> {
    match ty {
        "random" => Box::new(RandomSampler::new()) as Box<dyn Sampler>,
        "jittered" => Box::new(JitteredSampler::new(sample_counts)) as Box<dyn Sampler>,
        _ => panic!("Unknown sampler type '{}'", ty),
    }
}
