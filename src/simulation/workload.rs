use rand::Rng;

use crate::cluster::request::{ConsistencyLevel, Request};

/// A simple workload generator that creates random requests with varying attributes to simulate real-world scenarios.
pub struct WorkloadGenerator {
    next_id: u64, // A counter to assign unique IDs to each generated request
}

/// Implementation of core methods for `WorkloadGenerator`.
impl WorkloadGenerator {
    pub fn new() -> Self {
        Self { next_id: 0 }
    }

    /// Generates a random request.
    ///
    /// This method generates a random request with varying compute needs, latency sensitivity, size, and consistency requirements.
    /// # Parameters
    /// - `next_id`: A unique identifier for the request, incremented with each generation
    ///
    /// # Returns
    /// A `Request` struct with randomized attributes:
    /// - `compute_need`: A random float between 0.0 and 1.
    /// - `latency_sensitivity`: A random float between 0.0 and 1.0, indicating how sensitive the request is to latency.
    /// - `size`: A random float between 0.0 and 1.0, representing the size of the request.
    /// - `consistency`: A randomly assigned consistency level, either Strong (30% chance) or Eventual (70% chance).
    pub fn generate(&mut self) -> Request {
        let mut rng = rand::thread_rng();

        self.next_id += 1;

        Request {
            id: self.next_id,

            compute_need: rng.gen_range(0.0..1.0),

            latency_sensitivity: rng.gen_range(0.0..1.0),

            size: rng.gen_range(0.0..1.0),

            // Randomly assign consistency level with a 30% chance for Strong and 70% chance for Eventual
            consistency: if rng.gen_bool(0.3) {
                ConsistencyLevel::Strong
            } else {
                ConsistencyLevel::Eventual
            },
        }
    }
}
