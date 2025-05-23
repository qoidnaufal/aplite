pub struct Stats {
    counter: u32,
    render_time: std::time::Duration,
    startup_time: std::time::Duration,
    longest: std::time::Duration,
    shortest: std::time::Duration,
}

impl Stats {
    pub fn new() -> Self {
        Self {
            counter: 0,
            render_time: std::time::Duration::from_nanos(0),
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
            self.render_time += d;
        } else {
            self.longest = self.longest.max(d);
            self.shortest = self.shortest.min(d);
            self.render_time += d;
        }
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
            let counter = self.counter - 1;
            let average = self.render_time / counter;
            let fps = counter as f64 / self.render_time.as_secs_f64();
            eprintln!();
            eprintln!("startup:             {startup:?}");
            eprintln!("average:             {average:?}");
            eprintln!("hi:                  {:?}", self.longest);
            eprintln!("lo:                  {:?}", self.shortest);
            eprintln!("frames rendered:     {counter}");
            eprintln!("total time spent:    {:?}", self.render_time);
            eprintln!("fps:                 {:?}", fps.round() as usize);
        }
    }
}
