use crate::{
    core::{
        color::Color, intersection::Intersection, loader::InputParams,
        scene_resources::SceneResources,
    },
    scatter::{Scatter, SpecularTransmit},
};

use super::MaterialT;

pub struct PseudoMaterial {}

impl PseudoMaterial {
    pub fn new() -> Self {
        Self {}
    }

    pub fn load(_rsc: &SceneResources, _params: &mut InputParams) -> anyhow::Result<Self> {
        Ok(Self::new())
    }
}

impl MaterialT for PseudoMaterial {
    fn scatter(&self, _inter: &Intersection<'_>) -> Scatter {
        SpecularTransmit::new(Color::WHITE, 1.0).into()
    }
}
