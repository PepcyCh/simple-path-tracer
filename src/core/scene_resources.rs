use std::{collections::HashMap, sync::Arc};

use crate::{
    camera::Camera,
    core::{scene::Scene, surface::Surface},
    light::{EnvLight, Light, ShapeLight},
    light_sampler::{LightSampler, LightSamplerT, PowerIsLightSampler, UniformLightSampler},
    material::Material,
    medium::Medium,
    primitive::{BvhAccel, Group, Instance, Primitive},
    texture::Texture,
};

#[derive(Default)]
pub struct SceneResources {
    cameras: HashMap<String, Arc<Camera>>,
    instances: HashMap<String, Arc<Instance>>,
    primitives: HashMap<String, Arc<Primitive>>,
    surfaces: HashMap<String, Arc<Surface>>,
    materials: HashMap<String, Arc<Material>>,
    mediums: HashMap<String, Arc<Medium>>,
    textures: HashMap<String, Arc<Texture>>,
    lights: HashMap<String, Arc<Light>>,
    environment: Option<Arc<Light>>,
}

impl SceneResources {
    pub fn to_scene(
        self,
        aggregate_type: Option<&str>,
        light_sampler_type: Option<&str>,
    ) -> anyhow::Result<Scene> {
        let aggregate = self.build_aggregate(aggregate_type)?;
        let light_sampler = self.build_light_sampler(light_sampler_type)?;

        log::info!(
            "{} instances, {} lights",
            self.instances.len(),
            light_sampler.num_lights()
        );

        if self.cameras.is_empty() {
            anyhow::bail!("At least one camera is needed");
        }

        Ok(Scene::new(
            self.cameras,
            aggregate,
            light_sampler,
            self.environment,
        ))
    }

    fn build_aggregate(&self, ty: Option<&str>) -> anyhow::Result<Primitive> {
        let instances = self
            .instances
            .iter()
            .map(|(_, inst)| inst.clone())
            .collect::<Vec<_>>();

        let aggregate = if let Some(ty) = ty {
            match ty {
                "group" => Group::new(instances).into(),
                "bvh" => BvhAccel::new(instances, 4, 16).into(),
                _ => anyhow::bail!(format!("Unknown aggregate type '{}'", ty)),
            }
        } else {
            BvhAccel::new(instances, 4, 16).into()
        };

        Ok(aggregate)
    }

    fn build_light_sampler(&self, ty: Option<&str>) -> anyhow::Result<LightSampler> {
        let mut lights = self
            .lights
            .iter()
            .map(|(_, inst)| inst.clone())
            .collect::<Vec<_>>();

        for instance in self.instances.values() {
            if instance.surface().is_emissive() {
                let light = ShapeLight::new(instance.clone());
                lights.push(Arc::new(light.into()));
            }
        }

        let light_sampler = if let Some(ty) = ty {
            match ty {
                "uniform" => UniformLightSampler::new(lights).into(),
                "power_is" => PowerIsLightSampler::new(lights).into(),
                _ => anyhow::bail!(format!("Unknown light sampler type '{}'", ty)),
            }
        } else {
            UniformLightSampler::new(lights).into()
        };

        Ok(light_sampler)
    }

    pub fn add_camera(&mut self, name: String, camera: Camera) -> anyhow::Result<()> {
        if self.cameras.contains_key(&name) {
            anyhow::bail!(format!("Duplicated camera name '{}'", name));
        } else {
            self.cameras.insert(name, Arc::new(camera));
            Ok(())
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
