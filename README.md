# Aplite
This repo mainly serve as a learning material for me on retained mode GPU-rendered GUI (graphical user interface) with fine grained reactivity as the driver to trigger ui update.
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
    let (counter, set_counter) = Signal::create(0u32);
    let (rotate, set_rotate) = Signal::create(0.0);

    let color = move |_| get_color(counter.get());
    let shape = move |_| get_shape(counter.get());

    let click_count = move || set_counter.update(|num| *num += 1);
    let click_rotate = move || set_rotate.update(|val| *val += 30.0);

    Effect::new(move |_| counter.with(|val| eprint!("Counter: {val}    \r")));

    let button = Button::new()
        .set_stroke_color(|_| Rgba::WHITE)
        .set_stroke_width(|_| 6)
        .set_rotation(move |_| rotate.get())
        .set_corners(|_| CornerRadius::homogen(47))
        .set_dragable(true)
        .set_size((200, 69))
        .on_click(click_count);

    let circle = CircleWidget::new()
        .set_color(color)
        .set_shape(shape)
        .on_click(click_rotate)
        .set_stroke_width(|_| 6)
        .set_dragable(true)
        .set_size((150, 150));

    button.and(circle)
}

fn get_color(val: u32) -> Rgba<u8> { /* ... */ }

fn get_shape(val: u32) -> Shape { /* ... */ }

fn main() -> ApliteResult {
    Aplite::new(simple)
        .with_title("Dummy")
        .with_background_color(Rgba::DARK_GRAY)
        .launch()
}
```
### Name
In geological term [`Aplite`](https://en.wikipedia.org/wiki/Aplite) is a fine grained igneous rock.
