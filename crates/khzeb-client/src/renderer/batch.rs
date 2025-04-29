use std::sync::Mutex;

use bitflags::bitflags;
use bytemuck::{Pod, Zeroable};
use glam::{UVec2, Vec2};
use wgpu::{Buffer, BufferDescriptor, BufferSlice, BufferUsages, Device, Queue};

use super::{dirty::DirtyFlags, instance::BatchInstance};

pub const BATCH_DIRTY_FLAG_COUNT: usize = 4;
pub const INSTANCES_PER_REGION: u32 = 16;
pub const MAX_BATCH_SIZE: usize =
    DirtyFlags::<BATCH_DIRTY_FLAG_COUNT>::SIZE * INSTANCES_PER_REGION as usize;

bitflags! {
    #[derive(Debug, Clone, Copy, Pod, Zeroable, Default)]
    #[repr(C)]
    struct BatchFlags: u32 {
        const SNAP_INSTANCES_TO_GRID = 0b001;
    }
}

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct BatchMetadata {
    flags: BatchFlags,
    pub tileset_size: UVec2,
    pub tile_size: UVec2,
    pub origin: Vec2,
    pub scale: f32,
    pub zorder: u32,
}

struct BatchMutableState {
    dirty_flag: DirtyFlags<BATCH_DIRTY_FLAG_COUNT>,
    size: u32,
    local: Box<[BatchInstance]>,
}

pub struct Batch {
    mutable: Mutex<BatchMutableState>,
    buffer: Buffer,
}

impl Batch {
    pub fn new(max_size: usize, device: &Device) -> Self {
        assert!(
            max_size <= MAX_BATCH_SIZE,
            "Batch too big, requested {max_size}, but can only fit {MAX_BATCH_SIZE}"
        );

        let buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Batch Buffer"),
            size: (max_size as u64) * (std::mem::size_of::<BatchInstance>() as u64),
            usage: BufferUsages::COPY_DST | BufferUsages::VERTEX,
            mapped_at_creation: false,
        });

        let dirty_flag = DirtyFlags::new();

        Self {
            buffer,
            mutable: Mutex::new(BatchMutableState {
                dirty_flag,
                size: 0,
                local: vec![BatchInstance::zeroed(); MAX_BATCH_SIZE].into_boxed_slice(),
            }),
        }
    }

    pub fn push_unchecked(&self, instance: BatchInstance) {
        let mut mutable = self.mutable.lock().unwrap();

        assert_ne!(
            mutable.size as usize, MAX_BATCH_SIZE,
            "The buffer is filled up"
        );

        let idx = mutable.size;
        mutable
            .dirty_flag
            .mark((idx / INSTANCES_PER_REGION) as usize);
        mutable.size += 1;
        mutable.local[idx as usize] = instance;
    }

    pub fn flush(&self, queue: &Queue) {
        let mut mutable = self.mutable.lock().unwrap();

        for marked in mutable.dirty_flag.iter_marked() {
            let start = marked * INSTANCES_PER_REGION as usize;
            let end = (marked + 1) * INSTANCES_PER_REGION as usize;

            let byte_offset = start * size_of::<BatchInstance>();

            queue.write_buffer(
                &self.buffer,
                byte_offset as u64,
                bytemuck::cast_slice(&mutable.local[start..end]),
            );
        }

        mutable.dirty_flag.clear()
    }

    pub fn buffer_slice(&self) -> BufferSlice {
        let bytes_size = self.size() * size_of::<BatchInstance>() as u64;
        self.buffer.slice(..bytes_size)
    }

    pub fn size(&self) -> u64 {
        let mutable = self.mutable.lock().unwrap();
        mutable.size as u64
    }
}
