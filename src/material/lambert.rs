use std::sync::Arc;

use crate::{
    core::{intersection::Intersection, loader::InputParams, scene_resources::SceneResources},
    scatter::{LambertReflect, Scatter},
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
    fn scatter(&self, inter: &Intersection<'_>) -> Scatter {
        let albedo = self.albedo.color_at(inter);
        LambertReflect::new(albedo).into()
    }
}
