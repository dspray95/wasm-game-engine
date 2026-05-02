use std::io::{ BufReader, Cursor };

use crate::engine::{
    instance::Instance,
    model::{ material::Material, model::{ Model, ModelBounds } },
    resources::load_mesh_from_arrays,
    state::context::GpuContext,
};

pub fn load_model_from_obj_bytes(
    obj_bytes: &[u8],
    mtl_bytes: &[u8],
    gpu_context: &GpuContext,
    initial_instances: Option<Vec<Instance>>,
    max_instances: usize
) -> Model {
    let (raw_models, materials_result) = tobj
        ::load_obj_buf(
            &mut BufReader::new(Cursor::new(obj_bytes)),
            &(tobj::LoadOptions {
                triangulate: true,
                single_index: true,
                ..Default::default()
            }),
            |_path| { tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(mtl_bytes))) }
        )
        .expect("Failed to parse OBJ");

    let mut all_vertices: Vec<[f32; 3]> = Vec::new();

    let meshes = raw_models
        .into_iter()
        .map(|raw_model| {
            let vertex_count = raw_model.mesh.positions.len() / 3;

            let vertices: Vec<[f32; 3]> = (0..vertex_count)
                .map(|i| {
                    [
                        raw_model.mesh.positions[i * 3],
                        raw_model.mesh.positions[i * 3 + 1],
                        raw_model.mesh.positions[i * 3 + 2],
                    ]
                })
                .collect();

            all_vertices.extend(vertices.iter().copied());

            let normals: Vec<[f32; 3]> = if raw_model.mesh.normals.is_empty() {
                vec![]
            } else {
                (0..vertex_count)
                    .map(|i| {
                        [
                            raw_model.mesh.normals[i * 3],
                            raw_model.mesh.normals[i * 3 + 1],
                            raw_model.mesh.normals[i * 3 + 2],
                        ]
                    })
                    .collect()
            };

            let material = match (&materials_result, raw_model.mesh.material_id) {
                (Ok(materials), Some(material_id)) if material_id < materials.len() => {
                    let diffuse = materials[material_id].diffuse;
                    Material {
                        diffuse_color: [
                            (diffuse[0] * 255.0).round() as u32,
                            (diffuse[1] * 255.0).round() as u32,
                            (diffuse[2] * 255.0).round() as u32,
                        ],
                        alpha: materials[material_id].dissolve,
                    }
                }
                _ => Material::new([255, 255, 255], 1.0),
            };

            load_mesh_from_arrays(
                &raw_model.name,
                vertices,
                normals,
                raw_model.mesh.indices.clone(),
                gpu_context,
                material,
                initial_instances.clone(),
                max_instances
            )
        })
        .collect();

    let bounds = ModelBounds::from_vertices(all_vertices);

    Model { meshes, bounds }
}
