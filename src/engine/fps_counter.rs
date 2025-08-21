use core::time;
use std::time::Instant;

pub struct FpsCounter {
    frame_count: u32,
    last_update: Instant,
    fps: f32,
    update_interval: f32,
}

impl FpsCounter {
    pub fn new() -> Self {
        Self {
            frame_count: 0,
            last_update: Instant::now(),
            fps: 0.0,
            update_interval: 1.0,
        }
    }

    pub fn update(&mut self) -> f32 {
        self.frame_count += 1;
        let now = Instant::now();
        let time_elapsed = now.duration_since(self.last_update).as_secs_f32();

        if time_elapsed >= self.update_interval {
            self.fps = (self.frame_count as f32) / time_elapsed;
            self.frame_count = 0;
            self.last_update = now;
        }

        self.fps
    }

    pub fn get_fps(&self) -> f32 {
        self.fps
    }
}
