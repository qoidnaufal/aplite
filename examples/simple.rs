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

    let click_count = move || set_counter.update(|num| *num += 1);
    let click_rotate = move || set_rotate.update(|val| *val += 10.0);

    Effect::new(move |_| counter.with(|val| eprintln!("Counter: {val}")));

    let button = Button::new()
        .border_color(Rgba::WHITE)
        .color(rgba_hex("#104bcdbf"))
        .border_width(6.0)
        .dragable()
        .corners(CornerRadius::splat(47.))
        .size((200, 69))
        .on(LeftClick, click_count);

    let circle = CircleWidget::new()
        .dragable()
        .on(LeftClick, click_rotate)
        .border_width(6.0)
        .size((150, 150));

    let button_node = button.node_ref();
    let circle_node = circle.node_ref();

    Effect::new(move |_| counter.with(|num| {
        circle_node.set_color(get_color(*num));
        circle_node.set_shape(get_shape(*num));
    }));

    Effect::new(move |_| button_node.set_rotation_deg(rotate.get()));

    VStack::new()
        .child(button)
        .child(circle)
        .align_h(AlignH::Center)
        .padding(Padding::splat(20.0))
}

fn main() -> ApliteResult {
    Aplite::new(simple)
        .set_window_attributes(|window| window.title = "Simple Demo".to_string())
        .launch()
}
