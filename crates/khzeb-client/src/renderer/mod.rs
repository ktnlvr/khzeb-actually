pub mod atlas;
pub mod batch;
pub mod bindings;
pub mod buffer;
pub mod camera;
pub mod color;
pub mod dirty;
pub mod pipeline;
pub mod texture;

use std::sync::Arc;

use atlas::TextureAtlas;
use batch::{Batch, BatchInstance, BatchMetadata};
use bindings::{create_binding, create_binding_layout, Binding};
use buffer::{create_buffer, BufferHandle};
use bytemuck::{Pod, Zeroable};
use camera::Camera;
use pipeline::{create_render_pipeline, Pipeline};
use pollster::FutureExt;
use texture::Texture;
use wgpu::{
    AddressMode, Backends, BindingResource, BindingType, BufferBindingType, BufferUsages, Device,
    DeviceDescriptor, Features, FilterMode, Instance, InstanceDescriptor, Limits, PowerPreference,
    Queue, RequestAdapterOptionsBase, Sampler, SamplerBindingType, SamplerDescriptor, ShaderStages,
    Surface, SurfaceConfiguration, TextureSampleType, TextureUsages, TextureViewDimension,
};
use winit::{dpi::PhysicalSize, window::Window};

use khzeb::prelude::*;

pub struct Renderer<'surface, 'window: 'surface> {
    surface: Surface<'surface>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    size: PhysicalSize<u32>,
    window: &'window Window,

    universal_sampler: Sampler,
    camera: Camera,

    lookup: LookupTable,

    batches: Vec<Arc<Batch>>,

    texture_registry: Registry,
}

struct LookupTable {
    shader_context_buffer: BufferHandle<ShaderContext>,
    shader_context_bind_group: Binding,
    texture_bind_group: Binding,
    texture_atlas: TextureAtlas,

    batch_pipeline: Pipeline,
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

        let universal_sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("Universal Sampler"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });

        let world00_texture_raw = include_bytes!("./textures/world00.png");
        let world00_texture_image = image::load_from_memory(world00_texture_raw).unwrap();

        let world00_texture = Texture::new(
            &device,
            &queue,
            world00_texture_image,
            TextureUsages::TEXTURE_BINDING,
        );

        let shader_ctx_binding_layout = create_binding_layout(
            &device,
            ShaderStages::VERTEX,
            [BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            }],
        );

        let shader_context_buffer =
            create_buffer::<ShaderContext>(&device, BufferUsages::UNIFORM | BufferUsages::COPY_DST);

        let world00_view = world00_texture.to_view();

        let shader_context_bind_group = create_binding(
            &device,
            &shader_ctx_binding_layout,
            [shader_context_buffer.buffer.as_entire_binding()],
        );

        let texture_atlas = TextureAtlas::new_square(&device, &queue, 32, 8);

        let texture_binding_layout = create_binding_layout(
            &device,
            ShaderStages::VERTEX_FRAGMENT,
            [
                TextureAtlas::binding_type(),
                BindingType::Sampler(SamplerBindingType::Filtering),
                BindingType::Texture {
                    sample_type: TextureSampleType::Float { filterable: true },
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
            ],
        );

        let texture_bind_group = create_binding(
            &device,
            &texture_binding_layout,
            [
                texture_atlas.as_entire_binding(),
                BindingResource::Sampler(&universal_sampler),
                BindingResource::TextureView(&world00_view),
            ],
        );

        let batch_shader = device.create_shader_module(wgpu::include_wgsl!("shaders/batch.wgsl"));

        let batch_binding_layout = Batch::binding_layout(&device);

        let batch_pipeline = create_render_pipeline(
            &device,
            &batch_shader,
            [
                &shader_ctx_binding_layout,
                &texture_binding_layout,
                &batch_binding_layout,
            ],
            config.format,
            [BatchInstance::vertex_buffer_layout()],
        );

        let lookup = LookupTable {
            shader_context_buffer,
            shader_context_bind_group,
            texture_bind_group,
            batch_pipeline,
            texture_atlas,
        };

        let batches = vec![];

        let camera = Camera::new();

        let mut texture_registry = Registry::new();
        texture_registry.put("textures/world00", world00_texture);

        Self {
            surface,
            device,
            queue,
            config,
            size,
            window,
            universal_sampler,

            batches,

            camera,
            lookup,
            texture_registry,
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

        self.queue.write_buffer(
            &self.lookup.shader_context_buffer.buffer,
            0,
            shader_ctx_data,
        );

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

            render_pass.set_bind_group(0, &self.lookup.shader_context_bind_group, &[]);
            render_pass.set_bind_group(1, &self.lookup.texture_bind_group, &[]);
            render_pass.set_pipeline(&self.lookup.batch_pipeline);

            for batch in &self.batches {
                render_pass.set_bind_group(2, batch.binding(), &[]);
                render_pass.set_vertex_buffer(0, batch.buffer_slice());
                render_pass.draw(0..4, 0..(batch.size() as u32));
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
    pub fn create_batch(&mut self, max_size: usize, metadata: BatchMetadata) -> Arc<Batch> {
        let batch = Batch::new(&self.device, max_size, metadata);
        let arc_batch = Arc::new(batch);
        self.batches.push(arc_batch.clone());
        arc_batch
    }
}
