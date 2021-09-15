use crate::core::{color::Color, light::Light, sampler::Sampler};

pub struct RectangleLight {
    strength: Color,
    center: glam::Vec3A,
    direction: glam::Vec3A,
    width: f32,
    height: f32,
    right: glam::Vec3A,
    up: glam::Vec3A,
    _area: f32,
    area_inv: f32,
}

impl RectangleLight {
    pub fn new(
        center: glam::Vec3A,
        direction: glam::Vec3A,
        width: f32,
        height: f32,
        up: glam::Vec3A,
        strength: Color,
    ) -> Self {
        let direction = direction.normalize();
        let right = (direction.cross(up)).normalize();
        let up = right.cross(direction);
        let area = width * height;
        Self {
            center,
            direction,
            width,
            height,
            right,
            up,
            strength,
            _area: area,
            area_inv: 1.0 / area,
        }
    }
}

impl Light for RectangleLight {
    fn sample(
        &self,
        position: glam::Vec3A,
        sampler: &mut dyn Sampler,
    ) -> (glam::Vec3A, f32, Color, f32) {
        let (offset_x, offset_y) = sampler.uniform_2d();
        let sample_pos = self.center
            + (offset_x - 0.5) * self.width * self.right
            + (offset_y - 0.5) * self.height * self.up;
        let sample = sample_pos - position;
        let dist_sqr = sample.length_squared();
        let dist = dist_sqr.sqrt();
        let sample = sample / dist;
        let cos = -sample.dot(self.direction);
        let (pdf, strength) = if cos > 0.0 {
            (self.area_inv * dist_sqr / cos, self.strength)
        } else {
            (1.0, Color::BLACK)
        };
        (sample, pdf, strength, dist)
    }

    fn strength_dist_pdf(&self, position: glam::Vec3A, wi: glam::Vec3A) -> (Color, f32, f32) {
        let cos = self.direction.dot(wi);
        if cos < 0.0 {
            let t = (self.center - position).dot(self.direction) / cos;
            if t > 0.0 && t.is_finite() {
                let intersect = position + wi * t;
                let offset = intersect - self.center;
                let x = offset.dot(self.right);
                let y = offset.dot(self.up);
                if x.abs() <= 0.5 * self.width && y.abs() <= 0.5 * self.height {
                    let dist = t;
                    let dist_sqr = dist * dist;
                    let pdf = self.area_inv * dist_sqr / -cos;
                    return (self.strength, dist, pdf);
                }
            }
        }
        (Color::BLACK, f32::MAX, 1.0)
    }

    fn is_delta(&self) -> bool {
        false
    }
}
