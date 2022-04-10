use crate::{
    core::{color::Color, intersection::Intersection, ray::Ray, rng::Rng},
    primitive::PrimitiveT,
};

use super::{
    util, BxdfDirType, BxdfInputs, BxdfLobeType, BxdfSample, BxdfSampleType, BxdfSubsurfaceSample,
    BxdfT, Lambert,
};

#[enum_dispatch::enum_dispatch(Substrate)]
pub trait SubstrateT {
    fn sample<'a>(&self, inputs: &'a BxdfInputs, rng: &mut Rng) -> BxdfSample;

    fn pdf(&self, wo: glam::Vec3A, wi: glam::Vec3A) -> f32;

    fn bxdf(&self, wo: glam::Vec3A, wi: glam::Vec3A) -> Color;

    fn reflectance(&self) -> Color;
}

#[enum_dispatch::enum_dispatch]
pub enum Substrate {
    Lambert,
    Diffuse,
    Subsurface,
}

impl SubstrateT for Lambert {
    fn sample<'a>(&self, inputs: &'a BxdfInputs, rng: &mut Rng) -> BxdfSample {
        BxdfT::sample(self, inputs, rng)
    }

    fn pdf(&self, wo: glam::Vec3A, wi: glam::Vec3A) -> f32 {
        BxdfT::pdf(self, wo, wi)
    }

    fn bxdf(&self, wo: glam::Vec3A, wi: glam::Vec3A) -> Color {
        BxdfT::bxdf(self, wo, wi)
    }

    fn reflectance(&self) -> Color {
        self.reflectance()
    }
}

/*
pub struct Transmissive {
    transmittance: Color,
    ior: f32,
}

impl Transmissive {
    pub fn new(transmittance: Color, ior: f32) -> Self {
        Self {
            transmittance,
            ior,
        }
    }
}

impl SubstrateT for Transmissive {
    fn sample<'a>(&self, inputs: &'a BxdfInputs, rng: &Rng) -> BxdfSample {
        if let Some(wi) = util::refract_n(inputs.wo, m, self.ior) {
            let ior_ratio = if inputs.wo.z >= 0.0 { 1.0 / self.ior } else { self.ior };
            let num = 4.0 * inputs.wo.dot(m).abs() * wi.dot(m).abs();
            let denom = ior_ratio * inputs.wo.dot(m) + wi.dot(m);
            let denom = denom * denom;
            BxdfSample {
                pi: inputs.po,
                coord_pi: inputs.coord_po,
                wi,
                ty: BxdfSampleType {
                    lobe: BxdfLobeType::Specular,
                    dir: BxdfDirType::Transmit,
                    subsurface: false,
                },
                bxdf: self.transmittance * num / denom,
                pdf: 1.0,
            }
        } else {
            BxdfSample {
                pi: inputs.po,
                coord_pi: inputs.coord_po,
                wi: glam::Vec3A::ZERO,
                ty: BxdfSampleType {
                    lobe: BxdfLobeType::Specular,
                    dir: BxdfDirType::Transmit,
                    subsurface: false,
                },
                bxdf: Color::BLACK,
                pdf: 1.0,
            }
        }
    }

    fn pdf(&self, _wo: glam::Vec3A, _wi: glam::Vec3A, _m: glam::Vec3A) -> f32 {
        1.0
    }

    fn bxdf(&self, wo: glam::Vec3A, wi: glam::Vec3A) -> Color {
        if let Some(expected_wi) = util::refract_n(wo, m, self.ior) {
            if expected_wi.dot(wi) >= 0.999 {
                let ior_ratio = if wo.z >= 0.0 { 1.0 / self.ior } else { self.ior };
                let num = 4.0 * wo.dot(m).abs() * wi.dot(m).abs();
                let denom = ior_ratio * wo.dot(m) + wi.dot(m);
                let denom = denom * denom;
                return self.transmittance * num / denom;
            }
        }
        Color::BLACK
    }

    fn reflectance(&self) -> Color {
        self.transmittance
    }
}
 */

pub struct Diffuse {
    reflectance: Color,
    ior: f32,
    bxdf_wo_fresnel: Color,
}

impl Diffuse {
    pub fn new(reflectance: Color, ior: f32) -> Self {
        let fdr = 2.0 * util::fresnel_moment1(1.0 / ior);
        let bxdf_wo_fresnel = reflectance * std::f32::consts::FRAC_1_PI
            / ((Color::WHITE - reflectance * fdr) * ior * ior);
        Self {
            reflectance,
            ior,
            bxdf_wo_fresnel,
        }
    }
}

impl SubstrateT for Diffuse {
    fn sample<'a>(&self, inputs: &'a BxdfInputs, rng: &mut Rng) -> BxdfSample {
        let mut wi = rng.cosine_weighted_on_hemisphere();
        if inputs.wo.z < 0.0 {
            wi.z = -wi.z;
        }
        let fi = util::fresnel(self.ior, wi);
        let bxdf = (1.0 - fi) * self.bxdf_wo_fresnel;
        BxdfSample {
            wi,
            ty: BxdfSampleType {
                lobe: BxdfLobeType::Diffuse,
                dir: BxdfDirType::Reflect,
                subsurface: false,
            },
            bxdf,
            pdf: wi.z.abs() * std::f32::consts::FRAC_1_PI,
            subsurface: None,
        }
    }

    fn pdf(&self, wo: glam::Vec3A, wi: glam::Vec3A) -> f32 {
        if wo.z * wi.z >= 0.0 {
            wi.z.abs() * std::f32::consts::FRAC_1_PI
        } else {
            1.0
        }
    }

    fn bxdf(&self, wo: glam::Vec3A, wi: glam::Vec3A) -> Color {
        if wo.z * wi.z >= 0.0 {
            let fi = util::fresnel(self.ior, wi);
            (1.0 - fi) * self.bxdf_wo_fresnel
        } else {
            Color::BLACK
        }
    }

    fn reflectance(&self) -> Color {
        self.reflectance
    }
}

pub struct Subsurface {
    d: Color,
    diffuse: Diffuse,
}

lazy_static! {
    static ref SS_CDF_TABLE: Vec<(f32, f32)> = (0..512)
        .map(|i| {
            let x = i as f32 / 512.0;
            let x = -2.0 * (1.0 - x).ln();
            let y = 1.0 - (-x).exp() * 0.25 - (-x / 3.0).exp() * 0.75;
            (x, y)
        })
        .collect();
}

impl Subsurface {
    pub fn new(reflectance: Color, ior: f32, ld: f32) -> Self {
        let d = Color::new(
            ld / (3.5 + 100.0 * (reflectance.r - 0.33).powi(4)),
            ld / (3.5 + 100.0 * (reflectance.g - 0.33).powi(4)),
            ld / (3.5 + 100.0 * (reflectance.b - 0.33).powi(4)),
        );
        Self {
            d,
            diffuse: Diffuse::new(reflectance, ior),
        }
    }

    fn sp(&self, r: f32) -> Color {
        let exp1 = (-r / self.d).exp();
        let exp2 = (-r / self.d / 3.0).exp();
        (exp1 + exp2) * std::f32::consts::FRAC_1_PI / (8.0 * self.d * r)
    }

    fn sample_r(&self, rand: f32) -> f32 {
        for i in 1..SS_CDF_TABLE.len() {
            if SS_CDF_TABLE[i].1 >= rand {
                let t =
                    (rand - SS_CDF_TABLE[i - 1].1) / (SS_CDF_TABLE[i].1 - SS_CDF_TABLE[i - 1].1);
                let x = SS_CDF_TABLE[i].0 * t + SS_CDF_TABLE[i - 1].0 * (1.0 - t);
                return x;
            }
        }
        -1.0
    }
}

impl SubstrateT for Subsurface {
    fn sample<'a>(&self, inputs: &'a BxdfInputs, rng: &mut Rng) -> BxdfSample {
        let mut samp = BxdfSample {
            wi: glam::Vec3A::ZERO,
            ty: BxdfSampleType {
                lobe: BxdfLobeType::Diffuse,
                dir: BxdfDirType::Reflect,
                subsurface: true,
            },
            bxdf: Color::BLACK,
            pdf: 1.0,
            subsurface: None,
        };

        // sample pi
        let mut rand_u = rng.uniform_1d();
        let (rand_x, rand_y) = rng.uniform_2d();

        // p for primitive
        let pt = inputs.coord_po.to_world(glam::Vec3A::X);
        let pb = inputs.coord_po.to_world(glam::Vec3A::Y);
        let pn = inputs.coord_po.to_world(glam::Vec3A::Z);
        // s for sampled
        let (st, sb, sn) = if rand_u < 0.5 {
            rand_u = rand_u * 2.0;
            (pt, pb, pn)
        } else if rand_u < 0.75 {
            rand_u = rand_u * 4.0 - 2.0;
            (pb, pn, pt)
        } else {
            rand_u = rand_u * 4.0 - 3.0;
            (pn, pt, pb)
        };

        let sp_d = if rand_u < 1.0 / 3.0 {
            rand_u = 3.0 * rand_u;
            self.d.r
        } else if rand_u < 2.0 / 3.0 {
            rand_u = 3.0 * rand_u - 1.0;
            self.d.g
        } else {
            rand_u = 3.0 * rand_u - 2.0;
            self.d.b
        };
        let sample_r = self.sample_r(rand_x) * sp_d;
        let r_max = SS_CDF_TABLE.last().unwrap().0 * sp_d;
        if sample_r < 0.0 {
            return samp;
        }
        let pihi = 2.0 * std::f32::consts::PI * rand_y;
        let pihi_cos = pihi.cos();
        let pihi_sin = pihi.sin();
        let sample_l = (r_max * r_max + sample_r * sample_r).sqrt();

        let start_p: glam::Vec3A =
            inputs.po + st * pihi_cos * sample_r + sb * pihi_sin * sample_r + sn * sample_l;
        let mut ray = Ray::new(start_p, -sn);
        let mut inter = Intersection::with_t_max(2.0 * sample_l);
        let mut intersects = vec![];
        loop {
            if inputs.scene.intersect(&ray, &mut inter) {
                // TODO - check if the intersected one is the same as self
                let surf = inter.surface.unwrap();
                let coord_temp = surf.coord(&ray, &inter);
                intersects.push((ray.point_at(inter.t), inter.normal, coord_temp));
                ray.t_min = inter.t + Ray::T_MIN_EPS;
            } else {
                break;
            }
        }

        if intersects.is_empty() {
            return samp;
        }
        let sample_inter = ((rand_u * intersects.len() as f32) as usize).min(intersects.len() - 1);
        let (pi, sample_normal, sample_coord) = intersects[sample_inter];

        let sp = self.sp(pi.distance(inputs.po));

        let offset = inputs.coord_po.to_local(pi - inputs.po);
        let normal_local = inputs.coord_po.to_local(sample_normal);
        let r_xy = (offset.x * offset.x + offset.y * offset.y).sqrt();
        let r_yz = (offset.y * offset.y + offset.z * offset.z).sqrt();
        let r_zx = (offset.z * offset.z + offset.x * offset.x).sqrt();
        let pdf_xy = 0.5 * normal_local.z.abs() * self.sp(r_xy).avg();
        let pdf_yz = 0.25 * normal_local.x.abs() * self.sp(r_yz).avg();
        let pdf_zx = 0.25 * normal_local.y.abs() * self.sp(r_zx).avg();
        let pdf_pi = (pdf_xy + pdf_yz + pdf_zx) / intersects.len() as f32;

        samp.subsurface = Some(BxdfSubsurfaceSample {
            pi,
            coord_pi: sample_coord,
            sp,
            pdf_pi,
        });

        // sample wi
        let diffuse_samp = self.diffuse.sample(inputs, rng);
        samp.wi = diffuse_samp.wi;
        samp.pdf = diffuse_samp.pdf;
        samp.bxdf = diffuse_samp.bxdf;

        samp
    }

    fn pdf(&self, wo: glam::Vec3A, wi: glam::Vec3A) -> f32 {
        if wo.z * wi.z >= 0.0 {
            wi.z.abs() * std::f32::consts::FRAC_1_PI
        } else {
            1.0
        }
    }

    fn bxdf(&self, wo: glam::Vec3A, wi: glam::Vec3A) -> Color {
        self.diffuse.bxdf(wo, wi)
    }

    fn reflectance(&self) -> Color {
        self.diffuse.reflectance
    }
}
