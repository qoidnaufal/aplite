use learn_wgpu::prelude::*;

fn root(cx: &mut Context) {
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

    Stack::new(cx, |cx| {
        Stack::new(cx, |cx| {
            Image::new(cx, "assets/image1.jpg");
            Image::new(cx, "assets/image2.jpg").style(|s| s.set_dragable(true));
            TestTriangleWidget::new(cx)
                .style(|style| {
                    style.set_hover_color(Rgba::BLUE);
                    style.set_fill_color(Rgba::BLACK);
                });
        })
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
            });
        Stack::new(cx, |cx| {
            Stack::new(cx, |cx| {
                Button::new(cx)
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
                    .on_click(inc);
                Button::new(cx)
                    .style(|style| {
                        style.set_hover_color(Rgba::WHITE);
                        style.set_click_color(Rgba::BLUE);
                        style.set_stroke_width(5.);
                        style.set_corners(|r| r.set_each(0.039));
                    })
                    .on_click(dec);
                Button::new(cx)
                    .style(|style| {
                        style.set_hover_color(Rgba::YELLOW);
                        style.set_stroke_width(5.);
                        style.set_corners(|corners| {
                            corners.set_top_left(0.);
                            corners.set_bot_left(0.03);
                            corners.set_bot_right(0.);
                            corners.set_top_right(0.03);
                        });
                    });
                Button::new(cx)
                    .style(|style| {
                        style.set_corners(|corners| corners.set_each(0.04));
                        style.set_fill_color(Rgba::new(69, 172, 23, 255));
                    });
                })
                .style(|style| {
                    style.set_dragable(true);
                    style.set_fill_color(Rgba::new(111, 72, 234, 255));
                    style.set_min_width(1000);
                    style.set_padding(|padding| padding.set_all(20));
                    style.set_spacing(40);
                });
            TestCircleWidget::new(cx)
                .style(|style| {
                    style.set_hover_color(Rgba::GREEN);
                    style.set_stroke_width(10.);
                    style.set_fill_color(Rgba::BLACK);
                    style.set_stroke_color(Rgba::RED);
                });
        })
            .style(|style| {
                style.set_dragable(true);
                style.set_fill_color(Rgba::new(69, 69, 69, 255));
                style.set_orientation(Orientation::Horizontal);
                style.set_padding(|padding| {
                    padding.set_all(30);
                });
                style.set_spacing(30);
            });
        TestCircleWidget::new(cx)
            .style(|style| {
                style.set_dragable(true);
                style.set_hover_color(Rgba::BLACK);
                style.set_fill_color(Rgba::new(169, 72, 43, 255));
                style.set_stroke_width(5.);
                style.set_stroke_color(Rgba::WHITE);
            });
        })
        .style(|style| {
            style.set_padding(|padding| padding.set_all(20));
            style.set_spacing(20);
        });
}

fn dummy(cx: &mut Context) {
    let (counter, set_counter) = signal(0i32);

    let click = move || {
        set_counter.set(|num| *num += 1);
        eprintln!("counter: {}", counter.get());
    };

    Button::new(cx)
        .style(|style| {
            style.set_size((500, 200));
            style.set_stroke_color(Rgba::WHITE);
            style.set_stroke_width(10.);
            style.set_corners(|r| r.set_each(0.15));
            style.set_dragable(true);
        })
        .on_click(click);
    TestCircleWidget::new(cx)
        .style(|style| {
            style.set_dragable(true);
        });
}

fn main() -> AppResult {
    let mut args = std::env::args();
    match args.nth(1) {
        Some(arg) if arg == "dummy" => {
            App::new(dummy)
                .set_window_attributes(|window| {
                    window.set_title("Dummy");
                    window.set_inner_size((500, 500));
                })
                .set_background_color(Rgba::DARK_GRAY)
                .launch()
        },
        Some(arg) if arg == "empty" => {
            App::<fn(&mut Context)>::new_empty()
                .set_window_attributes(|window| {
                    window.set_title("Empty");
                })
                .set_background_color(Rgba::DARK_GREEN)
                .launch()
        }
        _ => {
            App::new(root).launch()
        }
    }
}
