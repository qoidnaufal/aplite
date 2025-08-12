use aplite::prelude::*;
use AspectRatio::Defined;

fn first_row() -> impl IntoView {
    HStack::new()
        .child(
            Image::new(|| image_reader("examples/assets/image1.jpg"))
                .image_aspect_ratio(Defined((8, 5)))
        )
        .child(
            Image::new(|| image_reader("examples/assets/image2.jpg"))
        )
        .child(
            CircleWidget::new()
                .color(Rgba::PURPLE)
                .hover_color(Rgba::RED)
        )
        .spacing(40.)
        .padding(Padding::new(20., 20., 40., 40.))
        .corners(CornerRadius::splat(10.))
        .border_width(5.0)
        .color(Rgba::LIGHT_GRAY)
        .border_color(Rgba::DARK_GRAY)
}

fn button_stack(
    inc: impl Fn() + 'static,
    dec: impl Fn() + 'static,
    counter: SignalRead<i32>,
    set_counter: SignalWrite<i32>,
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
                .on(LeftClick, move || set_counter.set(0))
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
        .dragable()
        .padding(Padding::splat(10.))
        .spacing(5.)
        .min_width(400.)
        .align_h(AlignH::Center)
        .align_v(AlignV::Middle)
}

fn second_row(
    inc: impl Fn() + 'static,
    dec: impl Fn() + 'static,
    counter: SignalRead<i32>,
    set_counter: SignalWrite<i32>,
) -> impl IntoView {
    HStack::new()
        .child(button_stack(inc, dec, counter, set_counter))
        .child(
            CircleWidget::new()
                .color(rgba_hex("#104bcdbf"))
                .hover_color(Rgba::GREEN)
                .border_color(200.into())
                .border_width(3.0)
        )
        .color(Rgba::LIGHT_GRAY)
        .dragable()
        .padding(Padding::splat(30.))
        .spacing(5.)
}

fn root() -> impl IntoView {
    let (counter, set_counter) = Signal::split(0i32);

    let inc = move || set_counter.update(|num| *num += 1);
    let dec = move || set_counter.update(|num| *num -= 1);

    Effect::new(move |_| eprintln!("{:?}", counter.get()));

    let circle = CircleWidget::new()
        .color(Rgba::new(169, 72, 43, 255))
        .hover_color(Rgba::new(169, 72, 43, 200))
        .border_color(Rgba::WHITE)
        .border_width(5.0)
        .dragable();

    first_row()
        .and(second_row(inc, dec, counter, set_counter))
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
