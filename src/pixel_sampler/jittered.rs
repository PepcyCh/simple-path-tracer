use crate::core::{loader::InputParams, rng::Rng};

use super::PixelSamplerT;

#[derive(Clone, Copy)]
pub struct JitteredSampler {
    division_x: u32,
    division_y: u32,
    division_x_inv: f32,
    division_y_inv: f32,
    curr_ind_x: u32,
    curr_ind_y: u32,
}

impl JitteredSampler {
    pub fn new(division_x: u32, division_y: u32) -> Self {
        Self {
            division_x,
            division_y,
            division_x_inv: 1.0 / division_x as f32,
            division_y_inv: 1.0 / division_y as f32,
            curr_ind_x: 0,
            curr_ind_y: 0,
        }
    }

    pub fn load(params: &mut InputParams) -> anyhow::Result<Self> {
        let division_x = params.get_int("division_x")? as u32;
        let division_y = params.get_int("division_y")? as u32;
        Ok(Self::new(division_x, division_y))
    }
}

impl PixelSamplerT for JitteredSampler {
    fn spp(&self) -> u32 {
        self.division_x * self.division_y
    }

    fn start_pixel(&mut self) {
        self.curr_ind_x = 0;
        self.curr_ind_y = 0;
    }

    fn next_sample(&mut self, rng: &mut Rng) -> Option<(f32, f32)> {
        if self.curr_ind_x == self.division_x && self.curr_ind_y == self.division_y {
            None
        } else {
            let rand_x = (self.curr_ind_x as f32 + rng.uniform_1d()) * self.division_x_inv;
            let rand_y = (self.curr_ind_y as f32 + rng.uniform_1d()) * self.division_y_inv;
            self.curr_ind_x += 1;
            if self.curr_ind_x == self.division_x {
                self.curr_ind_x = 0;
                self.curr_ind_y += 1;
            }
            Some((rand_x, rand_y))
        }
    }
}
