#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub request_id: u64,

    pub node_id: usize,

    pub latency_ms: f64,

    pub success: bool,
}
