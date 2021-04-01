use crate::core::color::Color;
use crate::core::material::Material;
use crate::core::sampler::Sampler;
use cgmath::{InnerSpace, Vector3};
use std::cell::RefCell;

pub struct Glass {
    reflectance: Color,
    transmittance: Color,
    ior: f32,
    sampler: Box<RefCell<dyn Sampler>>,
}

impl Glass {
    pub fn new(
        reflectance: Color,
        transmittance: Color,
        ior: f32,
        sampler: Box<RefCell<dyn Sampler>>,
    ) -> Self {
        Self {
            reflectance,
            transmittance,
            ior,
            sampler,
        }
    }
}

impl Material for Glass {
    fn sample(&self, wo: Vector3<f32>) -> (Vector3<f32>, f32, Color) {
        let fresnel = crate::material::util::schlick_fresnel(self.ior, wo.z);
        let rand = self.sampler.borrow_mut().uniform_1d();
        if rand <= fresnel {
            let reflect = crate::material::util::reflect(wo);
            (
                reflect,
                fresnel,
                fresnel * self.reflectance / reflect.z.abs(),
            )
        } else {
            if let Some(refract) = crate::material::util::refract(wo, self.ior) {
                let k = if wo.z >= 0.0 {
                    1.0 / self.ior
                } else {
                    self.ior
                };
                (
                    refract,
                    1.0 - fresnel,
                    k * k * (1.0 - fresnel) * self.transmittance / refract.z.abs(),
                )
            } else {
                (wo, 1.0 - fresnel, Color::BLACK)
            }
        }
    }

    fn bsdf(&self, wo: Vector3<f32>, wi: Vector3<f32>) -> Color {
        let fresnel = crate::material::util::schlick_fresnel(self.ior, wo.z);
        if wo.z * wi.z >= 0.0 {
            let reflect = crate::material::util::reflect(wo);
            if reflect.dot(wi) >= 0.99 {
                return fresnel * self.reflectance / wi.z.abs();
            }
        } else {
            if let Some(refract) = crate::material::util::refract(wo, self.ior) {
                if refract.dot(wi) >= 0.99 {
                    let k = if wo.z >= 0.0 {
                        1.0 / self.ior
                    } else {
                        self.ior
                    };
                    return k * k * (1.0 - fresnel) * self.transmittance / wi.z.abs();
                }
            }
        }
        Color::BLACK
    }

    fn pdf(&self, wo: Vector3<f32>, wi: Vector3<f32>) -> f32 {
        let fresnel = crate::material::util::schlick_fresnel(self.ior, wo.z);
        if wo.z * wi.z >= 0.0 {
            let reflect = crate::material::util::reflect(wo);
            if reflect.dot(wi) >= 0.99 {
                return fresnel;
            }
        } else {
            if let Some(refract) = crate::material::util::refract(wo, self.ior) {
                if refract.dot(wi) >= 0.99 {
                    return 1.0 - fresnel;
                }
            }
        }
        0.0
    }

    fn is_delta(&self) -> bool {
        true
    }
}
