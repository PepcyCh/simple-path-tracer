use crate::{
    core::{color::Color, intersection::Intersection, loader::InputParams, scene::Scene},
    scatter::{Scatter, SpecularTransmit},
};

use super::MaterialT;

pub struct PseudoMaterial {}

impl PseudoMaterial {
    pub fn new() -> Self {
        Self {}
    }

    pub fn load(_scene: &Scene, _params: &mut InputParams) -> anyhow::Result<Self> {
        Ok(PseudoMaterial::new())
    }
}

impl MaterialT for PseudoMaterial {
    fn scatter(&self, _inter: &Intersection<'_>) -> Scatter {
        SpecularTransmit::new(Color::WHITE, 1.0).into()
    }
}
