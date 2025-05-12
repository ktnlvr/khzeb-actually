struct ShaderContext {
    view_projection: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> shader_ctx: ShaderContext;

struct TileAtlas {
    size: u32,
    tile: u32,
};

@group(1) @binding(0)
var<uniform> tile_atlas: TileAtlas;
@group(1) @binding(1)
var s_diffuse: sampler;
@group(1) @binding(2)
var t_diffuse: texture_2d<f32>;

struct BatchMetadata {
    flags: u32,
    origin: vec2<f32>,
    scale: f32,
    zorder: u32,
}

@group(2) @binding(0)
var<uniform> batch_metadata: BatchMetadata;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tint_color: vec4<f32>,
    @location(1) texture_position: vec2<f32>,
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

fn unpack_u32_to_u16x2(packed: u32) -> vec2<u32> {
    let width: u32 = u32((packed >> 16) & 0xFFFF);
    let height: u32 = u32(packed & 0xFFFF);
    return vec2(width, height);
}

@vertex
fn vertex_main(
    @builtin(vertex_index) vertex_index: u32,
    @location(0) instance_position: vec2<i32>,
    @location(1) instance_scale: f32,
    @location(2) instance_color: u32,
    @location(3) instance_tile_idx: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 4>(
        vec2<f32>(-0.5, -0.5),
        vec2<f32>(0.5, -0.5),
        vec2<f32>(-0.5, 0.5),
        vec2<f32>(0.5, 0.5),
    );

    var output: VertexOutput;
    var pos = vec4<f32>(instance_scale * positions[vertex_index] + bitcast<vec2<f32>>(instance_position), 0.0, 1.0);
    output.position = shader_ctx.view_projection * pos;
    output.tint_color = unpack_u32_to_rgba(instance_color);

    var tile_size = unpack_u32_to_u16x2(tile_atlas.tile);
    var size = unpack_u32_to_u16x2(tile_atlas.size);

    var tiles_per = size / tile_size;

    var col = instance_tile_idx % tiles_per.x + u32(positions[vertex_index].x > 0);
    var row = instance_tile_idx / tiles_per.y + u32(positions[vertex_index].y < 0);
    var tex = vec2<f32>(tile_size) / vec2<f32>(size);
    output.texture_position = tex * vec2(f32(col), f32(row));
    return output;
}

@fragment
fn fragment_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, input.texture_position) * input.tint_color;
}
