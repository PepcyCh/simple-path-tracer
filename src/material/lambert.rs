use crate::core::color::Color;
use crate::core::material::Material;
use crate::core::sampler::Sampler;
use cgmath::Vector3;
use std::sync::Mutex;

pub struct Lambert {
    albedo: Color,
    emissive: Color,
    sampler: Box<Mutex<dyn Sampler>>,
}

impl Lambert {
    pub fn new(albedo: Color, emissive: Color, sampler: Box<Mutex<dyn Sampler>>) -> Self {
        Self {
            albedo,
            emissive,
            sampler,
        }
    }
}

impl Material for Lambert {
    fn sample(&self, _wo: Vector3<f32>) -> (Vector3<f32>, f32, Color) {
        let sample = {
            let mut sampler = self.sampler.lock().unwrap();
            sampler.cosine_weighted_on_hemisphere()
        };
        (
            sample,
            sample.z / std::f32::consts::PI,
            self.albedo / std::f32::consts::PI,
        )
    }

    fn bsdf(&self, _wo: Vector3<f32>, _wi: Vector3<f32>) -> Color {
        self.albedo / std::f32::consts::PI
    }

    fn pdf(&self, _wo: Vector3<f32>, wi: Vector3<f32>) -> f32 {
        wi.z.max(0.0) / std::f32::consts::PI
    }

    fn is_delta(&self) -> bool {
        false
    }

    fn emissive(&self) -> Color {
        self.emissive
    }
}
