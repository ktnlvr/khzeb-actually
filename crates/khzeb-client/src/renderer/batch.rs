use std::sync::Mutex;

use bitflags::bitflags;
use bytemuck::{Pod, Zeroable};
use glam::{IVec2, UVec2, Vec2};
use wgpu::{Buffer, BufferDescriptor, BufferSlice, BufferUsages, Device, Queue};

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

struct BatchMutableState {
    dirty_flag: BatchDirtyFlag,
    size: u32,
    local: Box<[BatchInstance]>,
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

        Self {
            buffer,
            mutable: Mutex::new(BatchMutableState {
                dirty_flag: [0; BATCH_DIRTY_FLAG_SIZE],
                size: 0,
                local: vec![BatchInstance::zeroed(); MAX_BATCH_SIZE].into_boxed_slice(),
            }),
        }
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
        mutable.local[idx as usize] = instance;
    }

    pub fn flush(&self, queue: &Queue) {
        let mut mutable = self.mutable.lock().unwrap();

        let mut slices = Vec::with_capacity(16);

        for (block_idx, block) in mutable.dirty_flag.iter_mut().enumerate() {
            let mut bits = *block;
            if bits == 0 {
                continue;
            }

            while bits != 0 {
                let first = bits.trailing_zeros() as usize;
                let run = (bits >> first).trailing_ones() as usize;

                let region_start = block_idx * BATCH_DIRTY_FLAGS_BITS_PER_BLOCK + first;

                let start_instance_idx = region_start as u32 * INSTANCES_PER_REGION;
                let end_instance_idx = (region_start + run) as u32 * INSTANCES_PER_REGION;

                if start_instance_idx < end_instance_idx {
                    let byte_offset = start_instance_idx as usize * size_of::<BatchInstance>();

                    let s = start_instance_idx as usize;
                    let e = end_instance_idx as usize;
                    slices.push((s, e, byte_offset as u64));
                }

                // Black magic
                bits &= !(((1_u64 << run) - 1) << first);
            }

            *block = 0;
        }

        for (s, e, byte_offset) in slices {
            let slice = &mutable.local[s..e];
            queue.write_buffer(&self.buffer, byte_offset, bytemuck::cast_slice(slice));
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
