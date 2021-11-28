use std::sync::Arc;

use crate::{
    core::{intersection::Intersection, loader::InputParams, scene::Scene},
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

    pub fn load(scene: &Scene, params: &mut InputParams) -> anyhow::Result<Self> {
        let albedo = scene.clone_texture(params.get_str("albedo")?)?;
        Ok(Lambert::new(albedo))
    }
}

impl MaterialT for Lambert {
    fn scatter(&self, inter: &Intersection<'_>) -> Scatter {
        let albedo = self.albedo.color_at(inter);
        LambertReflect::new(albedo).into()
    }
}
