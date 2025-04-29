struct ShaderContext {
    viewProj: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> shader_ctx: ShaderContext;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) frag_color: vec4<f32>,
};

fn unpack_u32_to_rgba(color: u32) -> vec4<f32> {
    let r: u32 = (color >> 24) & 0xFF;
    let g: u32 = (color >> 16) & 0xFF;
    let b: u32 = (color >> 8) & 0xFF;
    let a: u32 = color & 0xFF;

    return vec4<f32>(
        f32(r) / 255.0,
        f32(g) / 255.0,
        f32(b) / 255.0,
        f32(a) / 255.0
    );
}

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    @location(0) instance_position: vec2<i32>,
    @location(1) instance_scale: f32,
    @location(2) instance_color: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 6>(
        vec2<f32>(-0.5, -0.5),
        vec2<f32>(0.5, -0.5),
        vec2<f32>(-0.5, 0.5),
        vec2<f32>(-0.5, 0.5),
        vec2<f32>(0.5, -0.5),
        vec2<f32>(0.5, 0.5),
    );

    var output: VertexOutput;
    var pos = vec4<f32>(instance_scale * positions[vertex_index] + bitcast<vec2<f32>>(instance_position), 0.0, 1.0);
    output.position = shader_ctx.viewProj * pos;
    output.frag_color = unpack_u32_to_rgba(instance_color);
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return input.frag_color;
}
