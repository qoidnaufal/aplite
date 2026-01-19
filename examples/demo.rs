use aplite::prelude::*;

fn root() -> impl IntoView {
    let (counter, set_counter) = Signal::split(0i32);

    let inc = move || set_counter.update(|num| *num += 1);
    let dec = move || set_counter.update(|num| *num -= 1);

    Effect::new(move |_| eprintln!("count: {:?}", counter.get()));

    let button_1 = button("+", inc).style(|s| s.corner_radius = CornerRadius::splat(10));
    let button_2 = button("-", dec);

    vstack((
        hstack((button_1, button_2)),
        either(
            move || counter.get() % 2 == 0,
            circle,
            || button("dummy", || {}),
        ),
    ))
    .style(|state| state.padding = Padding::splat(5))
}


fn main() -> ApliteResult {
    let config = AppConfig {
        executor_capacity: 1,
        window_inner_size: (500, 700).into(),
    };

    // root.launch(config)
    // root.launch_with_default_config()
    Aplite::new(config, root).launch()
}
