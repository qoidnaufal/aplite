use learn_wgpu::prelude::*;

fn root() -> impl IntoView {
    let (counter, set_counter) = signal(0i32);
    eprintln!("{}", counter.get());

    let c1 = counter.clone();
    let s1 = set_counter.clone();
    let inc = move || {
        s1.set(|num| *num += 1);
        eprintln!("inc {}", c1.get());
    };

    let dec = move || {
        set_counter.set(|num| *num -= 1);
        eprintln!("dec {}", counter.get());
    };

    let first_row = [
        Image::new("assets/image1.jpg", |_| {}).into_view(),
        Image::new("assets/image2.jpg", |_| {}).style(|s| s.set_dragable(true)).into_view(),
        TestTriangleWidget::new()
            .style(|style| {
                style.set_hover_color(Rgba::BLUE);
                style.set_fill_color(Rgba::BLACK);
            })
            .into_view(),
    ];

    let second_row = [
        Stack::new([
            Button::new()
                .style(|style| {
                    style.set_hover_color(Rgba::BLUE);
                    style.set_click_color(Rgba::GREEN);
                    style.set_stroke_width(10.);
                    style.set_corners(|corners| {
                        corners.set_top_left(0.025);
                        corners.set_bot_left(0.025);
                        corners.set_bot_right(0.0);
                        corners.set_top_right(0.0);
                    });
                })
                .on_click(inc)
                .into_view(),
            Button::new()
                .style(|style| {
                    style.set_hover_color(Rgba::WHITE);
                    style.set_click_color(Rgba::BLUE);
                    style.set_stroke_width(5.);
                    style.set_corners(|r| r.set_each(0.039));
                })
                .on_click(dec)
                .into_view(),
            Button::new()
                .style(|style| {
                    style.set_hover_color(Rgba::YELLOW);
                    style.set_stroke_width(5.);
                    style.set_corners(|corners| {
                        corners.set_top_left(0.);
                        corners.set_bot_left(0.03);
                        corners.set_bot_right(0.);
                        corners.set_top_right(0.03);
                    });
                })
                .into_view(),
            Button::new()
                .style(|style| {
                    style.set_corners(|corners| corners.set_each(0.04));
                    style.set_fill_color(Rgba::new(69, 172, 23, 255));
                })
                .into_view(),
            ])
            .style(|style| {
                style.set_dragable(true);
                style.set_fill_color(Rgba::new(111, 72, 234, 255));
                style.set_min_width(1000);
                style.set_padding(|padding| padding.set_all(20));
                style.set_spacing(40);
            })
            .into_view(),
        TestCircleWidget::new()
            .style(|style| {
                style.set_hover_color(Rgba::GREEN);
                style.set_stroke_width(10.);
                style.set_fill_color(Rgba::BLACK);
                style.set_stroke_color(Rgba::RED);
            })
            .into_view(),
    ];

    Stack::new([
        Stack::new(first_row)
            .style(|style| {
                style.set_shape(Shape::RoundedRect);
                style.set_corners(|corner| {
                    corner.set_each(0.03);
                });
                style.set_stroke_color(Rgba::GREEN);
                style.set_stroke_width(5.);
                style.set_dragable(true);
                style.set_orientation(Orientation::Horizontal);
                style.set_fill_color(Rgba::YELLOW);
                style.set_padding(|padding| {
                    padding.set_left(40);
                    padding.set_right(40);
                    padding.set_top(20);
                    padding.set_bottom(20);
                });
                style.set_spacing(40);
            })
            .into_view(),
        Stack::new(second_row)
            .style(|style| {
                style.set_dragable(true);
                style.set_fill_color(Rgba::new(69, 69, 69, 255));
                style.set_orientation(Orientation::Horizontal);
                style.set_padding(|padding| {
                    padding.set_all(30);
                });
                style.set_spacing(30);
            })
            .into_view(),
        TestCircleWidget::new()
            .style(|style| {
                style.set_dragable(true);
                style.set_hover_color(Rgba::BLACK);
                style.set_fill_color(Rgba::new(169, 72, 43, 255));
                style.set_stroke_width(5.);
                style.set_stroke_color(Rgba::WHITE);
            })
            .into_view(),
        ])
        .style(|style| {
            style.set_padding(|padding| padding.set_all(20));
            style.set_spacing(20);
        })
}

fn dummy() -> impl IntoView {
    let (counter, set_counter) = signal(0i32);

    let click = move || {
        set_counter.set(|num| *num += 1);
        eprintln!("counter: {}", counter.get());
    };

    Stack::new([
        Button::new()
            .style(|style| {
                style.set_size((500, 200));
                style.set_stroke_color(Rgba::WHITE);
                style.set_stroke_width(10.);
                style.set_corners(|r| r.set_each(0.15));
            })
            .on_click(click)
            .into_view(),
        TestCircleWidget::new()
            .style(|style| {
                style.set_dragable(true);
            })
            .into_view(),
        ])
        .style(|style| {
            style.set_padding(|p| p.set_all(30));
        })
}

fn main() -> AppResult {
    let mut args = std::env::args();
    match args.nth(1) {
        Some(arg) if arg == "dummy" => {
            App::new(dummy)
                .set_window_properties(|window| {
                    window.set_title("Dummy");
                })
                .launch()
        },
        Some(arg) if arg == "empty" => {
            App::<fn() -> View>::new_empty()
                .set_window_properties(|window| window.set_title("Empty"))
                .launch()
        }
        _ => {
            App::new(root).launch()
        }
    }
}
