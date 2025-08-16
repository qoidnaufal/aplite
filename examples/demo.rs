use aplite::prelude::*;
use AspectRatio::Defined;

const IMAGE_1: &str = "../../Wallpaper/milky-way-over-mountains-4k-fl-1680x1050-2045764561.jpg";
const IMAGE_2: &str = "../../Wallpaper/1352909.jpeg";
const IMAGE_3: &str = "../../Wallpaper/pexels-daejeung-2734512.jpg";

fn image_row(counter: SignalRead<i32>) -> impl IntoView {
    let image1 = Image::new(|| image_reader(IMAGE_1))
        .min_width(350.)
        .image_aspect_ratio(Defined((16, 9)));

    let image2 = Image::new(|| image_reader(IMAGE_2))
        .min_width(350.)
        .image_aspect_ratio(Defined((16, 9)));

    let image3 = Image::new(|| image_reader(IMAGE_3))
        .min_width(350.)
        .image_aspect_ratio(Defined((16, 9)));

    let node1 = image1.node_ref();
    let node2 = image2.node_ref();
    let node3 = image3.node_ref();

    Effect::new(move |_| {
        let num = counter.get().abs();

        if num == 0 {
            node1.hide(false);
            node2.hide(false);
            node3.hide(false);
        } else if num % 2 == 0 {
            node1.hide(false);
            node2.hide(true);
            node3.hide(true);
        } else if num % 3 == 0 {
            node1.hide(true);
            node2.hide(false);
            node3.hide(true);
        } else {
            node1.hide(true);
            node2.hide(true);
            node3.hide(false);
        }
    });

    VStack::new()
        .child(image1)
        .child(image2)
        .child(image3)
        .spacing(20.)
        .padding(Padding::new(20., 20., 40., 40.))
        .border_width(5.0)
        .border_color(Rgba::DARK_GRAY)
}

fn button_stack(
    inc: impl Fn() + 'static,
    dec: impl Fn() + 'static,
    set_counter: SignalWrite<i32>,
) -> impl IntoView {
    HStack::new()
        .child(
            Button::new()
                .hover_color(Rgba::BLUE)
                .click_color(Rgba::DARK_GRAY)
                .border_width(5.0)
                .corners(CornerRadius::splat(50.))
                .on(LeftClick, dec)
        )
        .child(
            Button::new()
                .color(Rgba::GREEN)
                .hover_color(Rgba::LIGHT_GRAY)
                .click_color(Rgba::DARK_GREEN)
                .border_width(5.0)
                .corners(CornerRadius::splat(50.))
                .on(LeftClick, move || set_counter.set(0))
        )
        .child(
            Button::new()
                .color(Rgba::BLUE)
                .hover_color(Rgba::PURPLE)
                .border_width(5.0)
                .corners(CornerRadius::splat(50.))
                .on(LeftClick, inc)
        )
        .border_color(Rgba::LIGHT_GRAY)
        .padding(Padding::splat(7.))
        .spacing(5.)
        .min_width(430.)
        .align_h(AlignH::Center)
        .align_v(AlignV::Middle)
}

fn root() -> impl IntoView {
    let (counter, set_counter) = Signal::split(0i32);

    let inc = move || set_counter.update(|num| *num += 1);
    let dec = move || set_counter.update(|num| *num -= 1);

    Effect::new(move |_| eprint!("{:?}   \r", counter.get()));

    VStack::new()
        .child(button_stack(inc, dec, set_counter))
        .child(image_row(counter))
        .align_h(AlignH::Left)
        .padding(Padding::splat(20.0))
        .spacing(8.)
}

fn main() -> ApliteResult {
    Aplite::new(root)
        .set_window_attributes(|window| {
            window.title = "Demo".into();
        })
        .launch()
}
