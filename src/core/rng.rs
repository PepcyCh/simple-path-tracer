use rand::SeedableRng;

pub struct Rng {
    rng: rand::rngs::SmallRng,
}

impl Rng {
    pub fn new() -> Self {
        Self {
            rng: rand::rngs::SmallRng::from_entropy(),
        }
    }

    pub fn uniform_1d(&mut self) -> f32 {
        rand::Rng::gen(&mut self.rng)
    }

    pub fn uniform_2d(&mut self) -> (f32, f32) {
        (self.uniform_1d(), self.uniform_1d())
    }

    #[allow(dead_code)]
    pub fn gaussian_1d(&mut self, mu: f32, sigma: f32) -> f32 {
        self.gaussian_2d(mu, sigma).0
    }

    #[allow(dead_code)]
    pub fn gaussian_2d(&mut self, mu: f32, sigma: f32) -> (f32, f32) {
        let mut rand_xy;
        loop {
            rand_xy = self.uniform_2d();
            if rand_xy.0 > 1e-6 {
                break;
            }
        }

        let mag = sigma * (-2.0 * rand_xy.0.ln()).sqrt();
        let temp = 2.0 * std::f32::consts::PI * rand_xy.1;
        let x = mag * temp.cos() + mu;
        let y = mag * temp.sin() + mu;
        (x, y)
    }

    #[allow(dead_code)]
    pub fn uniform_in_disk(&mut self) -> (f32, f32) {
        loop {
            let (rand_x, rand_y) = self.uniform_2d();
            let x = rand_x * 2.0 - 1.0;
            let y = rand_y * 2.0 - 1.0;
            if x * x + y * y <= 1.0 {
                return (x, y);
            }
        }
    }

    pub fn uniform_on_sphere(&mut self) -> glam::Vec3A {
        let (rand_x, rand_y) = self.uniform_2d();
        let phi = rand_x * 2.0 * std::f32::consts::PI;
        let (sin_phi, cos_phi) = phi.sin_cos();
        let cos_theta = 1.0 - 2.0 * rand_y;
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
        glam::Vec3A::new(sin_theta * cos_phi, sin_theta * sin_phi, cos_theta)
    }

    #[allow(dead_code)]
    pub fn uniform_on_hemisphere(&mut self) -> glam::Vec3A {
        let mut sample = self.uniform_on_sphere();
        sample.z = sample.z.abs();
        sample
    }

    pub fn cosine_weighted_on_hemisphere(&mut self) -> glam::Vec3A {
        let (rand_x, rand_y) = self.uniform_2d();
        let phi = rand_x * 2.0 * std::f32::consts::PI;
        let (sin_phi, cos_phi) = phi.sin_cos();
        let sin_theta_sqr = rand_y;
        let sin_theta = sin_theta_sqr.sqrt();
        let cos_theta = (1.0 - sin_theta_sqr).sqrt();
        glam::Vec3A::new(sin_theta * cos_phi, sin_theta * sin_phi, cos_theta)
    }
}
