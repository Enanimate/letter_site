struct Camera3DUniform {
    view_proj: mat4x4<f32>,
};

struct ModelUniform {
    model: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: Camera3DUniform;

@group(1) @binding(0)
var<uniform> model_uniform: ModelUniform;

@vertex
fn vs_main(@location(0) position: vec3<f32>) -> @builtin(position) vec4<f32> {
    return camera.view_proj * model_uniform.model * vec4<f32>(position, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    // Hardcoded color (e.g., green)
    return vec4<f32>(0.0, 1.0, 0.0, 1.0);
}




