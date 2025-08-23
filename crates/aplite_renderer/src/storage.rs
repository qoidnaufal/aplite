use aplite_types::Matrix3x2;

use super::element::Element;
use super::buffer::Buffer;

pub(crate) struct StorageBuffers {
    pub(crate) elements: Buffer<Element>,
    pub(crate) transforms: Buffer<[f32; 6]>,
    pub(crate) bind_group: wgpu::BindGroup,
}

impl StorageBuffers {
    pub(crate) fn new(device: &wgpu::Device) -> Self {
        let storage = wgpu::BufferUsages::STORAGE;
        let elements = Buffer::<Element>::new(device, 1024, storage);
        let transforms = Buffer::<[f32; 6]>::new(device, 1024, storage);

        let bind_group = Self::bind_group(device, &[
            elements.bind_group_entry(0),
            transforms.bind_group_entry(1),
        ]);

        Self {
            elements,
            transforms,
            bind_group,
        }
    }

    pub(crate) fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        let binding_type = wgpu::BufferBindingType::Storage { read_only: true };
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("gfx bind group layout"),
            entries: &[
                Buffer::<Element>::bind_group_layout_entry(binding_type, 0),
                Buffer::<Matrix3x2>::bind_group_layout_entry(binding_type, 1),
            ],
        })
    }

    pub(crate) fn bind_group(
        device: &wgpu::Device,
        entries: &[wgpu::BindGroupEntry<'_>],
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("gfx bind group"),
            layout: &Self::bind_group_layout(device),
            entries,
        })
    }
}
