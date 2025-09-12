use std::io::{ BufReader, Cursor };

use cfg_if::cfg_if;
use cgmath::{ vec3, Rotation3 };
use wgpu::util::DeviceExt;
use crate::engine::model::material::Material;
use crate::engine::model::mesh;
use crate::engine::state::context::GpuContext;

use super::instance::Instance;
use super::model::mesh::{ triangles_to_lines, Mesh };
use super::model::vertex::ModelVertex;
use super::texture;
use super::model::model::Model;

#[cfg(target_arch = "wasm32")]
fn format_url(file_name: &str) -> reqwest::Url {
    let window = web_sys::window().unwrap();
    let location = window.location();
    let origin = location.origin().unwrap();

    // Models will be served under /res/
    let base = reqwest::Url::parse(&format!("{}/pkg/res/", origin)).unwrap();
    base.join(file_name).unwrap()
}

pub async fn load_string(file_name: &str) -> anyhow::Result<String> {
    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let url = format_url(file_name);

            let txt = reqwest::get(url).await?.text().await?;
        } else {
            let path = std::path::Path::new(env!("OUT_DIR")).join("res").join(file_name);
            let txt = std::fs::read_to_string(path)?;
        }
    }
    Ok(txt)
}

pub async fn load_binary(file_name: &str) -> anyhow::Result<Vec<u8>> {
    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let url = format_url(file_name);
            let data = reqwest::get(url).await?.bytes().await?.to_vec();
        } else {
            let path = std::path::Path::new(env!("OUT_DIR")).join("res").join(file_name);
            let data = std::fs::read(path)?;
        }
    }

    Ok(data)
}

pub async fn load_texture_from_file(
    file_name: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue
) -> anyhow::Result<texture::Texture> {
    let data = load_binary(file_name).await?;
    Ok(texture::Texture::from_bytes(device, queue, &data, file_name)?)
}

pub fn load_mesh_from_arrays(
    label: &str,
    vertices: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    triangle_indices: Vec<u32>,
    gpu_context: &GpuContext<'_>,
    material: Material,
    instances: Option<Vec<Instance>>
) -> Mesh {
    let device = gpu_context.device;
    let model_vertices: Vec<ModelVertex>;

    if normals.is_empty() {
        let generated_normals = mesh::calculate_normals(&vertices, &triangle_indices);
        model_vertices = (0..vertices.len())
            .map(|i| {
                ModelVertex {
                    position: [vertices[i][0], vertices[i][1], vertices[i][2]],
                    tex_coords: [0.0, 0.0],
                    normal: generated_normals[i],
                }
            })
            .collect::<Vec<_>>();
    } else {
        model_vertices = (0..vertices.len())
            .map(|i| {
                ModelVertex {
                    position: vertices[i],
                    tex_coords: [0.0, 0.0],
                    normal: normals[i],
                }
            })
            .collect::<Vec<_>>();
    }

    let vertex_buffer = device.create_buffer_init(
        &(wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{:?} Vertex Buffer", label)),
            contents: bytemuck::cast_slice(&model_vertices),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        })
    );

    let index_buffer = device.create_buffer_init(
        &(wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{:?} Index Buffer", label)),
            contents: bytemuck::cast_slice(&triangle_indices),
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        })
    );

    let wireframe_indices = triangles_to_lines(&triangle_indices);
    let line_index_buffer = device.create_buffer_init(
        &(wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{:?} Line Index Buffer", label)),
            contents: bytemuck::cast_slice(&wireframe_indices),
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        })
    );

    let initial_instances = if instances.is_none() {
        Some(
            vec![Instance {
                position: cgmath::Vector3 { x: 0.0, y: 0.0, z: 0.0 },
                rotation: cgmath::Quaternion::from_axis_angle(
                    (1.0, 1.0, 1.0).into(),
                    cgmath::Deg(0.0)
                ),
                scale: cgmath::Vector3 { x: 1.0, y: 1.0, z: 1.0 },
            }]
        )
    } else {
        instances
    };

    let mesh = Mesh::new(
        label.to_string(),
        vertex_buffer,
        index_buffer,
        line_index_buffer,
        wireframe_indices.len() as u32,
        triangle_indices.len() as u32,
        initial_instances,
        device,
        material
    );

    mesh
}

pub async fn load_model_from_file(file_name: &str, device: &wgpu::Device) -> anyhow::Result<Model> {
    let obj_text = load_string(file_name).await?;
    let obj_cursor = Cursor::new(obj_text);
    let mut obj_reader = BufReader::new(obj_cursor);

    let (models, materials_result) = tobj::load_obj_buf_async(
        &mut obj_reader,
        &(tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        }),
        |p| async move {
            let mat_text = load_string(&p).await.unwrap();
            tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(mat_text)))
        }
    ).await?;

    // Extract individual meshes from model file with normals + textures
    let meshes = models
        .into_iter()
        .map(|m| {
            let vertex_count = m.mesh.positions.len() / 3;
            let normal_count = if m.mesh.normals.is_empty() { 0 } else { m.mesh.normals.len() / 3 };

            let vertices = (0..vertex_count)
                .map(|i| {
                    if m.mesh.normals.is_empty() || i >= normal_count {
                        // Use default normal if no normals or index out of bounds
                        ModelVertex {
                            position: [
                                m.mesh.positions[i * 3],
                                m.mesh.positions[i * 3 + 1],
                                m.mesh.positions[i * 3 + 2],
                            ],
                            tex_coords: [0.0, 0.0],
                            normal: [0.0, 1.0, 0.0], // Default up normal
                        }
                    } else {
                        ModelVertex {
                            position: [
                                m.mesh.positions[i * 3],
                                m.mesh.positions[i * 3 + 1],
                                m.mesh.positions[i * 3 + 2],
                            ],
                            tex_coords: [0.0, 0.0],
                            normal: [
                                m.mesh.normals[i * 3],
                                m.mesh.normals[i * 3 + 1],
                                m.mesh.normals[i * 3 + 2],
                            ],
                        }
                    }
                })
                .collect::<Vec<_>>();

            let vertex_buffer = device.create_buffer_init(
                &(wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("{:?} Vertex Buffer", file_name)),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                })
            );

            let index_buffer = device.create_buffer_init(
                &(wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("{:?} Index Buffer", file_name)),
                    contents: bytemuck::cast_slice(&m.mesh.indices),
                    usage: wgpu::BufferUsages::INDEX,
                })
            );
            let wireframe_indices = triangles_to_lines(&m.mesh.indices);

            let line_index_buffer = device.create_buffer_init(
                &(wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("{:?} Line Index Buffer", file_name)),
                    contents: bytemuck::cast_slice(&triangles_to_lines(&wireframe_indices)),
                    usage: wgpu::BufferUsages::INDEX,
                })
            );

            // Use material from OBJ if available, otherwise default to white
            let material: Material = {
                match &materials_result {
                    Ok(materials) => {
                        match m.mesh.material_id {
                            Some(material_id) if material_id < materials.len() => {
                                let mesh_material_diffuse = materials[material_id].diffuse;
                                let to_rgb = [
                                    (mesh_material_diffuse[0] * 255.0) as u32,
                                    (mesh_material_diffuse[1] * 255.0) as u32,
                                    (mesh_material_diffuse[2] * 255.0) as u32,
                                ];

                                Material::new(to_rgb, materials[material_id].dissolve)
                            }
                            Some(material_id) => {
                                println!("Material ID {} out of bounds", material_id);
                                Material::new([255, 255, 255], 1.0)
                            }
                            None => { Material::new([255, 255, 255], 1.0) }
                        }
                    }
                    Err(load_error) => {
                        println!("Failed to load materials: {:?}", load_error);
                        Material::new([255, 255, 255], 1.0)
                    }
                }
            };

            Mesh::new(
                file_name.to_string(),
                vertex_buffer,
                index_buffer,
                line_index_buffer,
                wireframe_indices.len() as u32,
                m.mesh.indices.len() as u32,
                Some(
                    vec![Instance { // Default instantiate at world origin
                        position: vec3(0.0, 0.0, 0.0),
                        rotation: cgmath::Quaternion::from_axis_angle(
                            (1.0, 0.0, 0.0).into(),
                            cgmath::Deg(0.0)
                        ),
                        scale: vec3(1.0, 1.0, 1.0),
                    }]
                ),
                device,
                material
            )
        })
        .collect::<Vec<_>>();

    Ok(Model { meshes })
}
