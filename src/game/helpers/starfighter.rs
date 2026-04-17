use crate::{
    engine::{
        ecs::components::{ transform::Transform, velocity::Velocity },
        model::{ material::Material, mesh::{ MeshData, calculate_normals }, model::Model },
        resources,
        state::context::GpuContext,
    },
    game::components::hover_state::{ HoverDirection, HoverState },
};

const HOVER_SPEED: f32 = 0.2;

pub fn animate_hover(transform: &Transform, velocity: &mut Velocity, hover_state: &mut HoverState) {
    if transform.position.y > hover_state.upper_limit {
        hover_state.direction = HoverDirection::Down;
    } else if transform.position.y < hover_state.lower_limit {
        hover_state.direction = HoverDirection::Up;
    }

    if hover_state.direction == HoverDirection::Up {
        velocity.y += HOVER_SPEED;
    } else {
        velocity.y -= HOVER_SPEED;
    }
}

pub fn load_model(gpu_context: &GpuContext) -> Model {
    let fuselage_vertices = vec![
        [0.000001, -0.0, -1.853511],
        [0.35669, -0.0, -1.140133],
        [0.000001, 0.142676, -1.140133],
        [0.285351, 0.142676, 0.286622],
        [0.000001, 0.356689, -0.070067],
        [0.0, -0.0, 0.643311],
        [1.070067, -0.0, 1.0],
        [-0.356688, -0.0, -1.140133],
        [-1.070067, 0.0, 0.999999],
        [-0.285351, 0.142676, 0.286622]
    ];

    let fuselage_triangles = vec![
        0,
        2,
        1,
        2,
        3,
        1,
        4,
        5,
        3,
        1,
        3,
        6,
        7,
        2,
        0,
        7,
        9,
        2,
        5,
        6,
        3,
        9,
        5,
        4,
        8,
        5,
        9,
        8,
        9,
        7
    ];

    let fuselage_mesh_data = MeshData {
        normals: calculate_normals(&fuselage_vertices, &fuselage_triangles),
        vertices: fuselage_vertices,
        triangles: fuselage_triangles,
        material: Material {
            diffuse_color: [0, 255, 255],
            alpha: 1.0,
        },
    };

    let cockpit_vertices = vec![
        [0.000001, 0.142676, -1.140133],
        [0.285351, 0.142676, 0.286622],
        [0.000001, 0.356689, -0.070067],
        [-0.285351, 0.142676, 0.286622]
    ];

    let cockpit_triangles = vec![0, 2, 1, 0, 3, 2];
    let cockpit_mesh_data = MeshData {
        normals: calculate_normals(&cockpit_vertices, &cockpit_triangles),
        vertices: cockpit_vertices,
        triangles: cockpit_triangles,
        material: Material {
            diffuse_color: [0, 0, 0],
            alpha: 1.0,
        },
    };

    let fuselage_mesh = resources::load_mesh_from_arrays(
        "Starfighter Fuselage",
        fuselage_mesh_data.vertices,
        fuselage_mesh_data.normals,
        fuselage_mesh_data.triangles,
        &gpu_context,
        fuselage_mesh_data.material,
        None,
        1
    );

    let cockpit_mesh = resources::load_mesh_from_arrays(
        "Starfighter Cockpit",
        cockpit_mesh_data.vertices,
        cockpit_mesh_data.normals,
        cockpit_mesh_data.triangles,
        gpu_context,
        cockpit_mesh_data.material,
        None,
        1
    );

    Model {
        meshes: vec![fuselage_mesh, cockpit_mesh],
    }
}
