struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) fragColor: vec3<f32>,
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
    let pos = positions[vertex_index];
    output.position = vec4<f32>(pos, 0.0, 1.0);
    output.fragColor = vec3<f32>(0.2, 0.2, 1.0); // light blue
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(input.fragColor, 1.0);
}
