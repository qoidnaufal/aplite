mod app;
mod callback;
mod color;
mod error;
mod context;
mod renderer;
mod element;
mod signal;
mod tree;
mod view;

use app::launch;
use color::*;
use element::Element;
use signal::Signal;
use view::*;

use error::Error;

fn root() -> impl IntoView {
    let counter = Signal::new(0i32);
    eprintln!("init {}", counter.get());

    let c1 = counter.clone();
    let inc = move |el: &mut Element| {
        c1.set(|num| *num += 1);
        eprintln!("inc1 {}", c1.get());
        el.set_fill_color(|color| color.r += 150);
    };

    let c2 = counter.clone();
    let shift_left = move |el: &mut Element| {
        c2.set(|num| *num <<= 1);
        eprintln!("shift left {}", c2.get());
        el.set_fill_color(|color| color.r += 150);
    };

    let c3 = counter.clone();
    let dec = move |el: &mut Element| {
        c3.set(|num| *num -= 1);
        eprintln!("dec {}", c3.get());
        el.set_fill_color(|color| color.r += 150);
    };

    let c4 = counter.clone();
    let shift_right = move |el: &mut Element| {
        c4.set(|num| *num >>= 1);
        eprintln!("shift right {}", c4.get());
        el.set_fill_color(|color| color.r += 150);
    };

    let hover = move |el: &mut Element| { el.set_fill_color(|color| *color = Rgba::BLUE) };
    let drag = move |el: &mut Element| {
        el.set_fill_color(|color| *color = Rgba::GREEN);
    };

    vstack([
        hstack([
            image("assets/image1.jpg").into_any(),
            image("assets/image2.jpg").on_drag(drag).into_any(),
            TestTriangleWidget::new()
                .style(|style| style.set_fill_color(Rgba::BLACK))
                .on_hover(hover)
                .into_any(),
        ])
        .on_drag(drag)
        .into_any(),
        hstack([
            vstack([
                button()
                    .style(|style| {
                        style.set_stroke_width(10.);
                        style.set_corners(|r| {
                            r.set_top_left(0.025);
                            r.set_bot_left(0.025);
                            r.set_bot_right(0.0);
                            r.set_top_right(0.0);
                        });
                    })
                    .on_click(shift_right)
                    .on_hover(hover)
                    .into_any(),
                button()
                    .style(|style| {
                        style.set_stroke_width(5.);
                        style.set_corners(|r| r.set_each(0.039));
                    })
                    .on_click(shift_left)
                    .on_hover(hover)
                    .into_any(),
                button()
                    .style(|style| {
                        style.set_stroke_width(5.);
                        style.set_corners(|r| {
                            r.set_top_left(0.);
                            r.set_bot_left(0.03);
                            r.set_bot_right(0.);
                            r.set_top_right(0.03);
                        });
                    })
                    .on_click(dec)
                    .on_hover(hover)
                    .into_any(),
                button()
                    .style(|style| {
                        style.set_corners(|r| r.set_each(0.04));
                        style.set_fill_color(Rgba::new(69, 172, 23, 255));
                    })
                    .on_click(inc)
                    .on_hover(hover)
                    .into_any(),
            ])
            .style(|style| style.set_fill_color(Rgba::new(111, 72, 234, 255)))
            .on_drag(drag)
            .into_any(),
            TestCircleWidget::new()
                .style(|style| {
                    style.set_stroke_width(10.);
                    style.set_stroke_color(Rgba::GREEN);
                })
                .on_hover(hover)
                .into_any(),
        ])
        .style(|style| style.set_fill_color(Rgba::new(69, 69, 69, 255)))
        .on_drag(drag)
        .into_any(),
        TestCircleWidget::new()
            .style(|style| {
                style.set_fill_color(Rgba::new(169, 72, 43, 255));
                style.set_stroke_width(5.);
                style.set_stroke_color(Rgba::WHITE);
            })
            .on_drag(drag)
            .on_hover(hover)
            .into_any(),
    ])
}

fn dummy() -> impl IntoView {
    let hover = move |el: &mut Element| {
        el.set_fill_color(|color| *color = Rgba::BLUE);
    };
    let drag = move |el: &mut Element| {
        el.set_fill_color(|color| *color = Rgba::GREEN);
    };

    button()
        .style(|style| {
            style.set_dimensions((500, 200));
            style.set_stroke_color(Rgba::WHITE);
            style.set_stroke_width(10.);
            style.set_corners(|r| r.set_each(0.15));
        })
        .on_hover(hover)
        .on_drag(drag)
}

fn main() -> Result<(), Error> {
    let mut args = std::env::args();
    match args.nth(1) {
        Some(arg) if arg == "dummy" => launch(dummy),
        _ => launch(root)
    }
}
