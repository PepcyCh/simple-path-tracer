use crate::core::{intersection::Intersection, scatter::Scatter};

pub trait Material: Send + Sync {
    fn scatter(&self, inter: &Intersection<'_>) -> Box<dyn Scatter>;
}
