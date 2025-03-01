use std::io::{ BufReader, Cursor };

use cfg_if::cfg_if;
use wgpu::util::DeviceExt;

use crate::engine::model::mesh;

use super::instance::InstanceRaw;
use super::model::material::Material;
use super::model::mesh::Mesh;
use super::model::normals::calculate_normal_for_triangle;
use super::model::vertex::ModelVertex;
use super::texture;
use super::model::model::Model;

#[cfg(target_arch = "wasm32")]
fn format_url(file_name: &str) -> reqwest::Url {
    let window = web_sys::window().unwrap();
    let location = window.location();
    let mut origin = location.origin().unwrap();
    if !origin.ends_with("learn-wgpu") {
        origin = format!("{}/learn-wgpu", origin);
    }
    let base = reqwest::Url::parse(&format!("{}/", origin)).unwrap();
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

pub fn load_model_from_arrays(
    label: &str,
    vertices: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    triangle_indices: Vec<u32>,
    device: &wgpu::Device
) -> Model {
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
            usage: wgpu::BufferUsages::VERTEX,
        })
    );

    let index_buffer = device.create_buffer_init(
        &(wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{:?} Index Buffer", label)),
            contents: bytemuck::cast_slice(&triangle_indices),
            usage: wgpu::BufferUsages::INDEX,
        })
    );

    let mesh = Mesh {
        label: label.to_string(),
        vertex_buffer,
        index_buffer,
        num_elements: triangle_indices.len() as u32,
    };

    Model { meshes: vec![mesh] }
}

pub async fn load_model_from_file(file_name: &str, device: &wgpu::Device) -> anyhow::Result<Model> {
    let obj_text = load_string(file_name).await?;
    let obj_cursor = Cursor::new(obj_text);
    let mut obj_reader = BufReader::new(obj_cursor);

    let (models, obj_materials) = tobj::load_obj_buf_async(
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
            let vertices = (0..m.mesh.positions.len() / 3)
                .map(|i| {
                    if m.mesh.normals.is_empty() {
                        ModelVertex {
                            position: [
                                m.mesh.positions[i * 3],
                                m.mesh.positions[i * 3 + 1],
                                m.mesh.positions[i * 3 + 2],
                            ],
                            tex_coords: [0.0, 0.0],
                            normal: [0.0, 0.0, 0.0],
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

            Mesh {
                label: file_name.to_string(),
                vertex_buffer,
                index_buffer,
                num_elements: m.mesh.indices.len() as u32,
            }
        })
        .collect::<Vec<_>>();

    Ok(Model { meshes })
}
