use std::ops::Deref;

use wgpu::{
    BlendState, ColorTargetState, ColorWrites, Device, Face, FragmentState, FrontFace,
    MultisampleState, PipelineCompilationOptions, PolygonMode, PrimitiveState, PrimitiveTopology,
    RenderPipeline, RenderPipelineDescriptor, ShaderModule, TextureFormat, VertexBufferLayout,
    VertexState,
};

use super::bindings::BindingLayout;

pub struct Pipeline {
    render_pipeline: RenderPipeline,
}

impl Deref for Pipeline {
    type Target = RenderPipeline;

    fn deref(&self) -> &Self::Target {
        &self.render_pipeline
    }
}

pub fn create_render_pipeline<'all>(
    device: &'all Device,
    module: &'all ShaderModule,
    binding_layouts: impl IntoIterator<Item = &'all BindingLayout>,
    format: TextureFormat,
    buffer_layouts: impl IntoIterator<Item = VertexBufferLayout<'all>>,
) -> Pipeline {
    let bindings = binding_layouts
        .into_iter()
        .map(|layout| layout.layout.as_ref())
        .collect::<Vec<_>>();

    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &bindings[..],
        push_constant_ranges: &[],
    });

    let buffers = buffer_layouts.into_iter().collect::<Vec<_>>();

    let targets = &[Some(ColorTargetState {
        format,
        blend: Some(BlendState::ALPHA_BLENDING),
        write_mask: ColorWrites::ALL,
    })];

    let descriptor = RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&layout),
        vertex: VertexState {
            module,
            entry_point: Some("vertex_main"),
            buffers: &buffers[..],
            compilation_options: PipelineCompilationOptions::default(),
        },
        fragment: Some(FragmentState {
            module,
            entry_point: Some("fragment_main"),
            targets,
            compilation_options: PipelineCompilationOptions::default(),
        }),
        primitive: PrimitiveState {
            topology: PrimitiveTopology::TriangleStrip,
            strip_index_format: None,
            front_face: FrontFace::Ccw,
            cull_mode: Some(Face::Back),
            polygon_mode: PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
        cache: None,
    };

    let render_pipeline = device.create_render_pipeline(&descriptor);

    Pipeline { render_pipeline }
}
