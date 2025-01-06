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
mod widget;

use app::App;
use widget::*;
use winit::event_loop::EventLoop;

use error::Error;

fn main() -> Result<(), Error> {
    env_logger::init();

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);

    let mut count = 0i32;
    println!("init: {count}");
    let inc = move || {
        count += 1;
        eprintln!("inc {count}");
    };
    let dec = move || {
        count -= 1;
        eprintln!("dec {count}");
    };

    let mut app = App::new();
    app
        .add_widget(Button::new().on_click(inc))
        .add_widget(TestWidget::new().on_click(dec))
        .add_widget(Image::new().on_click(inc))
        .add_widget(TestWidget::new().on_click(dec));
    event_loop.run_app(&mut app)?;
    Ok(())
}
