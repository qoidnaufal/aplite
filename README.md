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

### Basic Architecture & Performance
So far seems good, capable of rendering at 120 fps, with the help of `RenderBundle` and some (over?) complicated dirty state management.
Not gonna lie, there are some usage of raw pointer to bypass `rustc`'s borrow checker though.
Progress so far:
- Font rendering is WIP.
- Still learning about compute pass.
- Not sure if rasterization is correct, seems like a mess to me, will find out when I work on font rendering.
- Also not sure how to integrate dynamic texture (eg: video).
- Transform is also a mess.

### Example
Check the others on the [`examples`](./examples) folder.

```rust
use aplite::prelude::*;

fn root() -> impl IntoView {
    let (counter, set_counter) = Signal::split(0i32);

    let inc = move || set_counter.update(|num| *num += 1);
    let dec = move || set_counter.update(|num| *num -= 1);

    Effect::new(move |_| eprintln!("count: {:?}", counter.get()));

    let button_1 = button((), dec);
    let button_2 = button((), inc);

    vstack((
        hstack((button_1, button_2))
            .style(|state| {
                state.padding = Padding::splat(5);
                state.spacing = Spacing::new(10);
            }),
        either(
            move || counter.get() % 2 == 0,
            circle,
            || button((), || {}),
        ),
    ))
    .style(|state| {
        state.padding = Padding::splat(5);
        state.spacing = Spacing::new(5);
    })
}


fn main() -> ApliteResult {
    let config = AppConfig {
        window_inner_size: (500, 700).into(),
    };

    Aplite::new(config, root).launch()
}

```
### Name
In geological term [`Aplite`](https://en.wikipedia.org/wiki/Aplite) is a fine grained igneous rock.
