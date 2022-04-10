use crate::{
    bxdf::{Bxdf, Pseudo},
    core::{intersection::Intersection, loader::InputParams, scene_resources::SceneResources},
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
    fn bxdf_context(&self, _inter: &Intersection<'_>) -> Bxdf {
        Pseudo::new().into()
    }
}
