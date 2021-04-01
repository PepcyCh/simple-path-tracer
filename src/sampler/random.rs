use crate::core::sampler::Sampler;
use rand::Rng;

pub struct RandomSampler {
    rng: rand::rngs::ThreadRng,
}

impl RandomSampler {
    pub fn new() -> Self {
        Self {
            rng: rand::thread_rng(),
        }
    }
}

impl Sampler for RandomSampler {
    fn uniform_1d(&mut self) -> f32 {
        self.rng.gen()
    }
}
