use crate::core::color::Color;
use crate::core::light::Light;
use crate::core::sampler::Sampler;
use cgmath::{InnerSpace, Point3, Vector3};
use std::cell::RefCell;

pub struct RectangleLight {
    strength: Color,
    center: Point3<f32>,
    direction: Vector3<f32>,
    width: f32,
    height: f32,
    right: Vector3<f32>,
    up: Vector3<f32>,
    sampler: Box<RefCell<dyn Sampler>>,
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
        sampler: Box<RefCell<dyn Sampler>>,
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
    fn sample_light(&self, position: Point3<f32>) -> (Vector3<f32>, f32, Color, f32) {
        let (offset_x, offset_y) = self.sampler.borrow_mut().uniform_2d();
        let sample_pos = self.center
            + (offset_x - 0.5) * self.width * self.right
            + (offset_y - 0.5) * self.height * self.up;
        let sample = sample_pos - position;
        let dist_sqr = sample.magnitude2();
        let dist = dist_sqr.sqrt();
        let sample = sample.normalize();
        let cos = -sample.dot(self.direction);
        let strength = if cos > 0.0 {
            self.strength * cos / dist_sqr
        } else {
            Color::BLACK
        };
        (sample, self.area_inv, strength, dist)
    }
}
