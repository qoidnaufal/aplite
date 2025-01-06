mod error;
mod shapes;
mod renderer;
mod pipeline;
mod app;
mod types;
mod shader;
mod color;
mod buffer;
mod layout;
mod gpu;
mod widget;

use std::{cell::RefCell, ops::DerefMut, rc::Rc};

use app::App;
use color::Rgb;
use shapes::Shape;
use widget::*;
use winit::event_loop::EventLoop;

use error::Error;

#[derive(Clone)]
struct Signal<T> {
    read: SignalRead<T>,
    write: SignalWrite<T>,
}

impl<T: Clone> Signal<T> {
    fn new(value: T) -> Self {
        let v = Rc::new(RefCell::new(value));
        Self {
            read: SignalRead(v.clone()),
            write: SignalWrite(v),
        }
    }

    fn get(&self) -> T {
        let val = self.read.0.as_ref().borrow();
        val.clone()
    }

    fn set<F: FnMut(&mut T) + 'static>(&self, mut f: F) {
        let mut val = self.write.0.borrow_mut();
        let v = val.deref_mut();
        f(v)
    }
}

#[derive(Clone)]
struct SignalRead<T>(Rc<RefCell<T>>);

#[derive(Clone)]
struct SignalWrite<T>(Rc<RefCell<T>>);

fn add_widget(app: &mut App) {
    let counter = Signal::new(0i32);
    eprintln!("init {}", counter.get());

    let c1 = counter.clone();
    let inc = move |shape: &mut Shape| {
        shape.set_color(|color| color.g += 150);
        c1.set(|num| *num += 1);
        eprintln!("inc1 {}", c1.get());
    };

    let c2 = counter.clone();
    let shift_left = move |shape: &mut Shape| {
        shape.set_color(|color| {
            color.r += 100;
            color.g += 100;
        });
        c2.set(|num| *num <<= 1);
        eprintln!("shift left {}", c2.get());
    };

    let c3 = counter.clone();
    let dec = move |shape: &mut Shape| {
        shape.set_color(|color| color.r += 150);
        c3.set(|num| *num -= 1);
        eprintln!("dec {}", c3.get());
    };

    let c4 = counter.clone();
    let right_shift = move |shape: &mut Shape| {
        shape.set_color(|color| *color = Rgb::BLACK);
        c4.set(|num| *num >>= 1);
        eprintln!("right shift {}", c4.get());
    };
    let drag = move |shape: &mut Shape| {
        shape.set_color(|color| *color = Rgb::GREEN);
        shape.set_position();
    };
    app
        .add_widget(Button::new().on_click(inc).on_drag(drag))
        .add_widget(TestWidget::new().on_click(dec).on_drag(drag))
        .add_widget(Image::new().on_click(shift_left).on_drag(drag))
        .add_widget(TestWidget::new().on_click(right_shift).on_drag(drag));
}

fn main() -> Result<(), Error> {
    env_logger::init();

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);

    let mut app = App::new();
    add_widget(&mut app);

    event_loop.run_app(&mut app)?;
    Ok(())
}
