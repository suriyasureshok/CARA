//! Metrics collection for latency, throughput, balance, reliability, and routing cost.
//!
//! The collector stores raw samples and exposes convenience methods used by the
//! benchmark sweep and final report generation.

use crate::cluster::execution::ExecutionResult;

/// Aggregated metrics gathered from benchmark runs.
#[derive(Debug)]
pub struct MetricsCollector {
    pub latencies: Vec<f64>,

    pub success_count: usize,

    pub failure_count: usize,
    pub routed_to_failed: usize,

    pub failed_requests: usize,
    pub total_requests: usize,

    pub stale_responses: usize,

    pub routing_latencies_ns: Vec<u128>,
}

impl MetricsCollector {
    /// Create a fresh metrics collector.
    pub fn new() -> Self {
        Self {
            latencies: Vec::new(),

            success_count: 0,

            failure_count: 0,
            routed_to_failed: 0,

            failed_requests: 0,
            total_requests: 0,

            stale_responses: 0,

            routing_latencies_ns: Vec::new(),
        }
    }

    /// Record a completed request execution.
    pub fn record(&mut self, result: &ExecutionResult) {
        let _ = result.request_id;
        let _ = result.node_id;

        self.latencies.push(result.latency_ms);

        self.total_requests += 1;

        if result.success {
            self.success_count += 1;
        } else {
            self.failure_count += 1;
            self.failed_requests += 1;
            self.routed_to_failed += 1;
        }
    }

    /// Record routing latency in nanoseconds.
    pub fn record_routing_ns(&mut self, ns: u128) {
        self.routing_latencies_ns.push(ns);
    }

    /// Compute the average end-to-end request latency.
    pub fn average_latency(&self) -> f64 {
        if self.latencies.is_empty() {
            return 0.0;
        }

        self.latencies.iter().sum::<f64>() / self.latencies.len() as f64
    }

    /// Compute the p95 latency from the recorded samples.
    pub fn p95_latency(&self) -> f64 {
        if self.latencies.is_empty() {
            return 0.0;
        }

        let mut values = self.latencies.clone();

        values.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let idx = (values.len() as f64 * 0.95) as usize;

        values[idx.min(values.len() - 1)]
    }

    /// Compute the p99 latency from the recorded samples.
    pub fn p99_latency(&self) -> f64 {
        if self.latencies.is_empty() {
            return 0.0;
        }

        let mut values = self.latencies.clone();

        values.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let idx = (values.len() as f64 * 0.99) as usize;

        values[idx.min(values.len() - 1)]
    }

    /// Compute requests per tick.
    pub fn throughput(&self, ticks: u64) -> f64 {
        if ticks == 0 {
            return 0.0;
        }

        self.total_requests as f64 / ticks as f64
    }

    /// Compute the standard deviation of queue lengths.
    pub fn queue_std_dev(queues: &[usize]) -> f64 {
        if queues.is_empty() {
            return 0.0;
        }

        let mean = queues.iter().sum::<usize>() as f64 / queues.len() as f64;

        let variance = queues
            .iter()
            .map(|q| {
                let diff = *q as f64 - mean;
                diff * diff
            })
            .sum::<f64>()
            / queues.len() as f64;

        variance.sqrt()
    }

    /// Compute the stale response rate.
    pub fn stale_rate(&self) -> f64 {
        if self.total_requests == 0 {
            return 0.0;
        }

        self.stale_responses as f64 / self.total_requests as f64
    }

    /// Compute average routing latency in nanoseconds.
    pub fn avg_routing_ns(&self) -> f64 {
        if self.routing_latencies_ns.is_empty() {
            return 0.0;
        }

        self.routing_latencies_ns.iter().sum::<u128>() as f64 / self.routing_latencies_ns.len() as f64
    }

    /// Compute the proportion of requests that avoided failures.
    pub fn failure_avoidance_rate(&self) -> f64 {
        if self.total_requests == 0 {
            return 0.0;
        }

        1.0 - (self.failed_requests as f64 / self.total_requests as f64)
    }
}
