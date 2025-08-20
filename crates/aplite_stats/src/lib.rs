pub struct Stats {
    counter: u32,
    fps: usize,
    startup_time: std::time::Duration,
    longest: std::time::Duration,
    shortest: std::time::Duration,
}

impl Stats {
    pub fn new() -> Self {
        Self {
            counter: 0,
            fps: 0,
            startup_time: std::time::Duration::from_nanos(0),
            longest: std::time::Duration::from_nanos(0),
            shortest: std::time::Duration::from_nanos(0),
        }
    }

    pub fn inc(&mut self, d: std::time::Duration) {
        if self.counter == 0 {
            self.startup_time += d;
        } else if self.counter == 1 {
            self.longest = d;
            self.shortest = d;
        } else {
            self.longest = self.longest.max(d);
            self.shortest = self.shortest.min(d);
        }

        let fps = (0.1_f64 / d.as_secs_f64()).round() as usize;
        eprint!("fps: {fps}      \r");

        if self.counter > 0 { self.fps += fps }
        self.counter += 1;
    }
}

impl Drop for Stats {
    fn drop(&mut self) {
        if self.counter == 1 {
            let startup = self.startup_time;
            eprintln!("startup time: {startup:?}");
        } else {
            let startup = self.startup_time;
            let count = self.counter - 1;
            let fps = self.fps / self.counter as usize;

            eprintln!();
            eprintln!(" > startup:            {startup:?}");
            eprintln!(" > frames rendered:    {count}");
            eprintln!(" > avg fps:            {fps}");
            eprintln!("   - hi:               {:?}", self.longest);
            eprintln!("   + lo:               {:?}", self.shortest);
        }
    }
}
