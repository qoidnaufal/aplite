use aplite_types::{Matrix3x2, Size};

use super::buffer::Buffer;

pub(crate) struct Screen {
    pub(crate) transform: Buffer<Matrix3x2>,
    pub(crate) bind_group: wgpu::BindGroup,

    // FIXME: not needed
    pub(crate) screen_resolution: Size,
    pub(crate) scale_factor: f64,
}

impl Screen {
    pub(crate) fn new(
        device: &wgpu::Device,
        screen_resolution: Size,
        scale_factor: f64,
    ) -> Self {
        let usage = wgpu::BufferUsages::UNIFORM;
        let transform = Buffer::<Matrix3x2>::new(device, 1, usage);
        let bind_group = Self::bind_group(device, &[
            transform.bind_group_entry(0),
        ]);

        Self {
            transform,
            bind_group,
            screen_resolution,
            scale_factor,
        }
    }

    pub(crate) fn screen_size(&self) -> Size { self.screen_resolution }

    pub(crate) fn write(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        matrix: Matrix3x2,
    ) {
        self.transform.write(device, queue, 0, &[matrix]);
    }

    pub(crate) fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("screen bind group layout"),
            entries: &[
                Buffer::<Matrix3x2>::bind_group_layout_entry(wgpu::BufferBindingType::Uniform, 0),
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

// #[derive(Debug, Default, Clone, Copy)]
// pub enum ScreenResolution {
//     SD480p,
//     HD720p,
//     FullHD1K,
//     FullHD2K,
//     #[default]
//     UltraHD4K,
//     UltraHD8K,
// }

// impl ScreenResolution {
//     pub(crate) fn size_f32(&self) -> Size<f32> {
//         match self {
//             ScreenResolution::SD480p => Size::new(640., 480.),
//             ScreenResolution::HD720p => Size::new(1280., 720.),
//             ScreenResolution::FullHD1K => Size::new(1920., 1080.),
//             ScreenResolution::FullHD2K => Size::new(2560., 1440.),
//             ScreenResolution::UltraHD4K => Size::new(3840., 2160.),
//             ScreenResolution::UltraHD8K => Size::new(7680., 4320.),
//         }
//     }
// }
