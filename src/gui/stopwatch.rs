use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug)]
pub struct Stopwatch {
    start_time: Option<Instant>,
    elapsed: Duration,
    running: bool,
    lap_number: u8
}


impl Stopwatch {
    pub fn new() -> Self {
        Stopwatch {
            start_time: None,
            elapsed: Duration::from_secs(0),
            running: false,
            lap_number: 1
        }
    }

    pub fn start(&mut self) {
        if !self.running {
            self.start_time = Some(Instant::now());
            self.running = true
        } else {
            println!("running can doi that")
        }
    }

    pub fn get_elapsed(&mut self) -> String {
        if self.running {
             self.elapsed = Instant::now() - self.start_time.unwrap_or_else(|| Instant::now());
            format!("{}.{}.{}", self.elapsed.as_secs() / 60, self.elapsed.as_secs() % 60, self.elapsed.subsec_millis() % 100)
        } else {
            format!("{}.{}.{}", self.elapsed.as_secs() / 60, self.elapsed.as_secs() % 60, self.elapsed.subsec_millis() % 100)
        }
    }

    pub fn pause_unpause(&mut self) {
        self.running = !self.running;
    }

    pub fn lap(&mut self) -> (u8 ,String) {
        self.lap_number += 1;
        (self.lap_number - 1, format!("{}.{}.{}", self.elapsed.as_secs() / 60, self.elapsed.as_secs() % 60, self.elapsed.subsec_millis() % 100))
    }
}