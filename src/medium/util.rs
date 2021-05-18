use cgmath::{InnerSpace, Vector3};

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

pub fn local_to_world(wo_world: Vector3<f32>, wi_local: Vector3<f32>) -> Vector3<f32> {
    let v = if wo_world.y.abs() < 0.99 {
        cgmath::Vector3::unit_y()
    } else {
        cgmath::Vector3::unit_x()
    };
    let u = (v.cross(wo_world)).normalize();
    let v = wo_world.cross(u);
    let local_to_world = cgmath::Matrix3::from_cols(u, v, wo_world);
    local_to_world * wi_local
}
