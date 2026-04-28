pub struct Events<T> {
    current: Vec<T>,
    previous: Vec<T>,
}

impl<T> Default for Events<T> {
    fn default() -> Self {
        Self { current: Vec::new(), previous: Vec::new() }
    }
}

impl<T> Events<T> {
    /// Producer API
    /// Append a new event
    pub fn send(&mut self, event: T) {
        self.current.push(event);
    }

    /// Consumer API
    /// Tterate events from the current and previous frame.
    /// Reading both buffers means consumers see events regardless of
    /// whether they run before or after the producer in the schedule.
    pub fn read(&self) -> impl Iterator<Item = &T> {
        self.previous.iter()
    }

    /// Called once per frame to age out events
    /// Drops previous (now stale), current becomes previous
    pub fn swap(&mut self) {
        self.previous.clear();
        std::mem::swap(&mut self.current, &mut self.previous);
    }
}
