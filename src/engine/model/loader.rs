use crate::engine::{
    model::{ descriptor::ModelDescriptor, material::Material, model::Model },
    resources::load_mesh_from_arrays,
    state::context::GpuContext,
};

pub fn load_model_from_descriptor(descriptor: &ModelDescriptor, gpu_context: &GpuContext) -> Model {
    let meshes = descriptor.meshes
        .iter()
        .map(|mesh_descriptor| {
            load_mesh_from_arrays(
                &mesh_descriptor.label,
                mesh_descriptor.vertices.iter().map(|(x, y, z)| [*x, *y, *z]).collect(),
                vec![], // normals computed automatically
                mesh_descriptor.triangles.clone(),
                gpu_context,
                Material {
                    diffuse_color: [
                        mesh_descriptor.material.diffuse_color.0,
                        mesh_descriptor.material.diffuse_color.1,
                        mesh_descriptor.material.diffuse_color.2,
                    ],
                    alpha: mesh_descriptor.material.alpha,
                },
                None,
                mesh_descriptor.max_instances
            )
        })
        .collect();
    Model { meshes }
}
