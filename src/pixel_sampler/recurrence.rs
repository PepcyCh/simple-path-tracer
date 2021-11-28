use crate::core::loader::InputParams;

use super::PixelSamplerT;

// http://extremelearning.com.au/unreasonable-effectiveness-of-quasirandom-sequences/
#[derive(Clone, Copy)]
pub struct AdditiveRecurrenceSampler {
    spp: u32,
    curr_index: u32,
    curr_2d_x: f32,
    curr_2d_y: f32,
}

impl AdditiveRecurrenceSampler {
    const INV_PHI2: f32 = 0.754877666246571;

    pub fn new(spp: u32) -> Self {
        Self {
            spp,
            curr_index: 0,
            curr_2d_x: 0.5,
            curr_2d_y: 0.5,
        }
    }

    pub fn load(params: &mut InputParams) -> anyhow::Result<Self> {
        let spp = params.get_int("spp")? as u32;
        Ok(Self::new(spp))
    }
}

impl PixelSamplerT for AdditiveRecurrenceSampler {
    fn spp(&self) -> u32 {
        self.spp
    }

    fn start_pixel(&mut self) {
        self.curr_index = 0;
    }

    fn next_sample(&mut self, _rng: &mut crate::core::rng::Rng) -> Option<(f32, f32)> {
        if self.curr_index < self.spp {
            self.curr_index += 1;
            self.curr_2d_x += Self::INV_PHI2;
            if self.curr_2d_x >= 1.0 {
                self.curr_2d_x -= 1.0;
            }
            self.curr_2d_y += Self::INV_PHI2 * Self::INV_PHI2;
            if self.curr_2d_y >= 1.0 {
                self.curr_2d_y -= 1.0;
            }
            Some((self.curr_2d_x, self.curr_2d_y))
        } else {
            None
        }
    }
}
