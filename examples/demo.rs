use aplite::prelude::*;

fn root() -> impl IntoView {
    let (counter, set_counter) = Signal::split(0i32);

    let inc = move || set_counter.update(|num| *num += 1);
    let dec = move || set_counter.update(|num| *num -= 1);

    Effect::new(move |_| eprintln!("count: {:?}", counter.get()));

    let button_1 = button("-", dec);
    let button_2 = button("+", inc);

    vstack((
        hstack((button_1, button_2))
            .style(|state| {
                state.padding = Padding::splat(5);
                state.spacing = Spacing::new(10);
            }),
        either(
            move || counter.get() % 2 == 0,
            circle,
            || button("dummy", || {}),
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
