use crate::{
    cluster::{node::NodeRole, request::Request, state::ClusterState},
    routing::router::{Router, RoutingDecision},
};

pub struct LeaderOnlyRouter;

impl Router for LeaderOnlyRouter {
    fn route(&mut self, _request: &Request, cluster: &ClusterState) -> Option<RoutingDecision> {
        cluster
            .nodes
            .iter()
            .find(|node| !node.failed && matches!(node.role, NodeRole::Leader))
            .map(|node| RoutingDecision {
                node_id: node.id,
                score: 0.0,
            })
    }
}
