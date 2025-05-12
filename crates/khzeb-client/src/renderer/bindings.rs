use std::ops::Deref;

use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, Device, ShaderStages,
};

pub struct BindingLayout {
    entries: Vec<BindGroupLayoutEntry>,
    pub layout: BindGroupLayout,
}

impl Deref for BindingLayout {
    type Target = BindGroupLayout;

    fn deref(&self) -> &Self::Target {
        &self.layout
    }
}

pub struct Binding {
    pub group: BindGroup,
}

impl Deref for Binding {
    type Target = BindGroup;

    fn deref(&self) -> &Self::Target {
        &self.group
    }
}

impl<'binding> From<&'binding Binding> for Option<&'binding BindGroup> {
    fn from(binding: &'binding Binding) -> Self {
        Some(&binding.group)
    }
}

pub fn create_binding_layout(
    device: &Device,
    visibility: ShaderStages,
    entries: impl IntoIterator<Item = BindingType>,
) -> BindingLayout {
    let entries = entries
        .into_iter()
        .enumerate()
        .map(|(i, ty)| BindGroupLayoutEntry {
            binding: (i as u32),
            visibility,
            ty,
            count: None,
        })
        .collect::<Vec<_>>();

    let layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: None,
        entries: &entries,
    });

    BindingLayout { entries, layout }
}

pub fn create_binding<'resource>(
    device: &Device,
    layout: &BindingLayout,
    resources: impl IntoIterator<Item = BindingResource<'resource>>,
) -> Binding {
    let entries = resources
        .into_iter()
        .enumerate()
        .map(|(i, resource)| BindGroupEntry {
            binding: i as u32,
            resource,
        })
        .collect::<Vec<_>>();

    let binds = device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout: &layout.layout,
        entries: &entries[..],
    });

    Binding { group: binds }
}
