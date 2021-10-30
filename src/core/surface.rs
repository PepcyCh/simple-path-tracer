use std::sync::Arc;

use crate::{
    core::{
        color::Color, coord::Coordinate, intersection::Intersection, material::Material,
        medium::Medium, ray::Ray, scatter::Scatter, scene::Scene, texture::Texture,
    },
    loader::{self, JsonObject, Loadable},
};

pub struct Surface {
    material: Arc<dyn Material>,
    normal_map: Option<Arc<dyn Texture<Color>>>,
    _displacement_map: Option<Arc<dyn Texture<f32>>>, // TODO: disp map, not supported
    emissive: Color,
    emissive_map: Option<Arc<dyn Texture<Color>>>,
    double_sided: bool,
    inside_medium: Option<Arc<dyn Medium>>,
}

impl Surface {
    pub fn new(
        material: Arc<dyn Material>,
        normal_map: Option<Arc<dyn Texture<Color>>>,
        displacement_map: Option<Arc<dyn Texture<f32>>>,
        emissive: Color,
        emissive_map: Option<Arc<dyn Texture<Color>>>,
        double_sided: bool,
        inside_medium: Option<Arc<dyn Medium>>,
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
                .map_or(Color::WHITE, |map| map.value_at(inter))
    }

    pub fn coord(&self, ray: &Ray, inter: &Intersection<'_>) -> Coordinate {
        let shade_normal = if let Some(normal_map) = &self.normal_map {
            let value = normal_map.value_at(inter);
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
        let coord = Coordinate::from_z(
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

    pub fn scatter_and_coord(
        &self,
        ray: &Ray,
        inter: &Intersection<'_>,
    ) -> (Box<dyn Scatter>, Coordinate) {
        let scatter = self.material.scatter(inter);

        let coord = self.coord(ray, inter);

        (scatter, coord)
    }

    pub fn inside_medium(&self) -> Option<&dyn Medium> {
        if self.double_sided {
            None
        } else {
            self.inside_medium.as_ref().map(|medium| medium.as_ref())
        }
    }

    pub fn double_sided(&self) -> bool {
        self.double_sided
    }
}

impl Loadable for Surface {
    fn load(
        scene: &mut Scene,
        _path: &std::path::PathBuf,
        json_value: &JsonObject,
    ) -> anyhow::Result<()> {
        let name = loader::get_str_field(json_value, "surface", "name")?;
        let env = format!("surface({})", name);
        if scene.surfaces.contains_key(name) {
            anyhow::bail!(format!("{}: name is duplicated", env));
        }

        let mat_name = loader::get_str_field(json_value, &env, "material")?;
        let material = if let Some(mat) = scene.materials.get(mat_name) {
            mat.clone()
        } else {
            anyhow::bail!(format!("{}: material '{}' not found", env, mat_name))
        };

        let normal_map = if json_value.contains_key("normal_map") {
            let tex_name = loader::get_str_field(json_value, &env, "normal_map")?;
            if let Some(tex) = scene.textures_color.get(tex_name) {
                Some(tex.clone())
            } else {
                anyhow::bail!(format!("{}: normal_map '{}' not found", env, tex_name))
            }
        } else {
            None
        };

        let displacement_map = if json_value.contains_key("displacement_map") {
            let tex_name = loader::get_str_field(json_value, &env, "displacement_map")?;
            if let Some(tex) = scene.textures_f32.get(tex_name) {
                Some(tex.clone())
            } else {
                anyhow::bail!(format!(
                    "{}: displacement_map '{}' not found",
                    env, tex_name
                ))
            }
        } else {
            None
        };

        let emissive =
            loader::get_float_array3_field_or(json_value, &env, "emissive", [0.0, 0.0, 0.0])?;

        let emissive_map = if json_value.contains_key("emissive_map") {
            let tex_name = loader::get_str_field(json_value, &env, "emissive_map")?;
            if let Some(tex) = scene.textures_color.get(tex_name) {
                Some(tex.clone())
            } else {
                anyhow::bail!(format!("{}: emissive_map '{}' not found", env, tex_name))
            }
        } else {
            None
        };

        let double_sided = loader::get_bool_field_or(json_value, &env, "double_sided", false)?;

        let inside_medium = if json_value.contains_key("inside_medium") {
            let med_name = loader::get_str_field(json_value, &env, "inside_medium")?;
            if let Some(med) = scene.mediums.get(med_name) {
                Some(med.clone())
            } else {
                anyhow::bail!(format!("{}: inside_medium '{}' not found", env, med_name))
            }
        } else {
            None
        };

        let surf = Surface::new(
            material,
            normal_map,
            displacement_map,
            emissive.into(),
            emissive_map,
            double_sided,
            inside_medium,
        );
        scene.surfaces.insert(name.to_owned(), Arc::new(surf));

        Ok(())
    }
}
