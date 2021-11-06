use std::sync::Arc;

use crate::{
    core::{
        color::Color, intersection::Intersection, material::Material, scatter::Scatter,
        scene::Scene,
    },
    loader::{self, JsonObject, LoadableSceneObject},
    scatter::SpecularTransmit,
};

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
}

impl LoadableSceneObject for PseudoMaterial {
    fn load(
        scene: &mut Scene,
        _path: &std::path::PathBuf,
        json_value: &JsonObject,
    ) -> anyhow::Result<()> {
        let name = loader::get_str_field(json_value, "material-pseudo", "name")?;
        let env = format!("material-pseudo({})", name);
        if scene.materials.contains_key(name) {
            anyhow::bail!(format!("{}: name is duplicated", env));
        }

        let mat = PseudoMaterial::new();
        scene.materials.insert(name.to_owned(), Arc::new(mat));

        Ok(())
    }
}
