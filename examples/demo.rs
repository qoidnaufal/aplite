use aplite::prelude::*;
use AspectRatio::Defined;

fn first_row() -> impl IntoView {
    HStack::new()
        .child(
            Image::new(|| image_reader("examples/assets/image1.jpg"))
                .state(|state| state.set_image_aspect_ratio(Defined((8, 5))))
        )
        .child(
            Image::new(|| image_reader("examples/assets/image2.jpg"))
        )
        .child(
            CircleWidget::new()
                .set_color(|_| Rgba::PURPLE)
                .set_hover_color(|| Rgba::RED)
        )
        .state(|s| {
            s.set_spacing(40.);
            s.set_padding(Padding::new(20., 20., 40., 40.));
        })
        .set_corners(|_| CornerRadius::splat(10.))
        .set_stroke_width(|_| 5)
        .set_color(|_| Rgba::LIGHT_GRAY)
        .set_stroke_color(|_| Rgba::DARK_GRAY)
}

fn button_stack(
    inc: impl Fn() + 'static,
    dec: impl Fn() + 'static,
    rotation: impl FnMut(Option<f32>) -> f32 + 'static,
    color: impl FnMut(Option<Rgba<u8>>) -> Rgba<u8> + 'static,
) -> impl IntoView {
    VStack::new()
        .child(
            Button::new()
                .set_hover_color(|| Rgba::BLUE)
                .set_stroke_width(|_| 5)
                .set_corners(|_| CornerRadius::new(80., 80., 0., 0.))
                .set_click_color(|| Rgba::DARK_GRAY)
                .on_click(inc)
        )
        .child(
            Button::new()
                .set_color(|_| Rgba::GREEN)
                .set_hover_color(|| Rgba::LIGHT_GRAY)
                .set_click_color(|| Rgba::DARK_GREEN)
                .set_stroke_width(|_| 5)
                .set_corners(|_| CornerRadius::splat(50.))
                .on_click(dec)
        )
        .child(
            Button::new()
                .set_color(|_| Rgba::BLUE)
                .set_hover_color(|| Rgba::PURPLE)
                .set_stroke_width(|_| 5)
                .set_corners(|_| CornerRadius::new(0., 69., 0., 69.))
        )
        .child(
            Button::new()
                .set_corners(|_| CornerRadius::splat(70.))
                .set_rotation(rotation)
                .set_color(color)
        )
        .set_color(|_| Rgba::new(0, 0, 0, 30))
        .set_dragable(true)
        .state(|s| {
            s.set_min_width(400.);
            s.set_alignment(|align| {
                align.set_h(AlignH::Center);
                align.set_v(AlignV::Middle);
            });
            s.set_padding(Padding::splat(10.));
            s.set_spacing(5.);
        })
}

fn second_row(
    inc: impl Fn() + 'static,
    dec: impl Fn() + 'static,
    rotation: impl FnMut(Option<f32>) -> f32 + 'static,
    color: impl FnMut(Option<Rgba<u8>>) -> Rgba<u8> + 'static,
) -> impl IntoView {
    HStack::new()
        .child(button_stack(inc, dec, rotation, color))
        .child(
            CircleWidget::new()
                .set_color(|_| rgba_hex("#104bcdbf"))
                .set_hover_color(|| Rgba::GREEN)
                .set_stroke_color(|_| 200.into())
                .set_stroke_width(|_| 3)
        )
        .set_color(|_| Rgba::LIGHT_GRAY)
        .set_dragable(true)
        .state(|s| {
            s.set_padding(Padding::splat(30.));
            s.set_spacing(5.);
        })
}

fn root() -> impl IntoView {
    let (counter, set_counter) = Signal::split(0i32);

    let inc = move || set_counter.update(|num| *num += 1);
    let dec = move || set_counter.update(|num| *num -= 1);
    let rotation = move |_| counter.with(|val| *val as f32 * 3.0);
    let color = move |_| select_color(counter.get());

    Effect::new(move |_| eprint!("{}        \r", counter.get()));

    let circle = CircleWidget::new()
        .set_color(|_| Rgba::new(169, 72, 43, 255))
        .set_hover_color(|| Rgba::new(169, 72, 43, 200))
        .set_stroke_color(|_| Rgba::WHITE)
        .set_stroke_width(|_| 5)
        .set_dragable(true);

    first_row()
        .and(second_row(inc, dec, rotation, color))
        .and(circle)
}

fn select_color(val: i32) -> Rgba<u8> {
    let val = val as u8;
    rgba_u8(
        val % 255,
        (val * 3) % 255,
        (val * 10) % 255,
        (255 - (val * 7) % 255).max(137)
    )
}

fn main() {
    let rt = aplite_future::Runtime::init_local();
    rt.spawn_local(async {
        Aplite::new(root)
            .set_window_attributes(|window| {
                window.title = "Demo".into();
            })
            .launch().unwrap();
    });
    rt.run();
}
