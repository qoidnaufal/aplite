use aplite_types::Matrix3x2;

use super::element::Element;
use super::buffer::Buffer;

pub(crate) struct Storage {
    pub(crate) elements: Buffer<Element>,
    pub(crate) transforms: Buffer<Matrix3x2>,
    pub(crate) bind_group: wgpu::BindGroup,
    pub(crate) element_data: Vec<Element>,
    pub(crate) transform_data: Vec<Matrix3x2>,
}

impl Storage {
    pub(crate) fn new(device: &wgpu::Device) -> Self {
        let elements = Buffer::<Element>::new(device, 1024, wgpu::BufferUsages::STORAGE, "element");
        let transforms = Buffer::<Matrix3x2>::new(device, 1024, wgpu::BufferUsages::STORAGE, "transforms");
        let bind_group = Self::bind_group(device, &[
            elements.bind_group_entry(0),
            transforms.bind_group_entry(1),
        ]);
        let element_data = vec![];
        let transform_data = vec![];

        Self { elements, transforms, bind_group, element_data, transform_data }
    }

    pub(crate) fn write(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) -> bool {
        let mut realloc = false;
        realloc |= self.elements.write(device, queue, 0, &self.element_data);
        realloc |= self.transforms.write(device, queue, 0, &self.transform_data);

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
                Buffer::<Element>::bind_group_layout_entry(wgpu::BufferBindingType::Storage { read_only: true }, 0),
                Buffer::<Matrix3x2>::bind_group_layout_entry(wgpu::BufferBindingType::Storage { read_only: true }, 1),
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

    pub(crate) fn update_element<F: Fn(&mut Element)>(&mut self, index: usize, f: F) {
        f(&mut self.element_data[index])
    }

    pub(crate) fn update_transform<F: Fn(&mut Matrix3x2)>(&mut self, index: usize, f: F) {
        f(&mut self.transform_data[index])
    }

    pub(crate) fn count(&self) -> usize { self.element_data.len() }
}
