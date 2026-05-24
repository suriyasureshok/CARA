use crate::{
    cluster::{request::Request, state::ClusterState},
    routing::router::{Router, RoutingDecision},
};

pub struct LeastLoadedRouter;

impl Router for LeastLoadedRouter {
    fn route(&mut self, _request: &Request, cluster: &ClusterState) -> Option<RoutingDecision> {
        cluster
            .nodes
            .iter()
            .filter(|n| n.is_available())
            .min_by_key(|n| n.queue_length)
            .map(|node| RoutingDecision {
                node_id: node.id,
                score: 0.0,
            })
    }
}
