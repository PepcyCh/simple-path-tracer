use std::{collections::HashMap, path::Path, sync::Arc};

use anyhow::Context;
use byte_slice_cast::AsSliceOf;
use glam::Vec4Swizzles;

use crate::{
    camera,
    core::{color::Color, scene::Scene, scene_resources::SceneResources, surface::Surface},
    light, material, primitive, texture,
};

pub fn load_scene<P: AsRef<Path>>(path: P) -> anyhow::Result<Scene> {
    let (gltf_doc, buffers, images) = gltf::import(path)?;

    let mut rsc = SceneResources::default();

    load_images(&mut rsc, images)?;

    let material_name_map = load_materials(&mut rsc, &gltf_doc)?;

    let mesh_name_map = load_primitives(&mut rsc, &gltf_doc, &buffers)?;

    let name_map = NameMaps {
        mesh_name_map,
        material_name_map,
    };

    for scene in gltf_doc.scenes() {
        for node in scene.nodes() {
            parse_nodes(&mut rsc, &node, &name_map, glam::Mat4::IDENTITY)?;
        }
    }

    let scene = rsc.to_scene(None, None)?;
    Ok(scene)
}

struct NameMaps {
    mesh_name_map: HashMap<usize, String>,
    material_name_map: HashMap<Option<usize>, String>,
}

fn load_images(rsc: &mut SceneResources, images: Vec<gltf::image::Data>) -> anyhow::Result<()> {
    for (i, image) in images.into_iter().enumerate() {
        let width = image.width;
        let height = image.height;
        let dynamic_image = match image.format {
            gltf::image::Format::R8 => image::DynamicImage::ImageLuma8(
                image::GrayImage::from_vec(width, height, image.pixels).context(format!(
                    "Failed to convert image {} (format is R8) to image::DynamicImage",
                    i
                ))?,
            ),
            gltf::image::Format::R8G8 => image::DynamicImage::ImageLumaA8(
                image::GrayAlphaImage::from_vec(width, height, image.pixels).context(format!(
                    "Failed to convert image {} (format is R8G8) to image::DynamicImage",
                    i
                ))?,
            ),
            gltf::image::Format::R8G8B8 => image::DynamicImage::ImageRgb8(
                image::RgbImage::from_vec(width, height, image.pixels).context(format!(
                    "Failed to convert image {} (format is R8G8B8) to image::DynamicImage",
                    i
                ))?,
            ),
            gltf::image::Format::R8G8B8A8 => image::DynamicImage::ImageRgba8(
                image::RgbaImage::from_vec(width, height, image.pixels).context(format!(
                    "Failed to convert image {} (format is R8G8B8A8) to image::DynamicImage",
                    i
                ))?,
            ),
            gltf::image::Format::B8G8R8 => image::DynamicImage::ImageBgr8(
                image::ImageBuffer::from_vec(width, height, image.pixels).context(format!(
                    "Failed to convert image {} (format is B8G8R8) to image::DynamicImage",
                    i
                ))?,
            ),
            gltf::image::Format::B8G8R8A8 => image::DynamicImage::ImageBgra8(
                image::ImageBuffer::from_vec(width, height, image.pixels).context(format!(
                    "Failed to convert image {} (format is B8G8R8A8) to image::DynamicImage",
                    i
                ))?,
            ),
            _ => anyhow::bail!(format!("Failed to convert image {} to image::DynamicImage, 16-bit image is currently not supported", i))
        };
        let tex =
            texture::ImageTex::new(dynamic_image, glam::Vec2::new(1.0, 1.0), glam::Vec2::ZERO);
        rsc.add_texture(format!("image_{}", i), tex.into())?;
    }

    let scalar_one_tex = texture::ScalarTex::new(Color::WHITE);
    rsc.add_texture("scalar_one".to_owned(), scalar_one_tex.into())?;

    Ok(())
}

fn load_materials(
    rsc: &mut SceneResources,
    gltf_doc: &gltf::Document,
) -> anyhow::Result<HashMap<Option<usize>, String>> {
    let mut name_map = HashMap::with_capacity(gltf_doc.materials().len());

    for (mat_index, gltf_mat) in gltf_doc.materials().enumerate() {
        let mat_name = if let Some(name) = gltf_mat.name() {
            name.to_owned()
        } else {
            format!("material_{}", mat_index)
        };
        name_map.insert(gltf_mat.index(), mat_name.clone());

        let mat = if let Some(pbr_specular) = gltf_mat.pbr_specular_glossiness() {
            let diffuse_fact = pbr_specular.diffuse_factor();
            let diffuse_fact_tex = texture::ScalarTex::new(Color::new(
                diffuse_fact[0],
                diffuse_fact[1],
                diffuse_fact[2],
            ))
            .into();
            let diffuse = if let Some(diffuse_tex) = pbr_specular.diffuse_texture() {
                let image_index = diffuse_tex.texture().index();
                texture::MulTex::new(
                    Arc::new(diffuse_fact_tex),
                    rsc.clone_texture(format!("image_{}", image_index))?,
                )
                .into()
            } else {
                diffuse_fact_tex
            };

            let specular_fact = pbr_specular.specular_factor();
            let specular_fact_tex = texture::ScalarTex::new(specular_fact.into()).into();
            let glossiness_fact = pbr_specular.glossiness_factor();
            let (specular, roughness, roughness_chan) = if let Some(sg_tex) =
                pbr_specular.specular_glossiness_texture()
            {
                let image_index = sg_tex.texture().index();
                let sg = rsc.clone_texture(format!("image_{}", image_index))?;
                (
                    texture::MulTex::new(Arc::new(specular_fact_tex), sg.clone()).into(),
                    texture::SubTex::new(
                        rsc.clone_texture("scalar_one".to_owned())?,
                        Arc::new(
                            texture::MulTex::new(
                                Arc::new(
                                    texture::ScalarTex::new(Color::gray(glossiness_fact)).into(),
                                ),
                                sg,
                            )
                            .into(),
                        ),
                    )
                    .into(),
                    texture::TextureChannel::A,
                )
            } else {
                (
                    specular_fact_tex,
                    texture::ScalarTex::new(Color::gray(1.0 - glossiness_fact)).into(),
                    texture::TextureChannel::R,
                )
            };

            let roughness_x: Arc<texture::Texture> = Arc::new(roughness);
            let roughness_y = roughness_x.clone();

            material::PbrSpecular::new(
                Arc::new(diffuse),
                Arc::new(specular),
                roughness_x,
                roughness_y,
                roughness_chan,
            )
            .into()
        } else {
            let pbr_metallic = gltf_mat.pbr_metallic_roughness();

            let base_color_fact = pbr_metallic.base_color_factor();
            let base_color_fact_tex = texture::ScalarTex::new(Color::new(
                base_color_fact[0],
                base_color_fact[1],
                base_color_fact[2],
            ))
            .into();
            let base_color = if let Some(base_color_tex) = pbr_metallic.base_color_texture() {
                let image_index = base_color_tex.texture().index();
                texture::MulTex::new(
                    Arc::new(base_color_fact_tex),
                    rsc.clone_texture(format!("image_{}", image_index))?,
                )
                .into()
            } else {
                base_color_fact_tex
            };

            let metallic_fact = pbr_metallic.metallic_factor();
            let metallic_fact_tex = texture::ScalarTex::new(Color::gray(metallic_fact)).into();
            let roughness_fact = pbr_metallic.roughness_factor();
            let roughness_fact_tex = texture::ScalarTex::new(Color::gray(roughness_fact)).into();
            let (metallic, roughness) =
                if let Some(mr_tex) = pbr_metallic.metallic_roughness_texture() {
                    let image_index = mr_tex.texture().index();
                    let mr = rsc.clone_texture(format!("image_{}", image_index))?;
                    (
                        texture::MulTex::new(Arc::new(metallic_fact_tex), mr.clone()).into(),
                        texture::MulTex::new(Arc::new(roughness_fact_tex), mr).into(),
                    )
                } else {
                    (metallic_fact_tex, roughness_fact_tex)
                };

            let roughness_x = Arc::new(roughness);
            let roughness_y = roughness_x.clone();

            material::PbrMetallic::new(
                Arc::new(base_color),
                roughness_x,
                roughness_y,
                texture::TextureChannel::G,
                Arc::new(metallic),
                texture::TextureChannel::B,
            )
            .into()
        };

        let double_sided = gltf_mat.double_sided();

        let emissive = gltf_mat.emissive_factor().into();
        let emissive_map = if let Some(emissive_tex) = gltf_mat.emissive_texture() {
            let image_index = emissive_tex.texture().index();
            Some(rsc.clone_texture(format!("image_{}", image_index))?)
        } else {
            None
        };

        let normal_map = if let Some(normal_tex) = gltf_mat.normal_texture() {
            let image_index = normal_tex.texture().index();
            Some(rsc.clone_texture(format!("image_{}", image_index))?)
        } else {
            None
        };

        let surf = Surface::new(
            Arc::new(mat),
            normal_map,
            None,
            emissive,
            emissive_map,
            double_sided,
            None,
        );
        rsc.add_surface(mat_name, surf)?;
    }

    Ok(name_map)
}

fn load_primitives(
    rsc: &mut SceneResources,
    gltf_doc: &gltf::Document,
    buffers: &[gltf::buffer::Data],
) -> anyhow::Result<HashMap<usize, String>> {
    let mut name_map = HashMap::with_capacity(gltf_doc.meshes().len());

    for (mesh_index, mesh) in gltf_doc.meshes().enumerate() {
        let mesh_name = if let Some(name) = mesh.name() {
            name.to_owned()
        } else {
            format!("mesh_{}", mesh_index)
        };
        name_map.insert(mesh.index(), mesh_name.clone());

        for (prim_index, prim) in mesh.primitives().enumerate() {
            let prim_name = format!("{}_prim_{}", mesh_name, prim_index);

            // indices
            let index_accessor = prim
                .indices()
                .context(format!("Primitives '{}' doesn't have indices", prim_name))?;
            let index_count = index_accessor.count();
            let mut indices = vec![0; index_count];
            let index_data = get_data_of_accessor(&index_accessor, buffers)?;
            if index_accessor.data_type() == gltf::accessor::DataType::U32 {
                let index_data = index_data.as_slice_of::<u32>()?;
                for i in 0..index_count {
                    indices[i] = index_data[i];
                }
            } else if index_accessor.data_type() == gltf::accessor::DataType::U16 {
                let index_data = index_data.as_slice_of::<u16>()?;
                for i in 0..index_count {
                    indices[i] = index_data[i] as u32;
                }
            }

            // positions
            let position_accessor = prim
                .get(&gltf::mesh::Semantic::Positions)
                .context(format!("Primitive '{}' doesn't have positions", prim_name))?;

            let vertex_count = position_accessor.count();
            let mut vertices = vec![primitive::MeshVertex::default(); vertex_count];

            let position_data = get_data_of_accessor(&position_accessor, buffers)?;
            let position_data = position_data.as_slice_of::<f32>().unwrap();
            for i in 0..vertex_count {
                vertices[i].position[0] = position_data[3 * i];
                vertices[i].position[1] = position_data[3 * i + 1];
                vertices[i].position[2] = position_data[3 * i + 2];
            }
            // texcoords
            if let Some(accessor) = prim.get(&gltf::mesh::Semantic::TexCoords(0)) {
                let data = get_data_of_accessor(&accessor, buffers)?;
                let data = data.as_slice_of::<f32>().unwrap();
                for i in 0..vertex_count {
                    vertices[i].texcoords[0] = data[2 * i];
                    vertices[i].texcoords[1] = data[2 * i + 1];
                }
            }
            // normal
            if let Some(accessor) = prim.get(&gltf::mesh::Semantic::Normals) {
                let data = get_data_of_accessor(&accessor, buffers)?;
                let data = data.as_slice_of::<f32>().unwrap();
                for i in 0..vertex_count {
                    vertices[i].normal[0] = data[3 * i];
                    vertices[i].normal[1] = data[3 * i + 1];
                    vertices[i].normal[2] = data[3 * i + 2];
                }
            } else {
                primitive::TriMesh::calc_normals(&mut vertices, &indices);
            }

            primitive::TriMesh::calc_tangents(&mut vertices, &indices);

            rsc.add_primitive(prim_name, primitive::TriMesh::new(vertices, indices).into())?;
        }
    }

    Ok(name_map)
}

fn parse_nodes(
    rsc: &mut SceneResources,
    node: &gltf::Node,
    name_map: &NameMaps,
    trans: glam::Mat4,
) -> anyhow::Result<()> {
    let curr_transform = glam::Mat4::from_cols_array_2d(&node.transform().matrix());
    let trans = trans * curr_transform;

    if let Some(mesh) = node.mesh() {
        let mesh_name = &name_map.mesh_name_map[&mesh.index()];
        for (prim_index, gltf_prim) in mesh.primitives().enumerate() {
            let prim_name = format!("{}_prim_{}", mesh_name, prim_index);
            let inst_name = format!("{}_node_{}", prim_name, node.index());
            let prim = rsc.clone_primitive(prim_name)?;

            let mat_name = name_map.material_name_map[&gltf_prim.material().index()].clone();
            let surface = rsc.clone_surface(mat_name)?;

            let instance =
                primitive::Instance::new(prim, glam::Affine3A::from_mat4(trans), surface);
            rsc.add_instance(inst_name, instance)?;
        }
    }
    if let Some(cam) = node.camera() {
        let cam_name = if let Some(name) = cam.name() {
            name.to_owned()
        } else {
            format!("camera_{}", cam.index())
        };

        match cam.projection() {
            gltf::camera::Projection::Orthographic(_) => {
                log::warn!(
                    "Camera '{}' is orthographic and is not supported yet",
                    cam_name
                );
            }
            gltf::camera::Projection::Perspective(proj) => {
                let fov = proj.yfov();
                let eye = trans.col(3).xyz().into();
                let forward = (-trans.col(2).xyz()).into();
                let up = trans.col(1).xyz().into();
                let cam = camera::PerspectiveCamera::new(eye, forward, up, fov);
                rsc.add_camera(cam_name, cam.into())?;
            }
        }
    }
    if let Some(light) = node.light() {
        let light_name = if let Some(name) = light.name() {
            name.to_owned()
        } else {
            format!("light_{}", light.index())
        };
        let light_name = format!("{}_node_{}", light_name, node.index());

        let intensity = light.intensity();
        let color: Color = light.color().into();
        let strength = color * intensity;
        let light = match light.kind() {
            gltf::khr_lights_punctual::Kind::Directional => {
                let direction = (-trans.col(2).xyz()).into();
                light::DirLight::new(direction, strength).into()
            }
            gltf::khr_lights_punctual::Kind::Point => {
                let position = trans.col(3).xyz().into();
                light::PointLight::new(position, strength).into()
            }
            gltf::khr_lights_punctual::Kind::Spot {
                inner_cone_angle,
                outer_cone_angle,
            } => {
                let position = trans.col(3).xyz().into();
                let direction = (-trans.col(2).xyz()).into();
                light::SpotLight::new(
                    position,
                    direction,
                    inner_cone_angle,
                    outer_cone_angle,
                    strength,
                )
                .into()
            }
        };
        rsc.add_light(light_name, light)?;
    }

    for ch in node.children() {
        parse_nodes(rsc, &ch, name_map, trans)?;
    }

    Ok(())
}

fn get_data_of_accessor<'a>(
    accessor: &gltf::Accessor<'a>,
    buffers: &'a [gltf::buffer::Data],
) -> anyhow::Result<&'a [u8]> {
    let buffer_view = accessor.view().context("Accessor has no buffer view")?;
    let buffer = buffer_view.buffer();
    let buffer_data = &buffers[buffer.index()];
    let buffer_view_data =
        &buffer_data[buffer_view.offset()..buffer_view.offset() + buffer_view.length()];
    let accessor_data = &buffer_view_data
        [accessor.offset()..accessor.offset() + accessor.count() * accessor.size()];
    Ok(accessor_data)
}
