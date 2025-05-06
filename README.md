# Aplite
In geological term [Aplite](https://en.wikipedia.org/wiki/Aplite) is a fine grained igneous rock.
My previous background as a geological engineering student, as well as my admiration toward [`leptos`](https://github.com/leptos-rs/leptos) guided me to pick this name.

This repo mainly serve me as a learning process to make a GPU-rendered GUI (graphical user interface).
I intended only to use ['wgpu'](https://github.com/gfx-rs/wgpu) and [`winit`](https://github.com/rust-windowing/winit) to be the only third party dependecies.
Everything else are written from scratch, which took a lot of inspiration from:
- [`leptos`](https://github.com/leptos-rs/leptos), their concept of fine grained reactivity which was inspired by [`solidjs`](https://github.com/solidjs/solid)
- [`floem`](https://github.com/lapce/floem), which take inspiration from `leptos`
- [`cushy`](https://github.com/khonsulabs/cushy), kind of similar with `floem`
- [`vizia`](https://github.com/vizia/vizia), I took their ecs-based system to manage the data
- [`kludgine`](https://github.com/khonsulabs/kludgine), and
- [`vger-rs`](https://github.com/audulus/vger-rs), which I read their whole codebase on how to do GPU-based rendering using `wgpu`

### Example
This is an example from the current (incomplete) works I've accomplished so far:

```rust
use aplite::prelude::*;

fn root(cx: &mut Context) {
    let (counter, set_counter) = signal(0i32);

    let click = move || {
        set_counter.set(|num| *num += 1);
        eprintln!("counter: {}", counter.get());
    };

    Button::new(cx)
        .style(|style| {
            style.set_size((500, 200));
            style.set_stroke_color(Rgba::WHITE);
            style.set_stroke_width(10.);
            style.set_corners(|r| r.set_each(0.15));
            style.set_dragable(true);
        })
        .on_click(click);
    TestCircleWidget::new(cx)
        .style(|style| {
            style.set_size((150, 150));
            style.set_dragable(true);
        });
}

fn main() -> AppResult {
    Aplite::new(dummy)
        .set_window_attributes(|window| {
            window.set_title("Dummy");
            window.set_inner_size((500, 500));
        })
        .set_background_color(Rgba::DARK_GRAY)
        .launch()
}
```
