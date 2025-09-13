struct Camera2DUniform {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: Camera2DUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
};

struct InstanceInput {
    @location(2) position_offset: vec2<f32>,
    @location(3) scale: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput
) -> VertexOutput {
    var out: VertexOutput;

    // First, apply the instance-specific scale to the vertex position
    let scaled_position = vec3<f32>(model.position.x * instance.scale.x, model.position.y * instance.scale.y, model.position.z);

    // Then, apply the instance-specific position offset
    let translated_position = scaled_position + vec3<f32>(instance.position_offset.x, instance.position_offset.y, 0.0);

    // Combine with the camera's view_proj matrix
    out.clip_position = camera.view_proj * vec4<f32>(translated_position, 1.0);

    out.color = model.color;
    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color);
}