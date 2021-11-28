use crate::core::loader::InputParams;

use super::FilterT;

pub struct BoxFilter {
    radius: f32,
    radius_int: i32,
}

impl BoxFilter {
    pub fn new(radius: f32) -> Self {
        let radius_int = (radius - 0.5).ceil() as i32;
        Self { radius, radius_int }
    }

    pub fn load(params: &mut InputParams) -> anyhow::Result<Self> {
        let radius = params.get_float("radius")?;
        Ok(Self::new(radius))
    }
}

impl FilterT for BoxFilter {
    fn radius(&self) -> i32 {
        self.radius_int
    }

    fn weight(&self, x: f32, y: f32) -> f32 {
        if x.abs() <= self.radius && y.abs() <= self.radius {
            1.0
        } else {
            0.0
        }
    }
}
