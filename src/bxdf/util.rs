use crate::core::color::Color;

pub fn reflect(i: glam::Vec3A) -> glam::Vec3A {
    glam::Vec3A::new(-i.x, -i.y, i.z)
}

pub fn reflect_n(i: glam::Vec3A, n: glam::Vec3A) -> glam::Vec3A {
    2.0 * i.dot(n) * n - i
}

pub fn refract(i: glam::Vec3A, ior: f32) -> Option<glam::Vec3A> {
    let ior_ratio = if i.z >= 0.0 { 1.0 / ior } else { ior };
    let o_z_sqr = 1.0 - (1.0 - i.z * i.z) * ior_ratio * ior_ratio;
    if o_z_sqr >= 0.0 {
        let o_z = if i.z >= 0.0 {
            -o_z_sqr.sqrt()
        } else {
            o_z_sqr.sqrt()
        };
        Some(glam::Vec3A::new(-i.x * ior_ratio, -i.y * ior_ratio, o_z))
    } else {
        None
    }
}

pub fn refract_n(i: glam::Vec3A, n: glam::Vec3A, ior: f32) -> Option<glam::Vec3A> {
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

pub fn fresnel(ior: f32, i: glam::Vec3A) -> f32 {
    fresnel_n(ior, i, glam::Vec3A::Z)
}

pub fn fresnel_n(ior: f32, i: glam::Vec3A, n: glam::Vec3A) -> f32 {
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

        0.5 * (rs + rp)
    } else {
        1.0
    }
}

pub fn fresnel_conductor(ior: Color, ior_k: Color, i: glam::Vec3A) -> Color {
    fresnel_conductor_n(ior, ior_k, i, glam::Vec3A::Z)
}

pub fn fresnel_conductor_n(ior: Color, ior_k: Color, i: glam::Vec3A, n: glam::Vec3A) -> Color {
    let cos = i.dot(n);
    let (ior_ratio, k_ratio) = if cos >= 0.0 {
        (ior, ior_k)
    } else {
        (Color::WHITE / ior, Color::WHITE / ior_k)
    };

    let cos2 = cos * cos;
    let sin2 = 1.0 - cos2;
    let ior_ratio2 = ior_ratio * ior_ratio;
    let k_ratio2 = k_ratio * k_ratio;

    let t0 = ior_ratio2 - k_ratio2 - Color::gray(sin2);
    let a2_b2 = (t0 * t0 + 4.0 * ior_ratio2 * k_ratio2).sqrt();
    let t1 = a2_b2 + Color::gray(cos2);
    let a = (0.5 * (a2_b2 + t0)).sqrt();
    let t2 = 2.0 * cos * a;
    let rs = (t1 - t2) / (t1 + t2);

    let t3 = cos2 * a2_b2 + Color::gray(sin2 * sin2);
    let t4 = t2 * sin2;
    let rp = rs * (t3 - t4) / (t3 + t4);

    return 0.5 * (rs + rp);
}

#[allow(dead_code)]
pub fn schlick_fresnel(ior: f32, cos: f32) -> f32 {
    let r0 = fresnel_r0(ior);
    r0 + (1.0 - r0) * pow5(1.0 - cos)
}

pub fn schlick_fresnel_with_r0(r0: Color, cos: f32) -> Color {
    r0 + (Color::WHITE - r0) * pow5(1.0 - cos)
}

pub fn fresnel_moment1(eta: f32) -> f32 {
    let eta2 = eta * eta;
    let eta3 = eta2 * eta;
    let eta4 = eta3 * eta;
    let eta5 = eta4 * eta;
    if eta < 1.0 {
        0.45966 - 1.73965 * eta + 3.37668 * eta2 - 3.904945 * eta3 + 2.49277 * eta4 - 0.68441 * eta5
    } else {
        -4.61686 + 11.1136 * eta - 10.4646 * eta2 + 5.11455 * eta3 - 1.27198 * eta4 + 0.12746 * eta5
    }
}

pub fn half_from_reflect(i: glam::Vec3A, o: glam::Vec3A) -> glam::Vec3A {
    if i.z >= 0.0 {
        (i + o).normalize()
    } else {
        -(i + o).normalize()
    }
}

pub fn half_from_refract(i: glam::Vec3A, o: glam::Vec3A, ior: f32) -> glam::Vec3A {
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

#[allow(dead_code)]
pub fn ggx_ndf(ndoth: f32, a2: f32) -> f32 {
    a2 * std::f32::consts::FRAC_1_PI / (pow2(ndoth * ndoth * (a2 - 1.0) + 1.0)).max(0.0001)
}

pub fn ggx_ndf_aniso(h: glam::Vec3A, ax: f32, ay: f32) -> f32 {
    std::f32::consts::FRAC_1_PI
        / (ax * ay * pow2(pow2(h.x / ax) + pow2(h.y / ay) + pow2(h.z))).max(0.0001)
}

/// return sampled (n dot h)^2
#[allow(dead_code)]
pub fn ggx_ndf_cdf_inverse(a2: f32, rand: f32) -> f32 {
    (1.0 - rand) / (1.0 - rand * (1.0 - a2))
}

pub fn smith_g1_aniso(v: glam::Vec3A, ax: f32, ay: f32) -> f32 {
    2.0 / (1.0 + (1.0 + (pow2(ax * v.x) + pow2(ay * v.y)) / pow2(v.z).max(0.0001)).sqrt())
}

pub fn smith_separable_visible_aniso(v: glam::Vec3A, l: glam::Vec3A, ax: f32, ay: f32) -> f32 {
    let v = v.z.abs() + (pow2(ax * v.x) + pow2(ay * v.y) + pow2(v.z)).sqrt();
    let l = l.z.abs() + (pow2(ax * l.x) + pow2(ay * l.y) + pow2(l.z)).sqrt();
    1.0 / (v * l)
}

#[allow(dead_code)]
pub fn smith_separable_visible(ndotv: f32, ndotl: f32, a2: f32) -> f32 {
    let v = ndotv.abs() + ((1.0 - a2) * ndotv * ndotv + a2).sqrt();
    let l = ndotl.abs() + ((1.0 - a2) * ndotl * ndotl + a2).sqrt();
    1.0 / (v * l)
}

pub fn ggx_smith_vndf_pdf(h: glam::Vec3A, v: glam::Vec3A, ax: f32, ay: f32) -> f32 {
    let v = if v.z >= 0.0 { v } else { -v };

    return smith_g1_aniso(v, ax, ay) * ggx_ndf_aniso(h, ax, ay) * v.dot(h).max(0.0)
        / v.z.max(0.0001);
}

pub fn ggx_smith_vndf_sample(
    ve: glam::Vec3A,
    ax: f32,
    ay: f32,
    rand: (f32, f32),
) -> (glam::Vec3A, f32) {
    let ve = if ve.z >= 0.0 { ve } else { -ve };

    let vh = glam::Vec3A::new(ax * ve.x, ay * ve.y, ve.z).normalize();
    let len_sqr = vh.x * vh.x + vh.y * vh.y;
    let t_vec1 = if len_sqr > 0.0 {
        glam::Vec3A::new(-vh.y, vh.x, 0.0) / len_sqr.sqrt()
    } else {
        glam::Vec3A::X
    };
    let t_vec2 = vh.cross(t_vec1);

    let r = rand.0.sqrt();
    let phi = 2.0 * std::f32::consts::PI * rand.1;
    let t1 = r * phi.cos();
    let t2 = r * phi.sin();
    let s = 0.5 * (1.0 + vh.z);
    let t2 = (1.0 - s) * (1.0 - t1 * t1).sqrt() + s * t2;
    let nh = t1 * t_vec1 + t2 * t_vec2 + (1.0 - t1 * t1 - t2 * t2).max(0.0).sqrt() * vh;
    let ne = glam::Vec3A::new(ax * nh.x, ay * nh.y, nh.z.max(0.0)).normalize();

    let pdf = ggx_smith_vndf_pdf(ne, ve, ax, ay);
    return (ne, pdf);
}

fn pow2(x: f32) -> f32 {
    x * x
}

fn pow5(x: f32) -> f32 {
    x * x * x * x * x
}
