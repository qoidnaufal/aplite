use aplite::prelude::*;

fn main() -> ApliteResult {
    let (counter, set_counter) = Signal::split(0_i32);
    let inc = move || set_counter.update(|num| *num += 2);

    button(move || counter.get(), inc).launch_with_default_config()
}
