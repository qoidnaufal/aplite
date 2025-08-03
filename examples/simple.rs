use aplite::prelude::*;

fn get_color(val: u32) -> Rgba<u8> {
    if val % 3 == 0 {
        Rgba::RED
    } else if val % 2 == 0 {
        Rgba::GREEN
    } else {
        Rgba::BLUE
    }
}

fn get_shape(val: u32) -> Shape {
    if val % 2 == 0 {
        Shape::Circle
    } else {
        Shape::RoundedRect
    }
}

fn simple() -> impl IntoView {
    let (counter, set_counter) = Signal::split(0u32);
    let (rotate, set_rotate) = Signal::split(0.0);

    let color = move |_| get_color(counter.get());
    let shape = move |_| get_shape(counter.get());

    let click_count = move || set_counter.update(|num| *num += 1);
    let click_rotate = move || set_rotate.update(|val| *val += 30.0);

    Effect::new(move |_| counter.read(|val| eprint!("Counter: {val}    \r")));

    let button = Button::new()
        .border_color(|_| Rgba::WHITE)
        .border_width(|_| 6)
        .rotation(move |_| rotate.get())
        .corners(|_| CornerRadius::splat(47.))
        .dragable(true)
        .size((200, 69))
        .on(LeftClick, click_count);

    let circle = CircleWidget::new()
        .color(color)
        .shape(shape)
        .on(LeftClick, click_rotate)
        .border_width(|_| 6)
        .dragable(true)
        .size((150, 150));

    button.and(circle)
}

fn main() -> ApliteResult {
    Aplite::new(simple)
        .set_window_attributes(|window| window.title = "Simple Demo".to_string())
        .launch()
}
