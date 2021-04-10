use crate::core::color::Color;

pub fn reflect(i: cgmath::Vector3<f32>) -> cgmath::Vector3<f32> {
    cgmath::Vector3::new(-i.x, -i.y, i.z)
}

pub fn refract(i: cgmath::Vector3<f32>, ior: f32) -> Option<cgmath::Vector3<f32>> {
    let ior_ratio = if i.z >= 0.0 { 1.0 / ior } else { ior };
    let o_z_sqr = 1.0 - (1.0 - i.z * i.z) * ior_ratio * ior_ratio;
    if o_z_sqr >= 0.0 {
        let o_z = if i.z >= 0.0 {
            -o_z_sqr.sqrt()
        } else {
            o_z_sqr.sqrt()
        };
        Some(cgmath::Vector3::new(
            -i.x * ior_ratio,
            -i.y * ior_ratio,
            o_z,
        ))
    } else {
        None
    }
}

pub fn fresnel_r0(ior: f32) -> f32 {
    pow2((1.0 - ior) / (1.0 + ior))
}

pub fn schlick_fresnel(ior: f32, cos: f32) -> f32 {
    let r0 = fresnel_r0(ior);
    r0 + (1.0 - r0) * pow5(1.0 - cos)
}

pub fn schlick_fresnel_with_r0(r0: Color, cos: f32) -> Color {
    r0 + (Color::WHITE - r0) * pow5(1.0 - cos)
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
