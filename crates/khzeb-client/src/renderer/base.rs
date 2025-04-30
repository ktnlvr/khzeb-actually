use wgpu::{
    BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BlendState, ColorTargetState,
    ColorWrites, Device, Face, FragmentState, FrontFace, MultisampleState,
    PipelineCompilationOptions, PipelineLayout, PolygonMode, PrimitiveState, PrimitiveTopology,
    RenderPipeline, RenderPipelineDescriptor, ShaderModule, ShaderStages, SurfaceConfiguration,
    VertexState,
};

use super::instance::BatchInstance;

/// Basic objects commonly used for rendering specific primitives
pub struct RendererBase {
    pub shader_ctx_bind_group_layout: BindGroupLayout,

    pub batch_shader: ShaderModule,
    pub batch_shader_pipeline_layout: PipelineLayout,
    pub batch_shader_pipeline: RenderPipeline,
}

impl RendererBase {
    pub fn new(device: &Device, surface_config: &SurfaceConfiguration) -> Self {
        let shader_ctx_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: None,
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let batch_shader = device.create_shader_module(wgpu::include_wgsl!("shaders/batch.wgsl"));

        let batch_shader_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&shader_ctx_bind_group_layout],
                push_constant_ranges: &[],
            });

        let batch_shader_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&batch_shader_pipeline_layout),
            vertex: VertexState {
                module: &batch_shader,
                entry_point: Some("vs_main"),
                buffers: &[BatchInstance::vertex_buffer_layout()],
                compilation_options: PipelineCompilationOptions::default(),
            },
            fragment: Some(FragmentState {
                module: &batch_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(ColorTargetState {
                    format: surface_config.format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: PipelineCompilationOptions::default(),
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
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
        });

        Self {
            shader_ctx_bind_group_layout,

            batch_shader,
            batch_shader_pipeline_layout,
            batch_shader_pipeline,
        }
    }
}
