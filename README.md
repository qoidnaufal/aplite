# Aplite
In geological term [`Aplite`](https://en.wikipedia.org/wiki/Aplite) is a fine grained igneous rock.
My previous background as a geological engineering student, as well as my admiration toward [`leptos`](https://github.com/leptos-rs/leptos) guided me to pick this name.

This repo mainly serve as a learning process for me on retained mode GPU-rendered GUI (graphical user interface) with fine grained reactivity as the driver to trigger ui update.
I intended only to use [`wgpu`](https://github.com/gfx-rs/wgpu), [`winit`](https://github.com/rust-windowing/winit), and [`image`](https://github.com/image-rs/image) to be the only third party dependecies.
Everything else are written from scratch, which took a lot of inspiration from:
- [`leptos`](https://github.com/leptos-rs/leptos), their concept of fine grained reactivity which was inspired by [`solidjs`](https://github.com/solidjs/solid)
- [`floem`](https://github.com/lapce/floem), which take inspiration from `leptos`
- [`cushy`](https://github.com/khonsulabs/cushy), kind of similar with `floem`
- [`vizia`](https://github.com/vizia/vizia), I took their ecs-based system to manage the data
- [`kludgine`](https://github.com/khonsulabs/kludgine), and
- [`vger-rs`](https://github.com/audulus/vger-rs), which I read their whole codebase on how to do GPU-based rendering using `wgpu`
- [`Yarrow`](https://github.com/MeadowlarkDAW/Yarrow), and
- [`rootvg`](https://github.com/MeadowlarkDAW/rootvg), for the different ideas on managing primitives
- and many more

### Basic Architecture
`Renderer` is decoupled from view data management by `Context`, to make it easier to experiment on each side without changing so much on the other side.
Both of them are connected via `trait Render` & `fn submit_data(&mut Renderer) {}` which is kind of minimal i think. That's all.

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
            style.set_size((200, 69));
            style.set_stroke_color(Rgba::WHITE);
            style.set_stroke_width(6);
            style.set_corners(|r| r.set_all(47));
            style.set_dragable(true);
        })
        .on_click(click);
    TestCircleWidget::new(cx)
        .style(|style| {
            style.set_stroke_width(6);
            style.set_size((150, 150));
            style.set_dragable(true);
        });
}

fn main() -> AppResult {
    Aplite::new(root)
        .set_window_attributes(|window| {
            window.set_title("Dummy");
            window.set_inner_size((500, 500));
        })
        .set_background_color(Rgba::DARK_GRAY)
        .launch()
}
```
