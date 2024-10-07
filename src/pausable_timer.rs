use std::time::{Duration, Instant};

#[derive(Debug, PartialEq, Eq)]
enum State {
    InProgress,
    Paused,
}

#[derive(Debug)]
pub struct Timer {
    start_time: Instant,
    accumulated_time: Duration,
    state: State,
}

impl Timer {
    pub fn reset_time(&mut self) {
        self.start_time = Instant::now();
        self.accumulated_time = Duration::default();
    }

    pub fn pause(&mut self) {
        self.accumulated_time = self
            .accumulated_time
            .saturating_add(self.start_time.elapsed());
        self.state = State::Paused;
    }

    pub fn resume(&mut self) {
        self.start_time = Instant::now();
        self.state = State::InProgress;
    }

    pub fn elapsed(&self) -> Duration {
        match self.state {
            State::InProgress => self
                .accumulated_time
                .saturating_add(self.start_time.elapsed()),
            State::Paused => self.accumulated_time,
        }
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self {
            start_time: Instant::now(),
            accumulated_time: Duration::default(),
            state: State::InProgress,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_timer() {
        let timer = Timer::default();

        assert!(timer.elapsed().as_millis() < 100);
        assert_eq!(timer.state, State::InProgress);
    }
}
