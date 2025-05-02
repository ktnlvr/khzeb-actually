pub mod base;
pub mod batch;
pub mod camera;
pub mod dirty;
pub mod instance;

use std::sync::Arc;

use base::RendererBase;
use batch::Batch;
use bytemuck::{Pod, Zeroable};
use camera::Camera;
use pollster::FutureExt;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Backends, BindGroup, Buffer, BufferUsages, Device, DeviceDescriptor, Features, Instance,
    InstanceDescriptor, Limits, PowerPreference, Queue, RequestAdapterOptionsBase, Surface,
    SurfaceConfiguration,
};
use winit::{dpi::PhysicalSize, window::Window};

use khzeb::{Name, Registry, Resource};

pub struct Renderer<'surface, 'window: 'surface> {
    surface: Surface<'surface>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    size: PhysicalSize<u32>,
    window: &'window Window,

    resources: Registry,

    camera: Camera,

    lookup: LookupTable,

    base: RendererBase,

    batches: Vec<Arc<Batch>>,
}

struct LookupTable {
    shader_context_buffer: Resource<Arc<Buffer>>,
    shader_context_bind_group: Resource<BindGroup>,
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

        let mut resources = Registry::new();

        let base = RendererBase::new(&device, &config);

        let _shader_context_buffer = Arc::new(device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[ShaderContext::default()]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        }));

        let shader_context_bind_group = resources.put(
            Name::new("renderer/shader-context-bind-group"),
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &base.shader_ctx_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: _shader_context_buffer.as_entire_binding(),
                }],
                label: Some("renderer/shader-context-bind-group"),
            }),
        );

        let shader_context_buffer = resources.put(
            Name::new("renderer/shader-context-buffer"),
            _shader_context_buffer,
        );

        let lookup = LookupTable {
            shader_context_buffer,
            shader_context_bind_group,
        };

        let batches = vec![];

        let camera = Camera::new();

        Self {
            surface,
            device,
            queue,
            config,
            size,
            window,

            base,
            batches,

            camera,
            resources,
            lookup,
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

        let shader_ctx_buffer = self
            .resources
            .get(self.lookup.shader_context_buffer.clone())
            .unwrap();

        self.queue
            .write_buffer(&shader_ctx_buffer, 0, shader_ctx_data);

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

            let shader_ctx_bind_group = self
                .resources
                .get(self.lookup.shader_context_bind_group.clone())
                .unwrap();

            // Render the batches
            render_pass.set_bind_group(0, shader_ctx_bind_group, &[]);
            render_pass.set_pipeline(&self.base.batch_shader_pipeline);

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
