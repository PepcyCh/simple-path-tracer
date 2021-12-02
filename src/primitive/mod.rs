mod bezier;
mod bvh;
mod catmull;
mod group;
mod instance;
mod sphere;
mod triangle;

pub use bvh::*;
pub use group::*;
pub use instance::*;

pub use bezier::*;
pub use catmull::*;
pub use sphere::*;
pub use triangle::*;

use crate::core::{
    bbox::Bbox, intersection::Intersection, loader::InputParams, ray::Ray, rng::Rng,
    scene_resources::SceneResources, transform::Transform,
};

#[enum_dispatch::enum_dispatch(Primitive)]
pub trait PrimitiveT: Send + Sync {
    fn intersect_test(&self, ray: &Ray, t_max: f32) -> bool;

    fn intersect<'a>(&'a self, ray: &Ray, inter: &mut Intersection<'a>) -> bool;

    fn bbox(&self) -> Bbox;

    fn sample<'a>(&'a self, rng: &mut Rng) -> (Intersection<'a>, f32);

    /// sample pdf relative to area
    fn pdf(&self, inter: &Intersection<'_>) -> f32;

    /// returns an estimated value, not need to be accurate
    fn surface_area(&self, trans: Transform) -> f32;
}

#[enum_dispatch::enum_dispatch]
pub enum Primitive {
    BvhAccelPrimitive(BvhAccel<Primitive>),
    BvhAccelInstance(BvhAccel<Instance>),
    BvhAccelTriangle(BvhAccel<Triangle>),
    BvhAccelCubicBezier(BvhAccel<CubicBezier>),
    CatmullClark,
    CubicBezier,
    GroupPrimitive(Group<Primitive>),
    Group(Group<Instance>),
    Instance,
    Sphere,
    TriMesh,
}

pub fn create_primitive_from_params(
    rsc: &mut SceneResources,
    params: &mut InputParams,
) -> anyhow::Result<()> {
    params.set_name("primitive".into());
    let ty = params.get_str("type")?;
    let name = params.get_str("name")?;
    params.set_name(format!("primitive-{}-{}", ty, name).into());

    let res = match ty.as_str() {
        "sphere" => Sphere::load(rsc, params)?.into(),
        "trimesh" => TriMesh::load(rsc, params)?.into(),
        "cubic_bezier" => CubicBezier::load(rsc, params)?.into(),
        "catmull_clark" => CatmullClark::load(rsc, params)?.into(),
        _ => anyhow::bail!(format!("{}: unknown type '{}'", params.name(), ty)),
    };

    rsc.add_primitive(name, res)?;

    params.check_unused_keys();

    Ok(())
}

#[derive(Clone, Copy)]
pub enum BasicPrimitiveRef<'a> {
    CubicBezier(&'a CubicBezier),
    Sphere(&'a Sphere),
    Triangle(&'a Triangle),
}

impl<'a> PrimitiveT for BasicPrimitiveRef<'a> {
    fn intersect_test(&self, ray: &Ray, t_max: f32) -> bool {
        match self {
            BasicPrimitiveRef::CubicBezier(ele) => ele.intersect_test(ray, t_max),
            BasicPrimitiveRef::Sphere(ele) => ele.intersect_test(ray, t_max),
            BasicPrimitiveRef::Triangle(ele) => ele.intersect_test(ray, t_max),
        }
    }

    fn intersect<'b>(&'b self, ray: &Ray, inter: &mut Intersection<'b>) -> bool {
        match self {
            BasicPrimitiveRef::CubicBezier(ele) => ele.intersect(ray, inter),
            BasicPrimitiveRef::Sphere(ele) => ele.intersect(ray, inter),
            BasicPrimitiveRef::Triangle(ele) => ele.intersect(ray, inter),
        }
    }

    fn bbox(&self) -> Bbox {
        match self {
            BasicPrimitiveRef::CubicBezier(ele) => ele.bbox(),
            BasicPrimitiveRef::Sphere(ele) => ele.bbox(),
            BasicPrimitiveRef::Triangle(ele) => ele.bbox(),
        }
    }

    fn sample<'b>(&'b self, rng: &mut Rng) -> (Intersection<'b>, f32) {
        match self {
            BasicPrimitiveRef::CubicBezier(ele) => ele.sample(rng),
            BasicPrimitiveRef::Sphere(ele) => ele.sample(rng),
            BasicPrimitiveRef::Triangle(ele) => ele.sample(rng),
        }
    }

    fn pdf(&self, inter: &Intersection<'_>) -> f32 {
        match self {
            BasicPrimitiveRef::CubicBezier(ele) => ele.pdf(inter),
            BasicPrimitiveRef::Sphere(ele) => ele.pdf(inter),
            BasicPrimitiveRef::Triangle(ele) => ele.pdf(inter),
        }
    }

    fn surface_area(&self, trans: Transform) -> f32 {
        match self {
            BasicPrimitiveRef::CubicBezier(ele) => ele.surface_area(trans),
            BasicPrimitiveRef::Sphere(ele) => ele.surface_area(trans),
            BasicPrimitiveRef::Triangle(ele) => ele.surface_area(trans),
        }
    }
}
