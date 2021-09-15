pub fn henyey_greenstein(asymmetric: f32, cos: f32) -> f32 {
    let g = asymmetric;
    let g2 = g * g;
    let denom = 1.0 + g2 + 2.0 * g * cos;
    let denom = denom * denom.sqrt();
    0.25 * std::f32::consts::FRAC_1_PI * (1.0 - g2) / denom
}

/// return sampled cos
pub fn henyey_greenstein_cdf_inverse(asymmetric: f32, rand: f32) -> f32 {
    if asymmetric.abs() < 0.01 {
        1.0 - 2.0 * rand
    } else {
        let g = asymmetric;
        let g2 = g * g;
        let temp = (1.0 - g2) / (1.0 - g + 2.0 * g * rand);
        0.5 * (1.0 + g2 - temp * temp) / g
    }
}

pub fn local_to_world(wo_world: glam::Vec3A, wi_local: glam::Vec3A) -> glam::Vec3A {
    let v = if wo_world.y.abs() < 0.99 {
        glam::Vec3A::Y
    } else {
        glam::Vec3A::X
    };
    let u = (v.cross(wo_world)).normalize();
    let v = wo_world.cross(u);
    let local_to_world = glam::Mat3A::from_cols(u, v, wo_world);
    local_to_world * wi_local
}
