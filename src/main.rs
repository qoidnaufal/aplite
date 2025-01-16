mod error;
mod shapes;
mod renderer;
mod pipeline;
mod app;
mod callback;
mod shader;
mod signal;
mod color;
mod buffer;
mod layout;
mod gpu;
mod widget;
mod texture;

use app::App;
use color::Rgb;
use shapes::Shape;
use signal::Signal;
use widget::*;
use winit::event_loop::EventLoop;

use error::Error;

fn add_widget(app: &mut App) {
    let counter = Signal::new(0i32);
    eprintln!("init {}", counter.get());

    let c1 = counter.clone();
    let inc = move |shape: &mut Shape| {
        shape.set_color(|color| {
            *color.g() += 150;
            *color.b() -= 150;
        });
        c1.set(|num| *num += 1);
        eprintln!("inc1 {}", c1.get());
    };

    let c2 = counter.clone();
    let shift_left = move |shape: &mut Shape| {
        shape.set_color(|color| {
            *color.r() += 100;
            *color.g() += 100;
        });
        c2.set(|num| *num <<= 1);
        eprintln!("shift left {}", c2.get());
    };

    let c3 = counter.clone();
    let dec = move |shape: &mut Shape| {
        shape.set_color(|color| *color.r() += 150);
        c3.set(|num| *num -= 1);
        eprintln!("dec {}", c3.get());
    };

    let c4 = counter.clone();
    let right_shift = move |shape: &mut Shape| {
        shape.set_color(|color| *color = Rgb::BLACK.into());
        c4.set(|num| *num >>= 1);
        eprintln!("right shift {}", c4.get());
    };

    let hover = move |shape: &mut Shape| { shape.set_color(|color| *color = Rgb::BLUE.into()) };
    let drag = move |shape: &mut Shape| {
        shape.set_color(|color| *color = Rgb::GREEN.into());
        shape.set_position();
    };

    app
        .add_widget(Button::new().on_click(inc).on_drag(drag).on_hover(hover))
        .add_widget(TestWidget::new().on_click(dec).on_drag(drag).on_hover(hover))
        .add_widget(image("assets/image2.jpg").on_click(shift_left.clone()).on_drag(drag).on_hover(hover))
        .add_widget(TestCircleWidget::new().on_click(right_shift).on_drag(drag).on_hover(hover))
        .add_widget(Image::new("assets/image1.jpg").on_click(shift_left).on_drag(drag).on_hover(hover));
}

fn main() -> Result<(), Error> {
    env_logger::init();

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);

    let mut app = App::new();
    add_widget(&mut app);

    event_loop.run_app(&mut app)?;
    Ok(())
}
