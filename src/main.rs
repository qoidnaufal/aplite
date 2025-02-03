mod app;
mod callback;
mod color;
mod error;
mod context;
mod renderer;
mod shapes;
mod signal;
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
        shape.set_color(|color| color.r += 150);
    };

    let c2 = counter.clone();
    let shift_left = move |shape: &mut Shape| {
        c2.set(|num| *num <<= 1);
        eprintln!("shift left {}", c2.get());
        shape.set_color(|color| color.r += 150);
    };

    let c3 = counter.clone();
    let dec = move |shape: &mut Shape| {
        c3.set(|num| *num -= 1);
        eprintln!("dec {}", c3.get());
        shape.set_color(|color| color.r += 150);
    };

    let c4 = counter.clone();
    let shift_right = move |shape: &mut Shape| {
        c4.set(|num| *num >>= 1);
        eprintln!("shift right {}", c4.get());
        shape.set_color(|color| color.r += 150);
    };

    let hover = move |shape: &mut Shape| { shape.set_color(|color| *color = Rgb::BLUE) };
    let drag = move |shape: &mut Shape| {
        shape.set_color(|color| *color = Rgb::GREEN);
    };

    vstack(
        [
            hstack(
                [
                    image("assets/image1.jpg").into_any(),
                    image("assets/image2.jpg").into_any(),
                    TestTriangleWidget::new().on_hover(hover).into_any(),
                ]
            ).into_any(),
            hstack(
                [
                    button().on_click(shift_right).on_hover(hover).into_any(),
                    button().on_click(shift_left).on_hover(hover).into_any(),
                    button().on_click(dec).on_hover(hover).into_any(),
                    button().on_click(inc).on_hover(hover).into_any(),
                ]
            ).into_any(),
            TestTriangleWidget::new()
                .on_drag(drag)
                .on_hover(hover)
                .into_any(),
        ]
    )
}

// fn dummy() -> impl IntoView {
//     let hover = move |shape: &mut Shape| { shape.set_color(|color| *color = Rgb::BLUE) };
//     let drag = move |shape: &mut Shape| {
//         shape.set_color(|color| *color = Rgb::GREEN);
//     };
//     vstack(
//         [
//             TestTriangleWidget::new().on_hover(hover).on_drag(drag).into_any(),
//             button().on_hover(hover).into_any(),
//             button().on_hover(hover).into_any(),
//         ]
//     )
// }

fn main() -> Result<(), Error> {
    let event_loop = EventLoop::new()?;
    let mut app = App::new();
    app.add_widget(root);

    event_loop.run_app(&mut app)?;
    Ok(())
}
