//Vertex Shader
struct CameraUniformBuffer {
    view_projection: mat4x4<f32>,
};
@group(1) @binding(0)
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
    @location(1) texture_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) texture_coords: vec2<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) world_position: vec3<f32>,
}

struct Light {
    position: vec3<f32>,
    color: vec3<f32>,
}
@group(2) @binding(0)
var<uniform> light: Light;

@vertex
fn vs_main(
    model: VerexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );
    let normal_matrix = mat3x3<f32>(
        instance.normal_matrix_0,
        instance.normal_matrix_1,
        instance.normal_matrix_2,
    );
    var out: VertexOutput;
    out.texture_coords = model.texture_coords;
    out.world_normal = normal_matrix * model.normal;
    var world_position: vec4<f32> = model_matrix * vec4<f32>(model.position, 1.0);
    out.world_position = world_position.xyz;
    out.clip_position = camera.view_projection * model_matrix * vec4<f32>(model.position, 1.0);
    return out;    
}

// Fragment Shader
@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let object_color: vec4<f32> = textureSample(t_diffuse, s_diffuse, in.texture_coords);

    let ambient_light_strength = 0.1;
    let ambient_light_color = light.color * ambient_light_strength;

    let light_direction = normalize(light.position - in.world_position);

    let diffuse_strength = max(dot(in.world_normal, light_direction), 0.0);
    let diffuse_color = light.color * diffuse_strength;

    let result = (ambient_light_color + diffuse_color) * object_color.xyz;

    return vec4<f32>(result, object_color.a);
}