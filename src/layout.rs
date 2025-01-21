use math::Size;
use winit::window::Window;

pub struct LayoutCtx<'a> {
    window: &'a Window,
    size: Size<u32>,
}
