/// A simple clock to keep track of simulation time in discrete ticks.
#[derive(Debug)]
pub struct Clock {
    tick: u64,
}

/// Implementation of core methods for `Clock`.
impl Clock {
    pub fn new() -> Self {
        Self { tick: 0 }
    }

    /// Returns the current tick of the clock.
    ///
    /// # Returns
    /// The current tick as a `u64` value.
    pub fn current(&self) -> u64 {
        self.tick
    }

    /// Advances the clock by one tick.
    ///
    /// This method increments the internal tick counter, simulating the passage of time in the simulation.
    /// # Effects
    /// - Increments the `tick` field by 1.
    pub fn advance(&mut self) {
        self.tick += 1;
    }
}
