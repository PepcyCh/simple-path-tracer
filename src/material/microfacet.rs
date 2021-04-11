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
        //! pdf: 0.5 * importance sampling of GGX NDF + 0.5 * cosine weighted on hemisphere
        let rand = {
            let mut sampler = self.sampler.lock().unwrap();
            sampler.uniform_1d()
        };
        let (sample_direction, sample_half) = if rand > 0.5 {
            let sample_direction = {
                let mut sampler = self.sampler.lock().unwrap();
                sampler.cosine_weighted_on_hemisphere()
            };
            let sample_half = (sample_direction + wo).normalize();
            (sample_direction, sample_half)
        } else {
            let (rand_x, rand_y) = {
                let mut sampler = self.sampler.lock().unwrap();
                sampler.uniform_2d()
            };
            let cos_theta_sqr =
                crate::material::util::ggx_ndf_cdf_inverse(self.roughness_sqr, rand_x);
            let cos_theta = cos_theta_sqr.sqrt();
            let sin_theta = (1.0 - cos_theta_sqr).sqrt();
            let phi = 2.0 * std::f32::consts::PI * rand_y;
            let sample_half = Vector3::new(sin_theta * phi.cos(), sin_theta * phi.sin(), cos_theta);
            let sample_direction = 2.0 * wo.dot(sample_half) * sample_half - wo;
            (sample_direction, sample_half)
        };
        if sample_direction.z <= 0.0 {
            (sample_direction, 1.0, Color::BLACK)
        } else {
            let pdf = 0.5
                * crate::material::util::ggx_ndf(sample_half.z, self.roughness_sqr)
                * sample_half.z
                / (4.0 * wo.dot(sample_half))
                + 0.5 * sample_direction.z / std::f32::consts::PI;
            let bsdf = self.bsdf(wo, sample_direction);
            (sample_direction, pdf, bsdf)
        }
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

    fn pdf(&self, wo: Vector3<f32>, wi: Vector3<f32>) -> f32 {
        let half = (wo + wi).normalize();
        0.5 * crate::material::util::ggx_ndf(half.z, self.roughness_sqr) * half.z
            / (4.0 * wo.dot(half))
            + 0.5 * wi.z / std::f32::consts::PI
    }

    fn is_delta(&self) -> bool {
        false
    }

    fn emissive(&self) -> Color {
        self.emissive
    }
}
