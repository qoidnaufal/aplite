use aplite::prelude::*;

fn drag_child() -> impl IntoView {
    VStack::new()
        .child(Button::new())
        .child(Button::new())
        .child(Button::new())
        .dragable()
        .spacing(20)
        .align_h(AlignH::Center)
        .align_v(AlignV::Middle)
        .padding(Padding::splat(30))
        .border_color(Rgba::LIGHT_GRAY)
        .border_width(7.)
}

fn main() -> ApliteResult {
    Aplite::new(drag_child)
        .launch()
}
