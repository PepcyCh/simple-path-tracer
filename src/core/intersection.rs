use crate::core::primitive::Primitive;

pub struct Intersection<'a> {
    pub t: f32,
    pub normal: cgmath::Vector3<f32>,
    pub texcoords: cgmath::Point2<f32>,
    pub primitive: Option<&'a dyn Primitive>,
}

impl Default for Intersection<'_> {
    fn default() -> Self {
        Self {
            t: f32::MAX,
            normal: cgmath::Vector3::unit_y(),
            texcoords: cgmath::Point2::new(0.0, 0.0),
            primitive: None,
        }
    }
}
