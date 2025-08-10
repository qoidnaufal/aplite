use aplite::prelude::*;
use AspectRatio::Defined;

fn first_row() -> impl IntoView {
    HStack::new()
        .child(
            Image::new(|| image_reader("examples/assets/image1.jpg"))
                .set_state(|state| state.set_image_aspect_ratio(Defined((8, 5))))
        )
        .child(
            Image::new(|| image_reader("examples/assets/image2.jpg"))
        )
        .child(
            CircleWidget::new()
                .color(Rgba::PURPLE)
                .hover_color(Rgba::RED)
        )
        .set_state(|s| {
            s.set_spacing(40.);
            s.set_padding(Padding::new(20., 20., 40., 40.));
        })
        .corners(CornerRadius::splat(10.))
        .border_width(5.0)
        .color(Rgba::LIGHT_GRAY)
        .border_color(Rgba::DARK_GRAY)
}

fn button_stack(
    inc: impl Fn() + 'static,
    dec: impl Fn() + 'static,
    counter: SignalRead<i32>,
) -> impl IntoView {
    VStack::new()
        .child(
            Button::new()
                .hover_color(Rgba::BLUE)
                .border_width(5.0)
                .corners(CornerRadius::new(80., 80., 0., 0.))
                .click_color(Rgba::DARK_GRAY)
                .on(LeftClick, inc)
        )
        .child(
            Button::new()
                .color(Rgba::GREEN)
                .hover_color(Rgba::LIGHT_GRAY)
                .click_color(Rgba::DARK_GREEN)
                .border_width(5.0)
                .corners(CornerRadius::splat(50.))
                .on(LeftClick, dec)
        )
        .child(
            Button::new()
                .color(Rgba::BLUE)
                .hover_color(Rgba::PURPLE)
                .border_width(5.0)
                .corners(CornerRadius::new(0., 69., 0., 69.))
        )
        .child({
            let button = Button::new().corners(CornerRadius::splat(70.));

            let node = button.node();

            Effect::new(move |_| {
                counter.with(|num| {
                    node.set_color(select_color(*num));
                    node.set_rotation_deg(*num as f32 * 3.0);
                })
            });

            button
        })
        .color(Rgba::new(0, 0, 0, 30))
        .dragable(true)
        .set_state(|s| {
            s.set_min_width(400.);
            s.set_align_h(AlignH::Center);
            s.set_align_v(AlignV::Middle);
            s.set_padding(Padding::splat(10.));
            s.set_spacing(5.);
        })
}

fn second_row(
    inc: impl Fn() + 'static,
    dec: impl Fn() + 'static,
    counter: SignalRead<i32>,
) -> impl IntoView {
    HStack::new()
        .child(button_stack(inc, dec, counter))
        .child(
            CircleWidget::new()
                .color(rgba_hex("#104bcdbf"))
                .hover_color(Rgba::GREEN)
                .border_color(200.into())
                .border_width(3.0)
        )
        .color(Rgba::LIGHT_GRAY)
        .dragable(true)
        .set_state(|s| {
            s.set_padding(Padding::splat(30.));
            s.set_spacing(5.);
        })
}

fn root() -> impl IntoView {
    let (counter, set_counter) = Signal::split(0i32);

    let inc = move || set_counter.update(|num| *num += 1);
    let dec = move || set_counter.update(|num| *num -= 1);

    Effect::new(move |_| eprint!("{}        \r", counter.get()));

    let circle = CircleWidget::new()
        .color(Rgba::new(169, 72, 43, 255))
        .hover_color(Rgba::new(169, 72, 43, 200))
        .border_color(Rgba::WHITE)
        .border_width(5.0)
        .dragable(true);

    first_row()
        .and(second_row(inc, dec, counter))
        .and(circle)
}

fn select_color(val: i32) -> Rgba<u8> {
    let val = val as u32;
    Rgba::from_u32(val)
}

fn main() -> ApliteResult {
    Aplite::new(root)
        .set_window_attributes(|window| {
            window.title = "Demo".into();
        })
        .launch()
}
