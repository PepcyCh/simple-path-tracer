use std::{collections::HashMap, sync::Arc};

use anyhow::Context;

use crate::{
    camera::Camera,
    core::surface::Surface,
    light::{EnvLight, Light, ShapeLight},
    material::Material,
    medium::Medium,
    primitive::{BvhAccel, Group, Instance, Primitive},
    texture::Texture,
};

#[derive(Default)]
pub struct Scene {
    pub cameras: HashMap<String, Arc<Camera>>,
    pub aggregate: Option<Primitive>,
    pub instances: HashMap<String, Arc<Instance>>,
    pub primitives: HashMap<String, Arc<Primitive>>,
    pub surfaces: HashMap<String, Arc<Surface>>,
    pub materials: HashMap<String, Arc<Material>>,
    pub mediums: HashMap<String, Arc<Medium>>,
    pub textures: HashMap<String, Arc<Texture>>,
    pub lights: HashMap<String, Arc<Light>>,
    pub environment: Option<Arc<Light>>,
}

impl Scene {
    pub fn aggregate(&self) -> &Primitive {
        self.aggregate.as_ref().unwrap()
    }

    pub fn build_aggregate(&mut self, ty: Option<&str>) -> anyhow::Result<()> {
        let instances = self
            .instances
            .iter()
            .map(|(_, inst)| inst.clone())
            .collect::<Vec<_>>();

        self.aggregate = Some(if let Some(ty) = ty {
            match ty {
                "group" => Group::new(instances).into(),
                "bvh" => BvhAccel::new(instances, 4, 16).into(),
                _ => anyhow::bail!(format!("Unknown aggregate type '{}'", ty)),
            }
        } else {
            BvhAccel::new(instances, 4, 16).into()
        });

        Ok(())
    }

    pub fn collect_shape_lights(&mut self) {
        for (name, instance) in &self.instances {
            if instance.surface().is_emissive() {
                let light = ShapeLight::new(instance.clone());
                self.lights
                    .insert(format!("$${}", name), Arc::new(light.into()));
            }
        }
    }

    pub fn add_camera(&mut self, name: String, camera: Camera) -> anyhow::Result<()> {
        if self.cameras.contains_key(&name) {
            anyhow::bail!(format!("Duplicated camera name '{}'", name));
        } else {
            self.cameras.insert(name, Arc::new(camera));
            Ok(())
        }
    }

    pub fn get_camera(&self, name: &Option<String>) -> &Camera {
        if let Some(name) = name {
            self.cameras
                .get(name)
                .context(format!("There is no camera names {}", name))
                .unwrap()
        } else if self.cameras.len() == 1 {
            self.cameras.values().next().unwrap()
        } else {
            panic!("There are multiple cameras so a name must be given");
        }
    }

    pub fn add_light(&mut self, name: String, light: Light) -> anyhow::Result<()> {
        if self.lights.contains_key(&name) {
            anyhow::bail!(format!("Duplicated light name '{}'", name));
        } else {
            self.lights.insert(name, Arc::new(light));
            Ok(())
        }
    }

    pub fn add_environment(&mut self, env: EnvLight) -> anyhow::Result<()> {
        if self.environment.is_some() {
            anyhow::bail!("Environment has been set before");
        } else {
            let env: Arc<Light> = Arc::new(env.into());
            self.lights.insert("$env".to_owned(), env.clone());
            self.environment = Some(env);
            Ok(())
        }
    }

    pub fn add_instance(&mut self, name: String, instance: Instance) -> anyhow::Result<()> {
        if self.instances.contains_key(&name) {
            anyhow::bail!(format!("Duplicated instance name '{}'", name));
        } else {
            self.instances.insert(name, Arc::new(instance));
            Ok(())
        }
    }

    pub fn add_material(&mut self, name: String, material: Material) -> anyhow::Result<()> {
        if self.materials.contains_key(&name) {
            anyhow::bail!(format!("Duplicated material name '{}'", name));
        } else {
            self.materials.insert(name, Arc::new(material));
            Ok(())
        }
    }

    pub fn clone_material(&self, name: String) -> anyhow::Result<Arc<Material>> {
        if let Some(material) = self.materials.get(&name) {
            Ok(material.clone())
        } else {
            anyhow::bail!(format!("There is no material named '{}'", name))
        }
    }

    pub fn add_medium(&mut self, name: String, medium: Medium) -> anyhow::Result<()> {
        if self.mediums.contains_key(&name) {
            anyhow::bail!(format!("Duplicated medium name '{}'", name));
        } else {
            self.mediums.insert(name, Arc::new(medium));
            Ok(())
        }
    }

    pub fn clone_medium(&self, name: String) -> anyhow::Result<Arc<Medium>> {
        if let Some(medium) = self.mediums.get(&name) {
            Ok(medium.clone())
        } else {
            anyhow::bail!(format!("There is no medium named '{}'", name))
        }
    }

    pub fn add_primitive(&mut self, name: String, primitive: Primitive) -> anyhow::Result<()> {
        if self.primitives.contains_key(&name) {
            anyhow::bail!(format!("Duplicated primitive name '{}'", name));
        } else {
            self.primitives.insert(name, Arc::new(primitive));
            Ok(())
        }
    }

    pub fn clone_primitive(&self, name: String) -> anyhow::Result<Arc<Primitive>> {
        if let Some(primitive) = self.primitives.get(&name) {
            Ok(primitive.clone())
        } else {
            anyhow::bail!(format!("There is no primitive named '{}'", name))
        }
    }

    pub fn add_surface(&mut self, name: String, surface: Surface) -> anyhow::Result<()> {
        if self.surfaces.contains_key(&name) {
            anyhow::bail!(format!("Duplicated surface name '{}'", name));
        } else {
            self.surfaces.insert(name, Arc::new(surface));
            Ok(())
        }
    }

    pub fn clone_surface(&self, name: String) -> anyhow::Result<Arc<Surface>> {
        if let Some(surface) = self.surfaces.get(&name) {
            Ok(surface.clone())
        } else {
            anyhow::bail!(format!("There is no surface named '{}'", name))
        }
    }

    pub fn add_texture(&mut self, name: String, texture: Texture) -> anyhow::Result<()> {
        if self.textures.contains_key(&name) {
            anyhow::bail!(format!("Duplicated texture name '{}'", name));
        } else {
            self.textures.insert(name, Arc::new(texture));
            Ok(())
        }
    }

    pub fn clone_texture(&self, name: String) -> anyhow::Result<Arc<Texture>> {
        if let Some(texture) = self.textures.get(&name) {
            Ok(texture.clone())
        } else {
            anyhow::bail!(format!("There is no texture named '{}'", name))
        }
    }
}
