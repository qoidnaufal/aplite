use shared::{Matrix4x4, Size};

use crate::app::DEFAULT_SCREEN_SIZE;

use super::buffer::Uniform;

pub(crate) struct Screen {
    transform: Uniform<Matrix4x4>,
    scaler: Uniform<Size<f32>>,
    pub(crate) bind_group: wgpu::BindGroup,
    initial_size: Size<f32>,
    initialized: bool,
}

impl Screen {
    pub(crate) fn new(device: &wgpu::Device, initial_size: Size<f32>) -> Self {
        let transform = Uniform::new(device, Matrix4x4::IDENTITY, "screen transform");
        let scaler = Uniform::new(device, DEFAULT_SCREEN_SIZE.into(), "screen scaler");
        let bind_group = Self::bind_group(device, &[
            transform.bind_group_entry(0),
            scaler.bind_group_entry(1)
        ]);
        Self {
            scaler,
            transform,
            bind_group,
            initial_size,
            initialized: false,
        }
    }

    pub(crate) fn write(&mut self, queue: &wgpu::Queue) {
        self.transform.write(queue);
        if !self.initialized {
            self.scaler.write(queue);
        }
        self.initialized = true;
    }

    pub(crate) fn initial_size(&self) -> Size<f32> {
        self.initial_size
    }

    pub(crate) fn update_transform<F: Fn(&mut Matrix4x4)>(&mut self, f: F) {
        self.transform.update(f);
    }

    pub(crate) fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("screen bind group layout"),
            entries: &[
                Uniform::<Matrix4x4>::bind_group_layout_entry(0),
                Uniform::<Size<f32>>::bind_group_layout_entry(1),
            ],
        })
    }

    pub(crate) fn bind_group(
        device: &wgpu::Device,
        entries: &[wgpu::BindGroupEntry]
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("screen bind group"),
            layout: &Self::bind_group_layout(device),
            entries,
        })
    }
}

