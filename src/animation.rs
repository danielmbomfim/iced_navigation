use iced::time::{Duration, Instant};

const DURATION: f32 = 0.3;

#[allow(missing_debug_implementations)]
#[derive(Debug, Clone)]
pub struct Frame {
    start: Instant,
    percent: f32,
    duration: Duration,
    f: Option<fn(f32) -> f32>,
}

impl Frame {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            duration: Duration::from_secs_f32(DURATION),
            percent: 0.0,
            f: None,
        }
    }

    pub fn map(mut self, f: fn(f32) -> f32) -> Self {
        self.f = Some(f);
        self
    }

    pub fn update(&mut self) {
        let progress = (Instant::now() - self.start).as_secs_f32();

        self.percent = (progress / self.duration.as_secs_f32()).min(1.0) * 100.0;
    }

    pub fn is_complete(&self) -> bool {
        self.percent == 100.0
    }

    pub fn get_value(&self) -> f32 {
        if let Some(f) = self.f {
            return f(self.percent);
        }

        self.percent
    }
}
