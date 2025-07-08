use aplite::prelude::*;
use AspectRatio::Defined;

fn root() -> impl IntoView {
    let (counter, set_counter) = Signal::new(0i32);

    let inc = move || {
        set_counter.update(|num| *num += 1);
    };

    let dec = move || {
        set_counter.update(|num| *num -= 1);
    };

    Effect::new(move |_| eprintln!("{}", counter.get()));

    let first_row = HStack::new()
        .append_child(
            Image::new(|| image_reader("examples/assets/image1.jpg"))
                .with_aspect_ratio(Defined((8, 5)))
        )
        .append_child(
            Image::new(|| image_reader("examples/assets/image2.jpg"))
                .set_dragable(true)
        )
        .append_child(
            TestCircleWidget::new()
                .set_color(|_| Rgba::PURPLE)
                .set_hover_color(|_| Rgba::RED)
        )
        .set_dragable(true)
        .state(|s| {
            s.set_spacing(40);
            s.set_padding(Padding::new(20, 20, 40, 40));
        })
        .set_corners(|_| CornerRadius::homogen(10))
        .set_stroke_width(|_| 5)
        .set_color(|_| Rgba::DARK_GRAY)
        .set_stroke_color(|_| Rgba::LIGHT_GRAY);

    let second_row = HStack::new().append_child(
        VStack::new().append_child(
            Button::new()
                .set_hover_color(|_| Rgba::BLUE)
                .set_stroke_width(|_| 5)
                .set_corners(|_| CornerRadius::new(80, 80, 0, 0))
                .set_click_color(|_| Rgba::GREEN)
                .on_click(inc)
        ).append_child(
            Button::new()
                .set_color(|_| Rgba::GREEN)
                .set_hover_color(|_| Rgba::WHITE)
                .set_click_color(|_| Rgba::GREEN)
                .set_stroke_width(|_| 5)
                .set_corners(|_| CornerRadius::homogen(50))
                .on_click(dec)
        ).append_child(
            Button::new()
                .set_color(|_| Rgba::BLUE)
                .set_hover_color(|_| Rgba::YELLOW)
                .set_stroke_width(|_| 5)
                .set_corners(|_| CornerRadius::new(0, 69, 0, 69))
        ).append_child(
            Button::new()
                .set_corners(|_| CornerRadius::homogen(70))
                .set_color(|_| Rgba::YELLOW)
        )
        .set_color(|_| Rgba::DARK_GREEN)
        .set_dragable(true)
        .state(|s| {
            s.set_min_width(400);
            s.set_alignment(|align| {
                align.set_h(HAlign::Left);
                align.set_v(VAlign::Bottom);
            });
            s.set_padding(Padding::all(10));
            s.set_spacing(5);
        })
    ).append_child(
        TestCircleWidget::new()
            .set_color(|_| Rgba::BLACK)
            .set_hover_color(|_| Rgba::GREEN)
            .set_stroke_color(|_| Rgba::RED)
            .set_stroke_width(|_| 3)
    )
    .set_color(|_| Rgba::LIGHT_GRAY)
    .set_dragable(true)
    .state(|s| {
        s.set_padding(Padding::all(30));
        s.set_spacing(5);
    });

    let circle = TestCircleWidget::new()
        .set_color(|_| Rgba::new(169, 72, 43, 255))
        .set_hover_color(|_| Rgba::BLACK)
        .set_stroke_color(|_| Rgba::WHITE)
        .set_stroke_width(|_| 5)
        .set_dragable(true);

    first_row
        .and(second_row)
        .and(circle)
}

struct Stats {
    duration: std::time::Duration,
    longest: std::time::Duration,
    shortest: std::time::Duration,
    counter: u32,
}

impl Stats {
    fn new() -> Self {
        Self {
            duration: std::time::Duration::from_nanos(0),
            longest: std::time::Duration::from_nanos(0),
            shortest: std::time::Duration::from_nanos(0),
            counter: 0,
        }
    }

    fn inc(&mut self, d: std::time::Duration) {
        if self.counter == 1 {
            self.longest = d;
            self.shortest = d;
            self.duration += d;
        } else {
            self.longest = self.longest.max(d);
            self.shortest = self.shortest.min(d);
            self.duration += d;
        }
        self.counter += 1;
    }
}

impl Drop for Stats {
    fn drop(&mut self) {
        let counter = self.counter - 1;
        let average = self.duration / counter;
        eprintln!();
        eprintln!(" > average:             {average:?}");
        eprintln!("   - hi:                {:?}", self.longest);
        eprintln!("   + lo:                {:?}", self.shortest);
        eprintln!(" > update amount:       {counter}");
    }
}

fn dummy() -> impl IntoView {
    let (counter, set_counter) = Signal::new(0i32);
    let set_time = RwSignal::new(Stats::new()).write_only();

    let click = move || {
        let start = std::time::Instant::now();
        set_counter.update(|num| *num += 1);
        set_time.update(|s| s.inc(start.elapsed()));
    };

    let color = move |_| {
        if counter.get() % 2 == 0 {
            Rgba::GREEN
        } else {
            Rgba::BLUE
        }
    };

    Effect::new(move |_| {
        counter.with(|num| eprintln!("{num}"))
    });

    let button = Button::new()
        .set_stroke_color(|_| Rgba::WHITE)
        .set_stroke_width(|_| 6)
        .set_corners(|_| CornerRadius::homogen(47))
        .set_dragable(true)
        .set_size((200, 69))
        .on_click(click);

    let circle = TestCircleWidget::new()
        .set_color(color)
        .set_stroke_width(|_| 6)
        .set_dragable(true)
        .set_size((150, 150));

    button.and(circle)
}

fn main() -> ApliteResult {
    let mut args = std::env::args();
    match args.nth(1) {
        Some(arg) if arg == "dummy" => {
            Aplite::new(dummy)
                .set_window_attributes(|window| {
                    window.set_title("Dummy");
                    // window.set_inner_size((500, 500));
                })
                // .set_background_color(Rgba::DARK_GRAY)
                .launch()
        },
        Some(arg) if arg == "empty" => {
            Aplite::new_empty()
                .set_window_attributes(|window| {
                    window.set_title("Empty");
                })
                // .set_background_color(Rgba::DARK_GREEN)
                .launch()
        }
        _ => {
            Aplite::new(root).launch()
        }
    }
}
