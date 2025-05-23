use aplite_types::Matrix3x2;

use super::element::Element;
use super::buffer::Storage;

pub(crate) struct Gfx {
    pub(crate) elements: Storage<Element>,
    pub(crate) transforms: Storage<Matrix3x2>,
    pub(crate) bind_group: wgpu::BindGroup,
}

impl Gfx {
    pub(crate) fn new(device: &wgpu::Device) -> Self {
        let elements = Storage::<Element>::new(device, "element");
        let transforms = Storage::<Matrix3x2>::new(device, "transforms");
        let bind_group = Self::bind_group(device, &[
            elements.bind_group_entry(0),
            transforms.bind_group_entry(1),
        ]);

        Self { elements, transforms, bind_group }
    }

    pub(crate) fn write(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) -> bool {
        let mut realloc = false;
        realloc |= self.elements.write(device, queue);
        realloc |= self.transforms.write(device, queue);

        if realloc {
            self.bind_group = Self::bind_group(device, &[
                self.elements.bind_group_entry(0),
                self.transforms.bind_group_entry(1),
            ]);
        }

        realloc
    }

    pub(crate) fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("gfx bind group layout"),
            entries: &[
                Storage::<Element>::bind_group_layout_entry(0),
                Storage::<Matrix3x2>::bind_group_layout_entry(1),
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

    pub(crate) fn count(&self) -> usize { self.elements.len() }
}
