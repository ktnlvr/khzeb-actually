use std::ops::Deref;

use image::DynamicImage;
use wgpu::{
    Device, Origin3d, Queue, TexelCopyBufferLayout, TexelCopyTextureInfo, Texture as WGPUTexture,
    TextureAspect, TextureUsages, TextureView as WGPUTextureView, TextureViewDescriptor,
};

pub struct Texture {
    texture: WGPUTexture,
}

pub struct TextureView {
    view: WGPUTextureView,
}

impl Deref for TextureView {
    type Target = WGPUTextureView;

    fn deref(&self) -> &Self::Target {
        &self.view
    }
}

impl Texture {
    pub fn new(
        device: &Device,
        queue: &Queue,
        image: impl Into<DynamicImage>,
        usage: TextureUsages,
    ) -> Texture {
        let diffuse = image.into().to_rgba8();
        let dimensions = diffuse.dimensions();
        let texture_extent = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: usage | TextureUsages::COPY_DST,
            label: None,
            view_formats: &[],
        });

        queue.write_texture(
            TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            &diffuse,
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            texture_extent,
        );

        Texture { texture }
    }

    pub fn to_view(&self) -> TextureView {
        let view = self.texture.create_view(&TextureViewDescriptor::default());
        TextureView { view }
    }
}
