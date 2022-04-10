use std::{collections::HashMap, sync::Arc};

use crate::{
    camera::Camera,
    core::{scene::Scene, surface::Surface},
    light::{EnvLight, Light, ShapeLight},
    light_sampler::{LightSampler, PowerIsLightSampler, UniformLightSampler},
    material::Material,
    medium::Medium,
    primitive::{BvhAccel, Group, Instance, InstancePtr, Primitive},
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
    environment_light_index: Option<usize>,
}

impl SceneResources {
    pub fn to_scene(
        self,
        aggregate_type: Option<&str>,
        light_sampler_type: Option<&str>,
    ) -> anyhow::Result<Scene> {
        let aggregate = self.build_aggregate(aggregate_type)?;
        let light_sampler = self.build_light_sampler(light_sampler_type)?;

        self.log_list();
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

    pub fn log_list(&self) {
        log::info!("{} primitives", self.primitives.len());
        for (i, name) in self.primitives.keys().enumerate() {
            log::info!("- primtive {} - {}", i, name)
        }
        log::info!("{} textures", self.textures.len());
        for (i, name) in self.textures.keys().enumerate() {
            log::info!("- texture {} - {}", i, name)
        }
        log::info!("{} mediums", self.mediums.len());
        for (i, name) in self.mediums.keys().enumerate() {
            log::info!("- medium {} - {}", i, name)
        }
        log::info!("{} materials", self.materials.len());
        for (i, name) in self.materials.keys().enumerate() {
            log::info!("- material {} - {}", i, name)
        }
        log::info!("{} surfaces", self.surfaces.len());
        for (i, name) in self.surfaces.keys().enumerate() {
            log::info!("- surface {} - {}", i, name)
        }
        log::info!("{} lights (without emissive mesh)", self.lights.len());
        for (i, name) in self.lights.keys().enumerate() {
            log::info!("- light {} - {}", i, name)
        }
        log::info!("{} instances", self.instances.len());
        for (i, name) in self.instances.keys().enumerate() {
            log::info!("- instance {} - {}", i, name)
        }
        log::info!("{} cameras", self.cameras.len());
        for (i, name) in self.cameras.keys().enumerate() {
            log::info!("- camera {} - {}", i, name)
        }
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

        let mut instance_light_map = HashMap::new();
        for instance in self.instances.values() {
            if instance.surface().is_emissive() {
                let light = ShapeLight::new(instance.clone());
                let light: Arc<Light> = Arc::new(light.into());
                lights.push(light.clone());
                instance_light_map.insert(InstancePtr(Arc::as_ptr(instance)), lights.len() - 1);
            }
        }

        let light_sampler = if let Some(ty) = ty {
            match ty {
                "uniform" => UniformLightSampler::new(lights, self.environment_light_index).into(),
                "power_is" => PowerIsLightSampler::new(
                    lights,
                    self.environment_light_index,
                    instance_light_map,
                )
                .into(),
                _ => anyhow::bail!(format!("Unknown light sampler type '{}'", ty)),
            }
        } else {
            UniformLightSampler::new(lights, self.environment_light_index).into()
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
            self.environment_light_index = Some(self.lights.len() - 1);
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

    pub fn merge(&mut self, another: SceneResources) {
        for (name, cam) in another.cameras {
            if !self.cameras.contains_key(&name) {
                self.cameras.insert(name, cam);
            }
        }
        for (name, inst) in another.instances {
            if !self.instances.contains_key(&name) {
                self.instances.insert(name, inst);
            }
        }
        for (name, prim) in another.primitives {
            if !self.primitives.contains_key(&name) {
                self.primitives.insert(name, prim);
            }
        }
        for (name, surf) in another.surfaces {
            if !self.surfaces.contains_key(&name) {
                self.surfaces.insert(name, surf);
            }
        }
        for (name, mat) in another.materials {
            if !self.materials.contains_key(&name) {
                self.materials.insert(name, mat);
            }
        }
        for (name, med) in another.mediums {
            if !self.mediums.contains_key(&name) {
                self.mediums.insert(name, med);
            }
        }
        for (name, tex) in another.textures {
            if !self.textures.contains_key(&name) {
                self.textures.insert(name, tex);
            }
        }
        for (name, light) in another.lights {
            if !self.lights.contains_key(&name) {
                self.lights.insert(name, light);
            }
        }
        if let Some(env) = another.environment {
            if self.environment.is_none() {
                self.environment = Some(env);
            }
        }
    }
}
