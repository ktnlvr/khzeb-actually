use bytemuck::{Pod, Zeroable};
use glam::{IVec2, Vec2};
use wgpu::{vertex_attr_array, BufferAddress, VertexAttribute, VertexBufferLayout, VertexStepMode};

use super::color::Rgba;

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
