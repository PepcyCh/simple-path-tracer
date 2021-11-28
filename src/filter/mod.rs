mod boxf;

pub use boxf::*;

use crate::core::loader::InputParams;

#[enum_dispatch::enum_dispatch(Filter)]
pub trait FilterT: Send + Sync {
    fn radius(&self) -> i32;

    fn weight(&self, x: f32, y: f32) -> f32;
}

#[enum_dispatch::enum_dispatch]
pub enum Filter {
    BoxFilter,
}

pub fn create_filter_from_params(params: &mut InputParams) -> anyhow::Result<Filter> {
    params.set_name("filter".into());
    let ty = params.get_str("type")?;
    params.set_name(format!("filter-{}", ty).into());

    let res = match ty.as_str() {
        "box" => BoxFilter::load(params)?.into(),
        _ => anyhow::bail!(format!("{}: unknown type '{}'", params.name(), ty)),
    };

    params.check_unused_keys();

    Ok(res)
}
