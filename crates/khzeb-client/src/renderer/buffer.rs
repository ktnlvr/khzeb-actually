use std::{marker::PhantomData, ops::Deref, sync::Arc};

use bytemuck::{Pod, Zeroable};
use wgpu::{util::DeviceExt, Buffer, BufferUsages, Device};

pub struct BufferHandle<T: Zeroable + Pod> {
    pub buffer: Arc<Buffer>,
    _phantom_data: PhantomData<T>,
}

impl<T: Zeroable + Pod> Deref for BufferHandle<T> {
    type Target = Buffer;

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

pub fn create_buffer<T: Zeroable + Pod>(device: &Device, usage: BufferUsages) -> BufferHandle<T> {
    let empty = vec![0; std::mem::size_of::<T>()];
    let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: &empty[..],
        usage,
    });

    BufferHandle {
        buffer: Arc::new(buffer),
        _phantom_data: Default::default(),
    }
}
