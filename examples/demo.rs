use aplite::prelude::*;
use AspectRatio::Defined;

fn first_row() -> impl IntoView {
    HStack::new()
        .append_child(
            Image::new(|| image_reader("examples/assets/image1.jpg"))
                .state(|state| state.set_image_aspect_ratio(Defined((8, 5))))
        )
        .append_child(
            Image::new(|| image_reader("examples/assets/image2.jpg"))
                .set_dragable(true)
        )
        .append_child(
            CircleWidget::new()
                .set_color(|_| Rgba::PURPLE)
                .set_hover_color(|_| Rgba::RED)
        )
        .set_dragable(true)
        .state(|s| {
            s.set_spacing(40);
            s.set_padding(Padding::new(20, 20, 40, 40));
        })
        .set_corners(|_| CornerRadius::homogen(10))
        .set_stroke_width(|_| 5)
        .set_color(|_| Rgba::DARK_GRAY)
        .set_stroke_color(|_| Rgba::LIGHT_GRAY)
}

fn button_stack(
    inc: impl Fn() + 'static,
    dec: impl Fn() + 'static,
    rotation: impl FnMut(Option<f32>) -> f32 + 'static,
    color: impl FnMut(Option<Rgba<u8>>) -> Rgba<u8> + 'static,
) -> impl IntoView {
    VStack::new()
        .append_child(
            Button::new()
                .set_hover_color(|_| Rgba::BLUE)
                .set_stroke_width(|_| 5)
                .set_corners(|_| CornerRadius::new(80, 80, 0, 0))
                .set_click_color(|_| Rgba::DARK_GRAY)
                .on_click(inc)
        )
        .append_child(
            Button::new()
                .set_color(|_| Rgba::GREEN)
                .set_hover_color(|_| Rgba::LIGHT_GRAY)
                .set_click_color(|_| Rgba::DARK_GREEN)
                .set_stroke_width(|_| 5)
                .set_corners(|_| CornerRadius::homogen(50))
                .on_click(dec)
        )
        .append_child(
            Button::new()
                .set_color(|_| Rgba::BLUE)
                .set_hover_color(move |_| Rgba::PURPLE)
                .set_stroke_width(|_| 5)
                .set_corners(|_| CornerRadius::new(0, 69, 0, 69))
        )
        .append_child(
            Button::new()
                .set_corners(|_| CornerRadius::homogen(70))
                .set_rotation(rotation)
                .set_color(color)
        )
        .set_color(|_| Rgba::TRANSPARENT)
        .set_dragable(true)
        .state(|s| {
            s.set_min_width(400);
            s.set_alignment(|align| {
                align.set_h(HAlign::Left);
                align.set_v(VAlign::Bottom);
            });
            s.set_padding(Padding::all(10));
            s.set_spacing(5);
        })
}

fn second_row(
    inc: impl Fn() + 'static,
    dec: impl Fn() + 'static,
    rotation: impl FnMut(Option<f32>) -> f32 + 'static,
    color: impl FnMut(Option<Rgba<u8>>) -> Rgba<u8> + 'static,
) -> impl IntoView {
    HStack::new()
        .append_child(button_stack(inc, dec, rotation, color))
        .append_child(
            CircleWidget::new()
                .set_color(|_| Rgba::BLACK)
                .set_hover_color(|_| Rgba::GREEN)
                .set_stroke_color(|_| Rgba::RED)
                .set_stroke_width(|_| 3)
        )
        .set_color(|_| Rgba::LIGHT_GRAY)
        .set_dragable(true)
        .state(|s| {
            s.set_padding(Padding::all(30));
            s.set_spacing(5);
        })
}

fn root() -> impl IntoView {
    let (counter, set_counter) = Signal::create(0i32);

    let inc = move || set_counter.update(|num| *num += 1);
    let dec = move || set_counter.update(|num| *num -= 1);
    let rotation = move |_| counter.with(|val| *val as f32 * 3.0);
    let color = move |_| select_color(counter.get());

    Effect::new(move |_| eprintln!("{}", counter.get()));

    let circle = CircleWidget::new()
        .set_color(|_| Rgba::new(169, 72, 43, 255))
        .set_hover_color(|_| Rgba::BLACK)
        .set_stroke_color(|_| Rgba::WHITE)
        .set_stroke_width(|_| 5)
        .set_dragable(true);

    first_row()
        .and(second_row(inc, dec, rotation, color))
        .and(circle)
}

fn select_color(val: i32) -> Rgba<u8> {
    if val % 3 == 0 {
        Rgba::RED
    } else if val % 2 == 0 {
        Rgba::GREEN
    } else {
        Rgba::BLUE
    }
}

fn main() -> ApliteResult {
    Aplite::new(root).launch()
}
