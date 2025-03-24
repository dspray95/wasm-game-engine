struct CameraUniformBuffer {
    view_projection: mat4x4<f32>,
};
@group(0) @binding(0) 
var<uniform> camera: CameraUniformBuffer;

@group(1) @binding(0)  
var<storage, read> indices : array<u32>;

@group(2) @binding(0)
var<storage, read> positions: array<f32>;

struct InstanceInput {
    // model //
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
    // normal //
    @location(9) normal_matrix_0: vec3<f32>,
    @location(10) normal_matrix_1: vec3<f32>,
    @location(11) normal_matrix_2: vec3<f32>,
}

struct VerexInput {
    @builtin(instance_index) instanceID : u32,
    @builtin(vertex_index) vertexID : u32,
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) world_position: vec3<f32>,
}

@vertex
fn vs_main(
    vertex: VerexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var localToElement = array<u32, 6>(0u, 1u, 1u, 2u, 2u, 0u);

    var triangleIndex = vertex.vertexID / 6u;
    var localVertexIndex = vertex.vertexID % 6u;

    var elementIndexIndex = 3u * triangleIndex + localToElement[localVertexIndex];
    var elementIndex = indices[elementIndexIndex];    

     var position = vec3<f32>(
        positions[3u * elementIndex + 0u],
        positions[3u * elementIndex + 1u],
        positions[3u * elementIndex + 2u],
    );

    let instance_model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    var out: VertexOutput;

    var world_position: vec4<f32> = instance_model_matrix * vec4<f32>(position, 1.0);
    out.world_position = world_position.xyz;

    out.clip_position = camera.view_projection * instance_model_matrix * vec4<f32>(position, 1.0);
    return out;    
}

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    return vec4(1.0, 0.0, 1.0, 1.0);
}