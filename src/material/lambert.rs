use std::sync::Arc;

use crate::{
    bxdf::{self, Bxdf},
    core::{intersection::Intersection, loader::InputParams, scene_resources::SceneResources},
    texture::{Texture, TextureT},
};

use super::MaterialT;

pub struct Lambert {
    albedo: Arc<Texture>,
}

impl Lambert {
    pub fn new(albedo: Arc<Texture>) -> Self {
        Self { albedo }
    }

    pub fn load(rsc: &SceneResources, params: &mut InputParams) -> anyhow::Result<Self> {
        let albedo = rsc.clone_texture(params.get_str("albedo")?)?;
        Ok(Self::new(albedo))
    }
}

impl MaterialT for Lambert {
    fn bxdf_context(&self, inter: &Intersection<'_>) -> Bxdf {
        let albedo = self.albedo.color_at(inter.into());
        bxdf::Lambert::new(albedo).into()
    }
}
