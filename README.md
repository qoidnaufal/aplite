# Learn Wgpu
Learn the inner process of a gui, especially the gpu-rendered one.<br>
Highly inspired by:
- [`leptos`](https://github.com/leptos-rs/leptos)
- [`cushy`](https://github.com/khonsulabs/cushy)
- [`floem`](https://github.com/lapce/floem)
- [`kludgine`](https://github.com/khonsulabs/kludgine)
- [`vger-rs`](https://github.com/audulus/vger-rs)

### Example
This is an example from the current (incomplete) works I've accomplished so far:

```rust
use learn_wgpu::prelude::*;

fn root() -> impl IntoView {
    let counter = signal(0i32);
    let set_counter = counter.clone();
    let hover = move |el: &mut Element| {
        let color = if set_counter.get() % 3 == 0 {
            Rgba::BLUE
        } else {
            Rgba::YELLOW
        };
        el.update_color(|c| *c = color);
    };
    let drag = |el: &mut Element| {
        el.update_color(|color| *color = Rgba::GREEN);
    };
    let click = move |_: &mut Element| {
        counter.set(|num| *num += 1);
    };

    button()
        .style(|style| {
            style.set_size((500, 200));
            style.set_stroke_color(Rgba::WHITE);
            style.set_stroke_width(10.);
            style.set_corners(|r| r.set_each(0.15));
        })
        .on_hover(hover)
        .on_drag(drag)
        .on_click(click)
}

fn main() -> AppResult {
    App::new(root)
        .set_window_properties(|window| {
            window.set_title("My App");
            window.set_decorations(false);
        })
        .launch()
}
```
