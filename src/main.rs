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
use layout::Button;
use winit::event_loop::EventLoop;

use error::Error;

fn main() -> Result<(), Error> {
    env_logger::init();

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);

    let mut app = App::new();
    app
        .add_widget(Button::new())
        .add_widget(Button::new());
    event_loop.run_app(&mut app)?;
    Ok(())
}
