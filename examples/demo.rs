use aplite::prelude::*;

fn root() -> impl IntoView {
    let (counter, set_counter) = Signal::split(0i32);

    let inc = move || set_counter.update(|num| *num += 1);
    let dec = move || set_counter.update(|num| *num -= 1);

    // Effect::new(move |_| eprintln!("count: {:?}", counter.get()));

    let button_1 = button("A", dec);
    let button_2 = button("B", inc);

    vstack((
        text(move || counter.get())
            .style(|s| s.color = rgb(0xebdbb2)),
        hstack((button_1, button_2))
            .style(|s| {
                s.padding = Padding::splat(5);
                s.spacing = Spacing::new(10);
            }),
        either(
            move || counter.get() % 2 == 0,
            circle,
            || button((), || {}),
        ),
    ))
    .style(|s| {
        s.padding = Padding::splat(5);
        s.spacing = Spacing::new(5);
    })
}


fn main() -> ApliteResult {
    let config = AppConfig {
        window_inner_size: (500, 700).into(),
    };

    // root.launch(config)
    Aplite::new(config, root).launch()
}
