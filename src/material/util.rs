use crate::core::color::Color;
use cgmath::{InnerSpace, Vector3};

pub fn reflect(i: Vector3<f32>) -> Vector3<f32> {
    Vector3::new(-i.x, -i.y, i.z)
}

pub fn reflect_n(i: Vector3<f32>, n: Vector3<f32>) -> Vector3<f32> {
    2.0 * i.dot(n) * n - i
}

pub fn refract(i: Vector3<f32>, ior: f32) -> Option<Vector3<f32>> {
    let ior_ratio = if i.z >= 0.0 { 1.0 / ior } else { ior };
    let o_z_sqr = 1.0 - (1.0 - i.z * i.z) * ior_ratio * ior_ratio;
    if o_z_sqr >= 0.0 {
        let o_z = if i.z >= 0.0 {
            -o_z_sqr.sqrt()
        } else {
            o_z_sqr.sqrt()
        };
        Some(Vector3::new(-i.x * ior_ratio, -i.y * ior_ratio, o_z))
    } else {
        None
    }
}

pub fn refract_n(i: Vector3<f32>, n: Vector3<f32>, ior: f32) -> Option<Vector3<f32>> {
    let cos_i = i.dot(n);
    if cos_i >= 0.0 {
        let ior_ratio = 1.0 / ior;
        let o_z_sqr = 1.0 - (1.0 - cos_i * cos_i) * ior_ratio * ior_ratio;
        if o_z_sqr >= 0.0 {
            Some((ior_ratio * cos_i - o_z_sqr.sqrt()) * n - ior_ratio * i)
        } else {
            None
        }
    } else {
        let ior_ratio = ior;
        let o_z_sqr = 1.0 - (1.0 - cos_i * cos_i) * ior_ratio * ior_ratio;
        if o_z_sqr >= 0.0 {
            Some((o_z_sqr.sqrt() + ior_ratio * cos_i) * n - ior_ratio * i)
        } else {
            None
        }
    }
}

#[allow(dead_code)]
pub fn fresnel_r0(ior: f32) -> f32 {
    pow2((1.0 - ior) / (1.0 + ior))
}

pub fn fresnel(ior: f32, i: Vector3<f32>) -> f32 {
    fresnel_n(ior, i, Vector3::unit_z())
}

pub fn fresnel_n(ior: f32, i: Vector3<f32>, n: Vector3<f32>) -> f32 {
    let (i_ior, o_ior) = if i.dot(n) >= 0.0 {
        (1.0, ior)
    } else {
        (ior, 1.0)
    };

    if let Some(refract) = refract_n(i, n, ior) {
        let idotn = i.dot(n).abs();
        let rdotn = refract.dot(n).abs();

        let denom = i_ior * idotn + o_ior * rdotn;
        let num = i_ior * idotn - o_ior * rdotn;
        let rs = num / denom;
        let rs = rs * rs;

        let denom = i_ior * rdotn + o_ior * idotn;
        let num = i_ior * rdotn - o_ior * idotn;
        let rp = num / denom;
        let rp = rp * rp;

        (rs + rp) * 0.5
    } else {
        1.0
    }
}

#[allow(dead_code)]
pub fn schlick_fresnel(ior: f32, cos: f32) -> f32 {
    let r0 = fresnel_r0(ior);
    r0 + (1.0 - r0) * pow5(1.0 - cos)
}

pub fn schlick_fresnel_with_r0(r0: Color, cos: f32) -> Color {
    r0 + (Color::WHITE - r0) * pow5(1.0 - cos)
}

pub fn half_from_refract(i: Vector3<f32>, o: Vector3<f32>, ior: f32) -> Vector3<f32> {
    let mut half = if i.z >= 0.0 {
        (i + ior * o).normalize()
    } else {
        (ior * i + o).normalize()
    };
    if half.z < 0.0 {
        half = -half;
    }
    half
}

pub fn ggx_ndf(ndoth: f32, a2: f32) -> f32 {
    a2 / (std::f32::consts::PI * pow2(ndoth * ndoth * (a2 - 1.0) + 1.0)).max(0.0001)
}

/// return sampled (n dot h)^2
pub fn ggx_ndf_cdf_inverse(a2: f32, rand: f32) -> f32 {
    (1.0 - rand) / (1.0 - rand * (1.0 - a2))
}

pub fn smith_separable_visible(ndotv: f32, ndotl: f32, a2: f32) -> f32 {
    let v = ndotv.abs() + ((1.0 - a2) * ndotv * ndotv + a2).sqrt();
    let l = ndotl.abs() + ((1.0 - a2) * ndotl * ndotl + a2).sqrt();
    1.0 / (v * l)
}

fn pow2(x: f32) -> f32 {
    x * x
}

fn pow5(x: f32) -> f32 {
    x * x * x * x * x
}
