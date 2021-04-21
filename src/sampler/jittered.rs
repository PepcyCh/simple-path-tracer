use crate::core::sampler::Sampler;
use rand::{Rng, SeedableRng};

pub struct JitteredSampler {
    rng: rand::rngs::SmallRng,
    _division: u32,
    division_sqrt: u32,
    division_sqrt_inv: f32,
    curr_ind_x: u32,
    curr_ind_y: u32,
}

impl JitteredSampler {
    pub fn new(division: u32) -> Self {
        let division_sqrt = (division as f32).sqrt() as u32;
        let mut rng = rand::rngs::SmallRng::from_entropy();
        let curr_ind_x = rng.gen_range(0..division_sqrt);
        let curr_ind_y = rng.gen_range(0..division_sqrt);
        Self {
            rng,
            _division: division,
            division_sqrt,
            division_sqrt_inv: 1.0 / division_sqrt as f32,
            curr_ind_x,
            curr_ind_y,
        }
    }

    fn pixel_2d(&mut self) -> (f32, f32) {
        self.curr_ind_x = if self.curr_ind_x + 1 == self.division_sqrt {
            0
        } else {
            self.curr_ind_x + 1
        };
        self.curr_ind_y = if self.curr_ind_y + 1 == self.division_sqrt {
            0
        } else {
            self.curr_ind_y + 1
        };
        let rand_x = (self.curr_ind_x as f32 + self.rng.gen::<f32>()) * self.division_sqrt_inv;
        let rand_y = (self.curr_ind_y as f32 + self.rng.gen::<f32>()) * self.division_sqrt_inv;
        (rand_x, rand_y)
    }
}

impl Sampler for JitteredSampler {
    fn uniform_1d(&mut self) -> f32 {
        // self.curr_ind = if self.curr_ind + 1 == self.division {
        //     0
        // } else {
        //     self.curr_ind + 1
        // };
        // (self.curr_ind as f32 + self.rng.gen::<f32>()) * self.division_inv
        self.rng.gen()
    }

    fn pixel_samples(&mut self, spp: u32) -> Vec<(f32, f32)> {
        let mut samples = Vec::with_capacity(spp as usize);
        for _ in 0..spp {
            samples.push(self.pixel_2d());
        }
        samples
    }
}
