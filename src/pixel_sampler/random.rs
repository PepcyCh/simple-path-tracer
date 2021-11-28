use crate::core::{loader::InputParams, rng::Rng};

use super::PixelSamplerT;

#[derive(Clone, Copy)]
pub struct RandomSampler {
    spp: u32,
    curr_index: u32,
}

impl RandomSampler {
    pub fn new(spp: u32) -> Self {
        Self { spp, curr_index: 0 }
    }

    pub fn load(params: &mut InputParams) -> anyhow::Result<Self> {
        let spp = params.get_int("spp")? as u32;
        Ok(Self::new(spp))
    }
}

impl PixelSamplerT for RandomSampler {
    fn spp(&self) -> u32 {
        self.spp
    }

    fn start_pixel(&mut self) {
        self.curr_index = 0;
    }

    fn next_sample(&mut self, rng: &mut Rng) -> Option<(f32, f32)> {
        if self.curr_index < self.spp {
            self.curr_index += 1;
            Some((rng.uniform_1d(), rng.uniform_1d()))
        } else {
            None
        }
    }
}
