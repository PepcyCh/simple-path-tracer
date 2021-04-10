use crate::core::color::Color;
use crate::core::light::Light;
use crate::core::sampler::Sampler;
use cgmath::{InnerSpace, Point3, Vector3};
use std::sync::Mutex;

pub struct RectangleLight {
    strength: Color,
    center: Point3<f32>,
    direction: Vector3<f32>,
    width: f32,
    height: f32,
    right: Vector3<f32>,
    up: Vector3<f32>,
    sampler: Box<Mutex<dyn Sampler>>,
    _area: f32,
    area_inv: f32,
}

impl RectangleLight {
    pub fn new(
        center: Point3<f32>,
        direction: Vector3<f32>,
        width: f32,
        height: f32,
        up: Vector3<f32>,
        strength: Color,
        sampler: Box<Mutex<dyn Sampler>>,
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
            sampler,
            _area: area,
            area_inv: 1.0 / area,
        }
    }
}

impl Light for RectangleLight {
    fn sample(&self, position: Point3<f32>) -> (Vector3<f32>, f32, Color, f32) {
        let (offset_x, offset_y) = {
            let mut sampler = self.sampler.lock().unwrap();
            sampler.uniform_2d()
        };
        let sample_pos = self.center
            + (offset_x - 0.5) * self.width * self.right
            + (offset_y - 0.5) * self.height * self.up;
        let sample = sample_pos - position;
        let dist_sqr = sample.magnitude2();
        let dist = dist_sqr.sqrt();
        let sample = sample / dist;
        let cos = -sample.dot(self.direction);
        let (pdf, strength) = if cos > 0.0 {
            (self.area_inv * dist_sqr / cos, self.strength)
        } else {
            (0.0, Color::BLACK)
        };
        (sample, pdf, strength, dist)
    }

    fn strength_dist_pdf(&self, position: Point3<f32>, wi: Vector3<f32>) -> (Color, f32, f32) {
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
        (Color::BLACK, f32::MAX, 0.0)
    }

    fn is_delta(&self) -> bool {
        false
    }
}
