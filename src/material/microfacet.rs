use crate::core::color::Color;
use crate::core::material::Material;
use crate::core::sampler::Sampler;
use cgmath::{InnerSpace, Vector3};
use std::sync::Mutex;

pub struct Microfacet {
    albedo: Color,
    emissive: Color,
    _roughness: f32,
    roughness_sqr: f32,
    metallic: f32,
    sampler: Box<Mutex<dyn Sampler>>,
}

impl Microfacet {
    const DIELECTRIC_FRESNEL_R0: Color = Color {
        r: 0.04,
        g: 0.04,
        b: 0.04,
    };

    pub fn new(
        albedo: Color,
        emissive: Color,
        roughness: f32,
        metallic: f32,
        sampler: Box<Mutex<dyn Sampler>>,
    ) -> Self {
        Self {
            albedo,
            emissive,
            _roughness: roughness,
            roughness_sqr: roughness * roughness,
            metallic,
            sampler,
        }
    }
}

impl Material for Microfacet {
    fn sample(&self, wo: Vector3<f32>) -> (Vector3<f32>, f32, Color) {
        let sample = {
            let mut sampler = self.sampler.lock().unwrap();
            sampler.cosine_weighted_on_hemisphere()
        };
        let bsdf = self.bsdf(wo, sample);
        (sample, sample.z / std::f32::consts::PI, bsdf)
    }

    fn bsdf(&self, wo: Vector3<f32>, wi: Vector3<f32>) -> Color {
        let half = (wo + wi).normalize();
        let ndoth = half.z;
        let ndotl = wi.z;
        let ndotv = wo.z;
        let vdoth = wo.dot(half);
        let ndf = crate::material::util::ggx_ndf(ndoth, self.roughness_sqr);
        let visible =
            crate::material::util::smith_separable_visible(ndotv, ndotl, self.roughness_sqr);
        let dielectric_fresnel =
            crate::material::util::schlick_fresnel_with_r0(Self::DIELECTRIC_FRESNEL_R0, vdoth);
        let metal_fresnel = crate::material::util::schlick_fresnel_with_r0(self.albedo, vdoth);
        let dielectric_shading = dielectric_fresnel * ndf * visible
            + (Color::WHITE - dielectric_fresnel) * self.albedo / std::f32::consts::PI;
        let metal_shading = metal_fresnel * ndf * visible;
        (1.0 - self.metallic) * dielectric_shading + self.metallic * metal_shading
    }

    fn pdf(&self, _wo: Vector3<f32>, wi: Vector3<f32>) -> f32 {
        wi.z / std::f32::consts::PI
    }

    fn is_delta(&self) -> bool {
        false
    }

    fn emissive(&self) -> Color {
        self.emissive
    }
}
