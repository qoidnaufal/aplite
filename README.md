# Learn Wgpu
Learn the inner process of a gui, especially the gpu-rendered one.<br>
Highly inspired by:
- [`leptos`](https://github.com/leptos-rs/leptos)
- [`floem`](https://github.com/lapce/floem)
- [`cushy`](https://github.com/khonsulabs/cushy)
- [`vizia`](https://github.com/vizia/vizia)
- [`kludgine`](https://github.com/khonsulabs/kludgine)
- [`vger-rs`](https://github.com/audulus/vger-rs)

### Example
This is an example from the current (incomplete) works I've accomplished so far:

```rust
use learn_wgpu::prelude::*;

fn root(cx: &mut Context) -> impl IntoView {
    let (counter, set_counter) = signal(0i32);

    let click = move |_| {
        set_counter.set(|num| *num += 1);
    };

    View::new(cx, |_| {
        Text::new(cx, |_| counter.get());
        Button::new(cx)
            .style(|style| {
                style.set_size((500, 200));
                style.set_stroke_color(Rgba::WHITE);
                style.set_stroke_width(10.);
                style.set_corners(|r| r.set_each(0.15));
            })
            .on_click(click);
    })
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
