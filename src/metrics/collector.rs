use crate::cluster::execution::ExecutionResult;

#[derive(Debug)]
pub struct MetricsCollector {
    pub latencies: Vec<f64>,

    pub success_count: usize,

    pub failure_count: usize,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            latencies: Vec::new(),

            success_count: 0,

            failure_count: 0,
        }
    }

    pub fn record(
        &mut self,
        result: &ExecutionResult,
    ) {
        self.latencies.push(
            result.latency_ms,
        );

        if result.success {
            self.success_count += 1;
        } else {
            self.failure_count += 1;
        }
    }

    pub fn average_latency(
        &self,
    ) -> f64 {

        if self.latencies.is_empty() {
            return 0.0;
        }

        self.latencies.iter().sum::<f64>()
            /
        self.latencies.len() as f64
    }

    pub fn p95_latency(
        &self,
    ) -> f64 {

        if self.latencies.is_empty() {
            return 0.0;
        }

        let mut values =
            self.latencies.clone();

        values.sort_by(
            |a,b|
            a.partial_cmp(b).unwrap()
        );

        let idx =
            (values.len() as f64 * 0.95)
            as usize;

        values[idx.min(
            values.len()-1
        )]
    }

    pub fn p99_latency(
        &self,
    ) -> f64 {

        if self.latencies.is_empty() {
            return 0.0;
        }

        let mut values =
            self.latencies.clone();

        values.sort_by(
            |a,b|
            a.partial_cmp(b).unwrap()
        );

        let idx =
            (values.len() as f64 * 0.99)
            as usize;

        values[idx.min(
            values.len()-1
        )]
    }
}

