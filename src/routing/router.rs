use crate::cluster::{request::Request, state::ClusterState};

#[derive(Debug, Clone)]
pub struct RoutingDecision {
    pub node_id: usize,
    pub score: f64,
}

pub trait Router {
    fn route(&mut self, request: &Request, cluster: &ClusterState) -> Option<RoutingDecision>;
}
