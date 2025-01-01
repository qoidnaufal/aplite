mod error;
mod shapes;
mod renderer;
mod pipeline;
mod app;
mod types;
mod shader;
mod color;
mod buffer;
mod layout;
mod gpu;

use app::App;
use color::Rgb;
use layout::Triangle;
use types::Vector3;
use winit::event_loop::EventLoop;

use error::Error;

fn main() -> Result<(), Error> {
    env_logger::init();
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);

    let triangle = Triangle::new(Vector3 { x: 0, y: 0, z: 0 }, 1500, 1000, Rgb { r: 255, g: 0, b: 0 });
    let mut app = App::new(triangle);
    event_loop.run_app(&mut app)?;

    Ok(())
}
