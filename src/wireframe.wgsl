struct CameraUniformBuffer {
    view_projection: mat4x4<f32>,
    position: vec3<f32>,
    _padding: f32,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniformBuffer;

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
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) camera_distance: f32, 
}

@vertex
fn vs_main(
    model: VerexInput,
    instance: InstanceInput,
) -> VertexOutput {
     let instance_model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );
    let instance_normal_matrix = mat3x3<f32>(
        instance.normal_matrix_0,
        instance.normal_matrix_1,
        instance.normal_matrix_2,
    );
    
    var out: VertexOutput;
    // transform the normal(model.normal) to match the transformation of the instance(instance_normal_matrix)
    // transform the vertices(model.position) to match the transformation of the instance(instance_model_matrix)
    var world_position: vec4<f32> = instance_model_matrix * vec4<f32>(model.position, 1.0);
    out.world_position = world_position.xyz;

    // Calculate distance for fade
    out.camera_distance = length(world_position.xyz - camera.position);

    out.clip_position = camera.view_projection * instance_model_matrix * vec4<f32>(model.position, 1.0);
    return out;    
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let fade_start = 50.0;
    let fade_end = 100.0;

    let fade_factor = 1.0 - smoothstep(fade_start, fade_end, in.camera_distance);
    
    // pink line with opacity fade
    return vec4<f32>(0.93, 0.11, 1.0, fade_factor);
}