# Aplite
This repo mainly serve as a learning process for me on retained mode GPU-rendered GUI (graphical user interface) with fine grained reactivity as the driver to trigger ui update.
Most of the stuffs in here are written from scratch, which took a lot of inspiration from others, such as:
- [`leptos`](https://github.com/leptos-rs/leptos),
- [`floem`](https://github.com/lapce/floem),
- [`cushy`](https://github.com/khonsulabs/cushy),
- [`vizia`](https://github.com/vizia/vizia),
- [`Ribir`](https://github.com/RibirX/Ribir),
- [`vello`](https://github.com/linebender/vello),
- [`vger-rs`](https://github.com/audulus/vger-rs),
- [`kludgine`](https://github.com/khonsulabs/kludgine),
- and many more.

### Architecture
WIP

### Example
Check the others on the [`examples`](./examples) folder.

```rust
use aplite::prelude::*;

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
        .corner_radius(CornerRadius::splat(47.))
        .dragable()
        .size((200, 69))
        .on(LeftClick, click_count);

    let circle = CircleWidget::new()
        .dragable()
        .border_width(6.0)
        .size((150, 150))
        .on(LeftClick, click_rotate);

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

fn get_color(val: u32) -> Rgba<u8> { /* ... */ }

fn get_shape(val: u32) -> Shape { /* ... */ }

fn main() -> ApliteResult {
    Aplite::new(simple)
        .set_window_attributes(|window| window.title = "Simple Demo".to_string())
        .launch()
}
```
### Name
In geological term [`Aplite`](https://en.wikipedia.org/wiki/Aplite) is a fine grained igneous rock.
