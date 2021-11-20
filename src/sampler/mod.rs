mod jittered;
mod random;
mod recurrence;

use jittered::JitteredSampler;
use random::RandomSampler;
use recurrence::AdditiveRecurrenceSampler;

use crate::core::sampler::Sampler;

pub fn sampler_from(ty: &str, sample_counts: u32) -> Box<dyn Sampler> {
    match ty {
        "random" => Box::new(RandomSampler::new()) as Box<dyn Sampler>,
        "jittered" => Box::new(JitteredSampler::new(sample_counts)) as Box<dyn Sampler>,
        "recurrence" => Box::new(AdditiveRecurrenceSampler::new()) as Box<dyn Sampler>,
        _ => panic!("Unknown sampler type '{}'", ty),
    }
}
