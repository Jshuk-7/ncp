pub struct Timer {
    instant: std::time::Instant,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            instant: std::time::Instant::now(),
        }
    }

    pub fn elapsed(&self) -> f32 {
        self.instant.elapsed().as_secs_f32()
    }
}
