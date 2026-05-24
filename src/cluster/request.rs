#[derive(Debug, Clone)]
pub enum ConsistencyLevel {
    Strong,
    Eventual,
}

#[derive(Debug, Clone)]
pub struct Request {
    pub id: u64,
    pub compute_need: f64,
    pub latency_sensitivity: f64,
    pub size: f64,
    pub consistency: ConsistencyLevel,
}
