mod util;

mod homogeneous;

pub use homogeneous::*;

use crate::core::{color::Color, loader::InputParams, rng::Rng, scene::Scene};

#[enum_dispatch::enum_dispatch(Medium)]
pub trait MediumT: Send + Sync {
    /// return (
    ///   sample position pi,
    ///   still in medium or not,
    ///   transport attenuation / transport pdf
    /// )
    fn sample_pi(
        &self,
        po: glam::Vec3A,
        wo: glam::Vec3A,
        t_max: f32,
        sampler: &mut Rng,
    ) -> (glam::Vec3A, bool, Color);

    /// return (
    ///   sample direction wi,
    ///   phase pdf
    /// )
    fn sample_wi(&self, wo: glam::Vec3A, sampler: &mut Rng) -> (glam::Vec3A, f32);

    fn transport_attenuation(&self, dist: f32) -> Color;

    fn phase(&self, wo: glam::Vec3A, wi: glam::Vec3A) -> f32;
}

#[enum_dispatch::enum_dispatch]
pub enum Medium {
    Homogeneous,
}

pub fn create_medium_from_params(
    scene: &mut Scene,
    params: &mut InputParams,
) -> anyhow::Result<()> {
    params.set_name("medium".into());
    let ty = params.get_str("type")?;
    let name = params.get_str("name")?;
    params.set_name(format!("medium-{}-{}", ty, name).into());

    let res = match ty.as_str() {
        "homogeneous" => Homogeneous::load(scene, params)?.into(),
        _ => anyhow::bail!(format!("{}: unknown type '{}'", params.name(), ty)),
    };

    scene.add_medium(name, res)?;

    params.check_unused_keys();

    Ok(())
}
