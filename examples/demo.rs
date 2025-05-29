use aplite::prelude::*;

fn root(cx: &mut Context) {
    let (counter, set_counter) = Signal::new(0i32);
    eprintln!("{}", counter.get());

    let inc = move || {
        set_counter.update(|num| *num += 1);
    };

    let dec = move || {
        set_counter.update(|num| *num -= 1);
    };

    Effect::new(move || eprintln!("{}", counter.get()));

    HStack::new(cx, |cx| {
        use AspectRatio::Defined;
        Image::new(cx, || image_reader("examples/assets/image1.jpg")).with_aspect_ratio(Defined((8, 5)));
        Image::new(cx, || image_reader("examples/assets/image2.jpg")).style(|s| s.set_dragable(true));
        TestCircleWidget::new(cx).style(|style| {
            style.set_hover_color(Rgba::BLUE);
            style.set_fill_color(Rgba::PURPLE);
        });
    }).style(|style| {
        style.set_shape(Shape::RoundedRect);
        style.set_corners(|corner| corner.set_all(10));
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
                    style.set_stroke_width(5);
                    style.set_corners(|corners| {
                        corners.set_top_left(80);
                        corners.set_bot_left(80);
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
                    style.set_corners(|r| r.set_all(50));
                })
                .on_click(dec);
            Button::new(cx)
                .style(|style| {
                    style.set_fill_color(Rgba::BLUE);
                    style.set_hover_color(Rgba::YELLOW);
                    style.set_stroke_width(5);
                    style.set_corners(|corners| {
                        corners.set_top_left(0);
                        corners.set_bot_left(69);
                        corners.set_bot_right(0);
                        corners.set_top_right(69);
                    });
                });
            Button::new(cx)
                .style(|style| {
                    style.set_corners(|c| c.set_all(70));
                    style.set_fill_color(Rgba::YELLOW);
                });
        }).style(|style| {
            style.set_dragable(true);
            style.set_fill_color(Rgba::DARK_GREEN);
            style.set_min_width(400);
            style.set_alignment(|align| {
                align.set_h(HAlign::Left);
                align.set_v(VAlign::Bottom);
            });
            style.set_padding(|padding| padding.set_all(10));
            style.set_spacing(5);
        });

        TestCircleWidget::new(cx)
            .style(|style| {
                style.set_hover_color(Rgba::GREEN);
                style.set_stroke_width(3);
                style.set_fill_color(Rgba::BLACK);
                style.set_stroke_color(Rgba::RED);
            });

    }).style(|style| {
        style.set_dragable(true);
        style.set_fill_color(Rgba::LIGHT_GRAY);
        style.set_padding(|padding| {
            padding.set_all(30);
        });
        style.set_spacing(5);
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
    let (counter, set_counter) = Signal::new(0i32);

    let click = move || { set_counter.update(|num| *num += 1) };

    Effect::new(move || {
        eprintln!("{}", counter.get());
    });

    Button::new(cx)
        .style(|style| {
            style.set_size((200, 69));
            style.set_stroke_color(Rgba::WHITE);
            style.set_stroke_width(6);
            style.set_corners(|r| r.set_all(47));
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
