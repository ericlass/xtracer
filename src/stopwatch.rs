pub struct StopWatch {
  start: u64,
  end: u64
}

impl StopWatch {
  pub fn new() -> StopWatch {
    StopWatch {
      start: 0,
      end: 0
    }
  }

  pub fn start(&mut self) {
    self.start = time::precise_time_ns();
  }

  pub fn stop(&mut self) {
    self.end = time::precise_time_ns();
  }

  pub fn get_millis(&self) -> f64 {
    (self.end - self.start) as f64 / 1000000.0
  }
}