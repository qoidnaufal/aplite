use shared::{Matrix4x4, Size};

use super::buffer::Uniform;

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

pub(crate) struct Screen {
    pub(crate) transform: Uniform<Matrix4x4>,
    pub(crate) resolution: Uniform<Size<f32>>,
    pub(crate) bind_group: wgpu::BindGroup,
    pub(crate) scale_factor: f64,
    res_changed: bool,
    is_resized: bool,
}

impl Screen {
    pub(crate) fn new(device: &wgpu::Device, initial_size: Size<f32>, scale_factor: f64) -> Self {
        // let sr = ScreenResolution::default().size_f32();
        // let sc = sr / (initial_size * scale_factor as f32);
        // let mat = Matrix4x4::IDENTITY
        //     .with_scale(sc.width(), sc.height())
        //     .with_translate(sc.width() - 1., 1. - sc.height());

        let transform = Uniform::new(device, Matrix4x4::IDENTITY, "screen transform");
        let resolution = Uniform::new(device, initial_size, "screen scaler");
        let bind_group = Self::bind_group(device, &[
            transform.bind_group_entry(0),
            resolution.bind_group_entry(1)
        ]);

        Self {
            resolution,
            transform,
            bind_group,
            scale_factor,
            res_changed: true,
            is_resized: true,
        }
    }

    pub(crate) fn resolution(&self) -> Size<f32> { self.resolution.data() }

    pub(crate) fn write(&mut self, queue: &wgpu::Queue) {
        if self.res_changed {
            self.resolution.write(queue);
            self.res_changed = false;
        }
        if self.is_resized {
            self.transform.write(queue);
            self.is_resized = false;
        }
    }

    pub(crate) fn update_transform<F: Fn(&mut Matrix4x4)>(&mut self, f: F) {
        self.transform.update(f);
        self.is_resized = true;
    }

    // pub(crate) fn update_screen_resolution<F: Fn(&mut Size<f32>)>(&mut self, f: F) {
    //     self.resolution.update(f);
    //     self.res_changed = true;
    // }

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

