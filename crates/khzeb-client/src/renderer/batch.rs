use std::sync::Mutex;

use bitflags::bitflags;
use bytemuck::{Pod, Zeroable};
use glam::{IVec2, Vec2};
use lazy_static::lazy_static;
use micromap::Map;
use wgpu::{
    vertex_attr_array, BindingResource, BindingType, Buffer, BufferAddress, BufferBindingType,
    BufferDescriptor, BufferSlice, BufferUsages, Device, Queue, ShaderStages, VertexAttribute,
    VertexBufferLayout, VertexStepMode,
};

use super::{
    bindings::{create_binding, create_binding_layout, Binding, BindingLayout},
    color::Rgba,
    dirty::DirtyFlags,
};

pub const BATCH_DIRTY_FLAG_COUNT: usize = 4;
pub const INSTANCES_PER_REGION: u32 = 16;
pub const MAX_BATCH_SIZE: usize =
    DirtyFlags::<BATCH_DIRTY_FLAG_COUNT>::SIZE * INSTANCES_PER_REGION as usize;
pub const BATCH_VISIBLE_SHADER_STAGES: ShaderStages = ShaderStages::VERTEX;

bitflags! {
    #[derive(Debug, Clone, Copy, Pod, Zeroable, Default, PartialEq, Eq)]
    #[repr(C)]
    pub struct BatchMetadataFlags: u32 {
        const SNAP_INSTANCES_TO_GRID = 0b001;
    }
}

#[derive(Debug, Clone, Copy, Pod, Zeroable, PartialEq)]
#[repr(C)]
pub struct BatchMetadata {
    pub flags: BatchMetadataFlags,
    pub origin: Vec2,
    pub scale: f32,
    pub zorder: u32,
}

struct BatchMutableState {
    metadata: BatchMetadata,
    is_metadata_dirty: bool,

    instance_dirty_flag: DirtyFlags<BATCH_DIRTY_FLAG_COUNT>,
    instance_local_array: Box<[BatchInstance]>,
    size: u32,
}

pub struct Batch {
    capacity: usize,
    mutable: Mutex<BatchMutableState>,

    instance_buffer: Buffer,
    metadata_buffer: Buffer,
    binding: Binding,
}

impl Batch {
    pub fn new(device: &Device, capacity: usize, metadata: BatchMetadata) -> Self {
        assert!(
            capacity <= MAX_BATCH_SIZE,
            "Batch too big, requested {capacity}, but can only fit {MAX_BATCH_SIZE}"
        );

        let instance_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Batch Buffer"),
            size: (capacity as u64) * (size_of::<BatchInstance>() as u64),
            usage: BufferUsages::COPY_DST | BufferUsages::VERTEX,
            mapped_at_creation: false,
        });

        let metadata_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Batch Metadata Buffer"),
            size: size_of::<BatchMetadata>() as u64,
            usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });

        let instance_dirty_flag = DirtyFlags::new();
        let instance_local_array = vec![BatchInstance::zeroed(); MAX_BATCH_SIZE].into_boxed_slice();

        let layout = Batch::binding_layout(device);
        let binding = create_binding(device, &layout, [metadata_buffer.as_entire_binding()]);

        Self {
            capacity,
            instance_buffer,
            metadata_buffer,
            mutable: Mutex::new(BatchMutableState {
                instance_dirty_flag,
                size: 0,
                instance_local_array,

                metadata,
                is_metadata_dirty: true,
            }),
            binding,
        }
    }

    pub fn push_unchecked(&self, instance: BatchInstance) {
        let mut mutable = self.mutable.lock().unwrap();

        assert!(
            mutable.size < self.capacity as u32,
            "The new Batch instance does not fit."
        );

        let idx = mutable.size;
        mutable
            .instance_dirty_flag
            .mark((idx / INSTANCES_PER_REGION) as usize);
        mutable.size += 1;
        mutable.instance_local_array[idx as usize] = instance;
    }

    pub fn flush(&self, queue: &Queue) {
        let mut mutable = self.mutable.lock().unwrap();

        if mutable.is_metadata_dirty {
            queue.write_buffer(
                &self.metadata_buffer,
                0,
                bytemuck::cast_slice(&[mutable.metadata]),
            );
        }

        for marked in mutable.instance_dirty_flag.iter_marked() {
            let start = marked * INSTANCES_PER_REGION as usize;
            let end = (marked + 1) * INSTANCES_PER_REGION as usize;

            let byte_offset = start * size_of::<BatchInstance>();

            queue.write_buffer(
                &self.instance_buffer,
                byte_offset as u64,
                bytemuck::cast_slice(&mutable.instance_local_array[start..end]),
            );
        }

        mutable.instance_dirty_flag.clear()
    }

    pub fn buffer_slice(&self) -> BufferSlice {
        let bytes_size = self.size() * size_of::<BatchInstance>() as u64;
        self.instance_buffer.slice(..bytes_size)
    }

    pub fn size(&self) -> u64 {
        let mutable = self.mutable.lock().unwrap();
        mutable.size as u64
    }

    pub fn mutate_metadata(&self, mut mutator: impl FnMut(&mut BatchMetadata)) {
        let mut mutable = self.mutable.lock().unwrap();
        let metadata_before_mutator = mutable.metadata;
        mutator(&mut mutable.metadata);
        mutable.is_metadata_dirty = metadata_before_mutator != mutable.metadata;
    }

    pub fn binding(&self) -> &Binding {
        &self.binding
    }
}

#[derive(Zeroable, Clone, Copy)]
#[repr(C)]
union InstancePosition {
    int: IVec2,
    float: Vec2,
}

unsafe impl Pod for InstancePosition {}

#[derive(Zeroable, Clone, Copy, Pod)]
#[repr(C)]
pub struct BatchInstance {
    position: InstancePosition,
    scale: f32,
    tint: Rgba,
    texture_index: u32,
}

impl Default for BatchInstance {
    fn default() -> Self {
        Self {
            position: InstancePosition { int: IVec2::ZERO },
            scale: 1.,
            tint: Rgba::default(),
            texture_index: 0,
        }
    }
}

impl BatchInstance {
    pub const ATTRIBUTES: [VertexAttribute; 4] = vertex_attr_array![
        0 => Sint32x2,
        1 => Float32,
        2 => Uint32,
        3 => Uint32,
    ];

    pub fn vertex_buffer_layout() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: size_of::<Self>() as BufferAddress,
            step_mode: VertexStepMode::Instance,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

impl BatchInstance {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_i32(position: IVec2, scale: f32) -> Self {
        Self::default()
            .with_position_i32(position)
            .with_scale(scale)
    }

    pub fn new_f32(position: Vec2, scale: f32) -> Self {
        Self::default()
            .with_position_f32(position)
            .with_scale(scale)
    }

    pub fn with_tint(self, tint: Rgba) -> Self {
        Self { tint, ..self }
    }

    pub fn with_scale(self, scale: f32) -> Self {
        Self { scale, ..self }
    }

    pub fn with_texture_idx(self, texture_index: u32) -> Self {
        Self {
            texture_index,
            ..self
        }
    }

    pub fn with_position_f32(self, float: Vec2) -> Self {
        Self {
            position: InstancePosition { float },
            ..self
        }
    }

    pub fn with_position_i32(self, int: IVec2) -> Self {
        Self {
            position: InstancePosition { int },
            ..self
        }
    }
}

lazy_static! {
    static ref BINDING_LAYOUTS: Mutex<Map<(Device, ShaderStages), BindingLayout, 1>> =
        Default::default();
}

impl Batch {
    pub fn binding_layout(device: &Device) -> BindingLayout {
        // NOTE(Artur): having bind group layouts and bind groups as separate things is a sin
        create_binding_layout(
            device,
            BATCH_VISIBLE_SHADER_STAGES,
            [BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            }],
        )
    }
}
