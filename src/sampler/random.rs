use crate::core::sampler::Sampler;
use rand::{Rng, SeedableRng};

pub struct RandomSampler {
    rng: rand::rngs::SmallRng,
}

impl RandomSampler {
    pub fn new() -> Self {
        Self {
            rng: rand::rngs::SmallRng::from_entropy(),
        }
    }
}

impl Sampler for RandomSampler {
    fn uniform_1d(&mut self) -> f32 {
        self.rng.gen()
    }
}
