pub mod batch;
pub mod camera;
pub mod dirty;
pub mod instance;

use std::sync::Arc;

use batch::Batch;
use bytemuck::{Pod, Zeroable};
use camera::Camera;
use instance::BatchInstance;
use pollster::FutureExt;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Backends, BindGroup, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BlendState, Buffer,
    BufferUsages, ColorTargetState, ColorWrites, Device, DeviceDescriptor, Face, Features,
    FragmentState, FrontFace, Instance, InstanceDescriptor, Limits, MultisampleState,
    PipelineCompilationOptions, PolygonMode, PowerPreference, PrimitiveState, PrimitiveTopology,
    Queue, RenderPipeline, RenderPipelineDescriptor, RequestAdapterOptionsBase, ShaderStages,
    Surface, SurfaceConfiguration, VertexState,
};
use winit::{dpi::PhysicalSize, window::Window};

pub struct Renderer<'surface, 'window: 'surface> {
    surface: Surface<'surface>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    size: PhysicalSize<u32>,
    window: &'window Window,

    camera: Camera,
    shader_ctx_buffer: Arc<Buffer>,
    shader_ctx_bind_group: BindGroup,

    render_pipeline: RenderPipeline,

    batches: Vec<Arc<Batch>>,
}

#[derive(Debug, Clone, Copy, Pod, Zeroable, Default)]
#[repr(C)]
pub struct ShaderContext {
    view_projection: [f32; 16],
}

impl<'surface, 'window> Renderer<'surface, 'window> {
    pub fn new(window: &'window Window) -> Self {
        let size = window.inner_size();

        let instance = Instance::new(&InstanceDescriptor {
            backends: Backends::VULKAN,
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance
            .request_adapter(&RequestAdapterOptionsBase {
                power_preference: PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .block_on()
            .expect("Failed to find an adapter");

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: None,
                    required_features: Features::empty(),
                    required_limits: Limits::default(),
                    memory_hints: Default::default(),
                },
                None,
            )
            .block_on()
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::include_wgsl!("shaders/standard.wgsl"));

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

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&shader_ctx_bind_group_layout],
                push_constant_ranges: &[],
            });

        let camera = Camera::default();

        let shader_ctx_buffer = Arc::new(device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[ShaderContext::default()]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        }));

        let shader_ctx_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &shader_ctx_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: shader_ctx_buffer.as_entire_binding(),
            }],
            label: Some("Camera Bind Group"),
        });

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[BatchInstance::vertex_buffer_layout()],
                compilation_options: PipelineCompilationOptions::default(),
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(ColorTargetState {
                    format: config.format,
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

        let batches = vec![];

        Self {
            surface,
            device,
            queue,
            config,
            size,
            window,

            render_pipeline,

            camera,
            shader_ctx_buffer,
            shader_ctx_bind_group,

            batches,
        }
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.camera.aspect_ratio = new_size.width as f32 / new_size.height as f32;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn render(&mut self) {
        let output = self.surface.get_current_texture().unwrap();

        let shader_ctx = ShaderContext {
            view_projection: self.camera.bake().to_cols_array(),
        };

        let shader_ctx_data = [shader_ctx];
        let shader_ctx_data: &[u8] = bytemuck::cast_slice(&shader_ctx_data);
        self.queue
            .write_buffer(&self.shader_ctx_buffer, 0, shader_ctx_data);

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_bind_group(0, &self.shader_ctx_bind_group, &[]);
            render_pass.set_pipeline(&self.render_pipeline);

            for batch in &self.batches {
                render_pass.set_vertex_buffer(0, batch.buffer_slice());
                render_pass.draw(0..6, 0..(batch.size() as u32));
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));

        output.present();
    }

    pub fn transfer_queue(&self) -> &Queue {
        &self.queue
    }
}

impl<'surface, 'window> Renderer<'surface, 'window> {
    pub fn create_batch(&mut self, max_size: usize) -> Arc<Batch> {
        let batch = Batch::new(max_size, &self.device);
        let arc_batch = Arc::new(batch);
        self.batches.push(arc_batch.clone());
        arc_batch
    }
}
