struct ShaderCtx {
    viewProj: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> shader_ctx: ShaderCtx;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) frag_color: vec3<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>(-1.0,  1.0),
        vec2<f32>(-1.0,  1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>( 1.0,  1.0),
    );

    var output: VertexOutput;
    var pos = vec4<f32>(positions[vertex_index], 0.0, 1.0);
    output.position = shader_ctx.viewProj * pos;
    output.frag_color = vec3<f32>(0.2, 0.2, 1.0); // light blue
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(input.frag_color, 1.0);
}
