struct Camera {
    view_proj: mat4x4<f32>,
}
@group(1) @binding(0)
var<uniform> camera: Camera;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec4<f32>,
    @location(3) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(2) color: vec4<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
    out.color = model.color;
    return out;
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0)@binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Get the color from the texture
    var final_color = textureSample(t_diffuse, s_diffuse, in.tex_coords);

    // Combine the texture color with the vertex color.
    // WGSL does not support assignment to a swizzle, so we must
    // create a new vector from the combined components.
    
    // Calculate the combined RGB
    let combined_rgb = final_color.rgb * in.color.rgb;

    // Calculate the combined alpha
    let combined_alpha = final_color.a * in.color.a;

    // Premultiply the RGB by the combined alpha
    let premultiplied_rgb = combined_rgb * combined_alpha;

    // Create the final vec4 with the premultiplied RGB and the combined alpha
    final_color = vec4<f32>(premultiplied_rgb, combined_alpha);
    
    return final_color;
}