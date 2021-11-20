use rand::{Rng, SeedableRng};

use crate::core::sampler::Sampler;

// http://extremelearning.com.au/unreasonable-effectiveness-of-quasirandom-sequences/
pub struct AdditiveRecurrenceSampler {
    rng: rand::rngs::SmallRng,
    #[allow(dead_code)]
    curr_1d: f32,
    curr_2d_x: f32,
    curr_2d_y: f32,
}

impl AdditiveRecurrenceSampler {
    #[allow(dead_code)]
    const INV_PHI1: f32 = 0.618033988749895;
    const INV_PHI2: f32 = 0.754877666246571;

    pub fn new() -> Self {
        let mut rng = rand::rngs::SmallRng::from_entropy();
        let curr_1d = rng.gen();
        let curr_2d_x = rng.gen();
        let curr_2d_y = rng.gen();
        Self {
            rng,
            curr_1d,
            curr_2d_x,
            curr_2d_y,
        }
    }

    #[allow(dead_code)]
    fn get_1d(&mut self) -> f32 {
        self.curr_1d += Self::INV_PHI1;
        if self.curr_1d >= 1.0 {
            self.curr_1d -= 1.0;
        }
        self.curr_1d
    }

    fn get_2d(&mut self) -> (f32, f32) {
        self.curr_2d_x += Self::INV_PHI2;
        if self.curr_2d_x >= 1.0 {
            self.curr_2d_x -= 1.0;
        }
        self.curr_2d_y += Self::INV_PHI2 * Self::INV_PHI2;
        if self.curr_2d_y >= 1.0 {
            self.curr_2d_y -= 1.0;
        }
        (self.curr_2d_x, self.curr_2d_y)
    }
}

impl Sampler for AdditiveRecurrenceSampler {
    fn uniform_1d(&mut self) -> f32 {
        // self.get_1d()
        self.rng.gen()
    }

    fn pixel_samples(&mut self, spp: u32) -> Vec<(f32, f32)> {
        let mut samples = Vec::with_capacity(spp as usize);

        for _ in 0..spp {
            samples.push(self.get_2d());
        }

        samples
    }
}
