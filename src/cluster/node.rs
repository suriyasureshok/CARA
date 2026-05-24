use rand::Rng;

use crate::cluster::{execution::ExecutionResult, request::Request};

#[derive(Debug, Clone)]
pub enum NodeRole {
    Leader,
    Follower,
}

#[derive(Debug, Clone)]
pub struct Node {
    pub id: usize,
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub queue_length: usize,
    pub latency_ms: f64,
    pub log_index: u64,
    pub role: NodeRole,
    pub failed: bool,
    pub health_score: f64,
}

impl Node {
    pub fn is_available(&self) -> bool {
        !self.failed
    }

    pub fn freshness(&self, leader_log: u64) -> f64 {
        1.0 / (1.0 + (leader_log - self.log_index) as f64)
    }

    pub fn normalized_queue(&self, max_queue: usize) -> f64 {
        if max_queue == 0 {
            return 0.0;
        }

        self.queue_length as f64 / max_queue as f64
    }

    pub fn update_state(&mut self) {
        let mut rng = rand::thread_rng();

        self.cpu_usage = (self.cpu_usage + rng.gen_range(-0.05..0.05)).clamp(0.0, 1.0);

        self.memory_usage = (self.memory_usage + rng.gen_range(-0.05..0.05)).clamp(0.0, 1.0);

        self.latency_ms = (self.latency_ms + rng.gen_range(-2.0..2.0)).max(1.0);

        self.health_score = (self.health_score + rng.gen_range(-0.03..0.03)).clamp(0.0, 1.0);

        if self.health_score < 0.10 {
            self.failed = true;
        } else if self.health_score > 0.30 {
            self.failed = false;
        }
    }

    pub fn execute(&mut self, request: &Request) -> ExecutionResult {
        self.queue_length += 1;

        let queue_penalty = self.queue_length as f64 * 2.0;

        let cpu_penalty = self.cpu_usage * 50.0;

        let request_penalty = request.compute_need * 25.0;

        let latency = self.latency_ms + queue_penalty + cpu_penalty + request_penalty;

        self.queue_length -= 1;

        ExecutionResult {
            request_id: request.id,

            node_id: self.id,

            latency_ms: latency,

            success: !self.failed,
        }
    }
}
