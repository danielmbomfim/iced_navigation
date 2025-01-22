use iced::time::{Duration, Instant};

const DURATION: f32 = 0.2;

#[derive(Debug, Clone)]
pub struct Frame {
    start: Instant,
    percent: f32,
    duration: Duration,
}

impl Frame {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            percent: 0.0,
            duration: Duration::from_secs_f32(DURATION),
        }
    }

    pub fn update(&mut self) {
        let progress = (Instant::now() - self.start).as_secs_f32();

        self.percent = (progress / self.duration.as_secs_f32()).min(1.0) * 100.0;
    }

    pub fn is_complete(&self) -> bool {
        self.percent == 100.0
    }

    pub fn get_value(&self) -> f32 {
        self.percent
    }
}
