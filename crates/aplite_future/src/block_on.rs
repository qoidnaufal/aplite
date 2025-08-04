use std::future::Future;

pub(crate) enum State {
    Empty,
    Waiting,
    Notified,
}

pub(crate) struct Signal {
    state: std::sync::Mutex<State>,
    cond: std::sync::Condvar,
}

impl Signal {
    pub(crate) fn new() -> Self {
        Self {
            state: std::sync::Mutex::new(State::Empty),
            cond: std::sync::Condvar::new()
        }
    }

    pub(crate) fn wait(&self) {
        let mut state = self.state.lock().unwrap();
        match *state {
            State::Notified => *state = State::Empty,
            State::Waiting => unreachable!(),
            State::Empty => {
                *state = State::Waiting;
                while let State::Waiting = *state {
                    state = self.cond.wait(state).unwrap();
                }
            },
        }
    }

    pub(crate) fn notify(&self) {
        let mut state = self.state.lock().unwrap();
        match *state {
            State::Notified => {},
            State::Empty => *state = State::Notified,
            State::Waiting => {
                *state = State::Empty;
                self.cond.notify_one();
            },
        }
    }
}

impl std::task::Wake for Signal {
    fn wake(self: std::sync::Arc<Self>) {
        (&self).wake_by_ref();
    }

    fn wake_by_ref(self: &std::sync::Arc<Self>) {
        self.notify();
    }
}

pub fn block_on<F: std::future::IntoFuture>(fut: F) -> F::Output {
    let mut fut = core::pin::pin!(fut.into_future());
    let signal = std::sync::Arc::new(Signal::new());
    let waker = std::task::Waker::from(std::sync::Arc::clone(&signal));
    let mut context = std::task::Context::from_waker(&waker);

    loop {
        match fut.as_mut().poll(&mut context) {
            std::task::Poll::Ready(item) => break item,
            std::task::Poll::Pending => signal.wait(),
        }
    }
}
