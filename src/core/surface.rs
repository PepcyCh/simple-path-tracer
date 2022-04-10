use std::sync::Arc;

use crate::{
    bxdf::Bxdf,
    core::{
        color::Color, coord::Coordinate, intersection::Intersection, loader::InputParams, ray::Ray,
        scene_resources::SceneResources,
    },
    material::{Material, MaterialT},
    medium::Medium,
    texture::{Texture, TextureT},
};

pub struct Surface {
    material: Arc<Material>,
    normal_map: Option<Arc<Texture>>,
    _displacement_map: Option<Arc<Texture>>, // TODO: disp map, not supported
    emissive: Color,
    emissive_map: Option<Arc<Texture>>,
    double_sided: bool,
    inside_medium: Option<Arc<Medium>>,
}

impl Surface {
    pub fn new(
        material: Arc<Material>,
        normal_map: Option<Arc<Texture>>,
        displacement_map: Option<Arc<Texture>>,
        emissive: Color,
        emissive_map: Option<Arc<Texture>>,
        double_sided: bool,
        inside_medium: Option<Arc<Medium>>,
    ) -> Self {
        Self {
            material,
            normal_map,
            _displacement_map: displacement_map,
            emissive,
            emissive_map,
            double_sided,
            inside_medium,
        }
    }

    pub fn is_emissive(&self) -> bool {
        self.emissive.luminance() > 0.0
    }

    pub fn emissive(&self, inter: &Intersection<'_>) -> Color {
        self.emissive
            * self
                .emissive_map
                .as_ref()
                .map_or(Color::WHITE, |map| map.color_at(inter.into()))
    }

    pub fn average_emissive(&self) -> Color {
        self.emissive
            * self
                .emissive_map
                .as_ref()
                .map_or(Color::WHITE, |map| map.average_color())
    }

    pub fn coord(&self, ray: &Ray, inter: &Intersection<'_>) -> Coordinate {
        let shade_normal = if let Some(normal_map) = &self.normal_map {
            let value = normal_map.color_at(inter.into());
            let normal_color = value * 2.0 - Color::WHITE;
            let normal = glam::Vec3A::new(normal_color.r, normal_color.g, normal_color.b);
            let shade_normal_local = normal.normalize();
            (shade_normal_local.x * inter.tangent.normalize()
                + shade_normal_local.y * inter.bitangent.normalize()
                + shade_normal_local.z * inter.normal)
                .normalize()
        } else {
            inter.normal
        };

        let hit_back = ray.direction.dot(inter.normal) > 0.0;
        let coord = Coordinate::from_tangent_normal(
            inter.tangent,
            if self.double_sided && hit_back {
                -shade_normal
            } else {
                shade_normal
            },
            if hit_back {
                -inter.normal
            } else {
                inter.normal
            },
        );

        coord
    }

    pub fn scatter_and_coord(&self, ray: &Ray, inter: &Intersection<'_>) -> (Bxdf, Coordinate) {
        let scatter = self.material.bxdf_context(inter);

        let coord = self.coord(ray, inter);

        (scatter, coord)
    }

    pub fn inside_medium(&self) -> Option<&Medium> {
        if self.double_sided {
            None
        } else {
            self.inside_medium.as_ref().map(|medium| medium.as_ref())
        }
    }

    pub fn double_sided(&self) -> bool {
        self.double_sided
    }

    pub fn load(rsc: &mut SceneResources, params: &mut InputParams) -> anyhow::Result<()> {
        params.set_name("surface".into());
        let name = params.get_str("name")?;
        params.set_name(format!("surface-{}", name).into());

        let material = rsc.clone_material(params.get_str("material")?)?;

        let normal_map = if params.contains_key("normal_map") {
            Some(rsc.clone_texture(params.get_str("normal_map")?)?)
        } else {
            None
        };
        let displacement_map = if params.contains_key("displacement_map") {
            Some(rsc.clone_texture(params.get_str("displacement_map")?)?)
        } else {
            None
        };

        let emissive = params.get_float3_or("emissive", [0.0, 0.0, 0.0]).into();
        let emissive_map = if params.contains_key("emissive_map") {
            Some(rsc.clone_texture(params.get_str("emissive_map")?)?)
        } else {
            None
        };

        let double_sided = params.get_bool_or("double_sided", false);

        let inside_medium = if params.contains_key("inside_medium") {
            Some(rsc.clone_medium(params.get_str("inside_medium")?)?)
        } else {
            None
        };

        let res = Surface::new(
            material,
            normal_map,
            displacement_map,
            emissive,
            emissive_map,
            double_sided,
            inside_medium,
        );
        rsc.add_surface(name, res)?;

        params.check_unused_keys();

        Ok(())
    }
}
