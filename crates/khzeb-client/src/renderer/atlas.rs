use std::ops::Deref;

use bytemuck::{Pod, Zeroable};
use wgpu::{BindingType, Buffer, BufferBindingType, BufferUsages, Device, Queue};

use super::buffer::{create_buffer, BufferHandle};

#[derive(Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct TextureAtlasProperties {
    pub width: u16,
    pub height: u16,
    pub tile_width: u16,
    pub tile_height: u16,
}

pub type TextureAtlasBuffer = BufferHandle<TextureAtlasProperties>;

pub struct TextureAtlas {
    pub properties: TextureAtlasProperties,
    buffer: TextureAtlasBuffer,
}

impl TextureAtlas {
    pub fn new_square(device: &Device, queue: &Queue, side: u16, tile: u16) -> Self {
        let properties = TextureAtlasProperties {
            width: side,
            height: side,
            tile_width: tile,
            tile_height: tile,
        };

        Self::new(device, queue, properties)
    }

    pub fn new(device: &Device, queue: &Queue, properties: TextureAtlasProperties) -> Self {
        let buffer = create_buffer(device, BufferUsages::UNIFORM | BufferUsages::COPY_DST);
        let atlas = Self { properties, buffer };
        atlas.flush(queue);
        atlas
    }

    pub fn flush(&self, queue: &Queue) {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.properties]));
    }

    pub fn binding_type() -> BindingType {
        BindingType::Buffer {
            ty: BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        }
    }
}

impl Deref for TextureAtlas {
    type Target = Buffer;

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}
