use crate::{
    cluster::{request::Request, state::ClusterState},
    routing::router::{Router, RoutingDecision},
};

pub struct RoundRobinRouter {
    current: usize,
}

impl RoundRobinRouter {
    pub fn new() -> Self {
        Self { current: 0 }
    }
}

impl Router for RoundRobinRouter {
    fn route(&mut self, _request: &Request, cluster: &ClusterState) -> Option<RoutingDecision> {
        let available: Vec<_> = cluster.nodes.iter().filter(|n| n.is_available()).collect();

        if available.is_empty() {
            return None;
        }

        let idx = self.current % available.len();

        self.current += 1;

        Some(RoutingDecision {
            node_id: available[idx].id,
            score: 0.0,
        })
    }
}
