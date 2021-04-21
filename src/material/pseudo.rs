use crate::core::color::Color;
use crate::core::intersection::Intersection;
use crate::core::material::Material;
use crate::core::scatter::Scatter;
use crate::scatter::SpecularTransmit;

pub struct PseudoMaterial {}

impl PseudoMaterial {
    pub fn new() -> Self {
        Self {}
    }
}

impl Material for PseudoMaterial {
    fn scatter(&self, _inter: &Intersection<'_>) -> Box<dyn Scatter> {
        Box::new(SpecularTransmit::new(Color::WHITE, 1.0)) as Box<dyn Scatter>
    }

    fn emissive(&self, _inter: &Intersection<'_>) -> Color {
        Color::BLACK
    }
}
