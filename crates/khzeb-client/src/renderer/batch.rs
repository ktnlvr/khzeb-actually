use std::sync::Mutex;

use bitflags::bitflags;
use bytemuck::{NoUninit, Pod, Zeroable};
use glam::{IVec2, UVec2, Vec2};
use wgpu::{naga::FastHashMap, Buffer, BufferDescriptor, BufferSlice, BufferUsages, Device, Queue};

pub const BATCH_DIRTY_FLAG_SIZE: usize = 4;
pub type BatchDirtyFlagBlock = u64;
pub const BATCH_DIRTY_FLAGS_BITS_PER_BLOCK: usize = 64;
pub type BatchDirtyFlag = [BatchDirtyFlagBlock; BATCH_DIRTY_FLAG_SIZE];

pub const INSTANCES_PER_REGION: u32 = 16;
pub const MAX_BATCH_SIZE: usize =
    BATCH_DIRTY_FLAG_SIZE * BATCH_DIRTY_FLAGS_BITS_PER_BLOCK * INSTANCES_PER_REGION as usize;

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

#[derive(Default)]
struct BatchMutableState {
    dirty_flag: BatchDirtyFlag,
    size: u32,
    writes: FastHashMap<u32, BatchInstance>,
}

pub struct Batch {
    mutable: Mutex<BatchMutableState>,
    buffer: Buffer,
}

impl Batch {
    pub fn new(max_size: usize, device: &Device) -> Self {
        if max_size > MAX_BATCH_SIZE {
            panic!("Batch too big, requested {max_size}, but can only fit {MAX_BATCH_SIZE}")
        }

        let buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Batch Buffer"),
            size: (max_size as u64) * (std::mem::size_of::<BatchInstance>() as u64),
            usage: BufferUsages::COPY_DST | BufferUsages::VERTEX,
            mapped_at_creation: false,
        });

        let mutable = Mutex::new(BatchMutableState::default());

        Self { mutable, buffer }
    }

    fn mark_dirty(mutable: &mut BatchMutableState, idx: u32) {
        assert!(
            idx < MAX_BATCH_SIZE as u32,
            "Index has to be within the batch"
        );

        let region_idx = idx / INSTANCES_PER_REGION;
        let slice_idx = region_idx as usize / BATCH_DIRTY_FLAGS_BITS_PER_BLOCK;
        let bit_idx = region_idx as usize % BATCH_DIRTY_FLAGS_BITS_PER_BLOCK;

        mutable.dirty_flag[slice_idx] |= 1 << bit_idx;
    }

    pub fn push_unchecked(&self, instance: BatchInstance) {
        let mut mutable = self.mutable.lock().unwrap();

        if mutable.size as usize == MAX_BATCH_SIZE {
            panic!("Bang! Batch overfilled")
        }

        let idx = mutable.size;
        Batch::mark_dirty(&mut mutable, idx);
        mutable.size += 1;
        mutable.writes.insert(idx, instance);
    }

    pub fn flush(&self, queue: &Queue) {
        // TODO: actually use the dirty flag
        let mut mutable = self.mutable.lock().unwrap();
        for (i, write) in mutable.writes.drain() {
            let written_data = [write];
            queue.write_buffer(
                &self.buffer,
                (i as usize * std::mem::size_of::<BatchInstance>()) as u64,
                bytemuck::cast_slice(&written_data),
            );
        }
    }

    pub fn buffer_slice(&self) -> BufferSlice {
        self.buffer.slice(..)
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
}

impl BatchInstance {
    pub fn builder() -> Self {
        BatchInstance {
            position: InstancePosition { int: IVec2::ZERO },
            scale: 1.,
        }
    }

    pub fn with_scale(self, scale: f32) -> Self {
        Self { scale, ..self }
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
