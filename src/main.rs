mod app;
mod callback;
mod color;
mod error;
mod context;
mod renderer;
mod shapes;
mod signal;
mod texture;
mod storage;
mod view;

use app::App;
use color::*;
use shapes::Shape;
use signal::Signal;
use view::*;
use winit::event_loop::EventLoop;

use error::Error;

fn root() -> impl IntoView {
    let counter = Signal::new(0i32);
    eprintln!("init {}", counter.get());

    let c1 = counter.clone();
    let inc = move |shape: &mut Shape| {
        c1.set(|num| *num += 1);
        eprintln!("inc1 {}", c1.get());
        shape.set_color(|color| *color = Rgb::WHITE);
    };

    let c2 = counter.clone();
    let shift_left = move |_: &mut Shape| {
        c2.set(|num| *num <<= 1);
        eprintln!("shift left {}", c2.get());
    };

    let c3 = counter.clone();
    let dec = move |shape: &mut Shape| {
        c3.set(|num| *num -= 1);
        eprintln!("dec {}", c3.get());
        shape.set_color(|color| color.r += 150);
    };

    let c4 = counter.clone();
    let shift_right = move |_: &mut Shape| {
        c4.set(|num| *num >>= 1);
        eprintln!("shift right {}", c4.get());
    };

    let hover = move |shape: &mut Shape| { shape.set_color(|color| *color = Rgb::BLUE) };
    let drag = move |shape: &mut Shape| {
        shape.set_color(|color| *color = Rgb::GREEN);
    };

    vstack(
        [
            hstack(
                [
                    image("assets/image1.jpg").on_click(shift_right).into_any(),
                    image("assets/image2.jpg").on_click(shift_left).into_any(),
                    TestTriangleWidget::new().on_hover(hover).into_any(),
                ]
            ).on_hover(hover).into_any(),
            vstack(
                [
                    button().on_click(inc.clone()).on_hover(hover).into_any(),
                    button().on_click(inc.clone()).on_hover(hover).into_any(),
                    button().on_click(inc.clone()).on_hover(hover).into_any(),
                    button().on_click(inc).on_hover(hover).into_any(),
                ]
            ).on_hover(hover).into_any(),
            TestTriangleWidget::new()
                .on_click(dec)
                .on_drag(drag)
                .on_hover(hover)
                .into_any(),
        ]
    ).on_hover(hover)
}

fn main() -> Result<(), Error> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);

    let mut app = App::new();
    app.add_widget(root());

    event_loop.run_app(&mut app)?;
    Ok(())
}
