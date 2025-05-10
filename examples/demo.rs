use aplite::prelude::*;

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

    HStack::new(cx, |cx| {
        Image::new(cx, "assets/image1.jpg");
        Image::new(cx, "assets/image2.jpg").style(|s| s.set_dragable(true));
        TestCircleWidget::new(cx).style(|style| {
            style.set_hover_color(Rgba::BLUE);
            style.set_fill_color(Rgba::PURPLE);
        });
    }).style(|style| {
        style.set_shape(Shape::RoundedRect);
        style.set_corners(|corner| corner.set_each(30));
        style.set_fill_color(Rgba::DARK_GRAY);
        style.set_stroke_color(Rgba::LIGHT_GRAY);
        style.set_stroke_width(5);
        style.set_dragable(true);
        style.set_spacing(40);
        style.set_padding(|padding| {
            padding.set_left(40);
            padding.set_right(40);
            padding.set_top(20);
            padding.set_bottom(20);
        });
    });

    HStack::new(cx, |cx| {
        VStack::new(cx, |cx| {
            Button::new(cx)
                .style(|style| {
                    style.set_hover_color(Rgba::BLUE);
                    style.set_click_color(Rgba::GREEN);
                    style.set_stroke_width(10);
                    style.set_corners(|corners| {
                        corners.set_top_left(40);
                        corners.set_bot_left(40);
                        corners.set_bot_right(0);
                        corners.set_top_right(0);
                    });
                })
                .on_click(inc);
            Button::new(cx)
                .style(|style| {
                    style.set_fill_color(Rgba::GREEN);
                    style.set_hover_color(Rgba::WHITE);
                    style.set_click_color(Rgba::RED);
                    style.set_stroke_width(5);
                    style.set_corners(|r| r.set_each(25));
                })
                .on_click(dec);
            Button::new(cx)
                .style(|style| {
                    style.set_fill_color(Rgba::BLUE);
                    style.set_hover_color(Rgba::YELLOW);
                    style.set_stroke_width(5);
                    style.set_corners(|corners| {
                        corners.set_top_left(0);
                        corners.set_bot_left(30);
                        corners.set_bot_right(0);
                        corners.set_top_right(30);
                    });
                });
            Button::new(cx)
                .style(|style| {
                    style.set_corners(|corners| corners.set_each(50));
                    style.set_fill_color(Rgba::YELLOW);
                });
        }).style(|style| {
            style.set_dragable(true);
            style.set_fill_color(Rgba::DARK_GREEN);
            style.set_min_width(1000);
            style.set_alignment(|align| {
                align.set_h(HAlign::Left);
                align.set_v(VAlign::Bottom);
            });
            style.set_padding(|padding| padding.set_all(20));
            style.set_spacing(10);
        });

        TestCircleWidget::new(cx)
            .style(|style| {
                style.set_hover_color(Rgba::GREEN);
                style.set_stroke_width(10);
                style.set_fill_color(Rgba::BLACK);
                style.set_stroke_color(Rgba::RED);
            });

    }).style(|style| {
        style.set_dragable(true);
        style.set_fill_color(Rgba::LIGHT_GRAY);
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
            style.set_stroke_width(5);
            style.set_stroke_color(Rgba::WHITE);
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
            style.set_stroke_width(10);
            style.set_corners(|r| r.set_each(40));
            style.set_dragable(true);
        })
        .on_click(click);
    TestCircleWidget::new(cx)
        .style(|style| {
            style.set_stroke_width(6);
            style.set_size((150, 150));
            style.set_dragable(true);
        });
}

fn main() -> ApliteResult {
    let mut args = std::env::args();
    match args.nth(1) {
        Some(arg) if arg == "dummy" => {
            Aplite::new(dummy)
                .set_window_attributes(|window| {
                    window.set_title("Dummy");
                    window.set_inner_size((500, 500));
                })
                .set_background_color(Rgba::DARK_GRAY)
                .launch()
        },
        Some(arg) if arg == "empty" => {
            Aplite::<fn(&mut Context)>::new_empty()
                .set_window_attributes(|window| {
                    window.set_title("Empty");
                })
                .set_background_color(Rgba::DARK_GREEN)
                .launch()
        }
        _ => {
            Aplite::new(root).launch()
        }
    }
}
