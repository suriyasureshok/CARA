use rand::seq::SliceRandom;

use crate::{
    cluster::{request::Request, state::ClusterState},
    routing::router::{Router, RoutingDecision},
};

pub struct RandomRouter;

impl Router for RandomRouter {
    fn route(&mut self, _request: &Request, cluster: &ClusterState) -> Option<RoutingDecision> {
        let mut rng = rand::thread_rng();

        cluster
            .nodes
            .iter()
            .filter(|n| n.is_available())
            .collect::<Vec<_>>()
            .choose(&mut rng)
            .map(|node| RoutingDecision {
                node_id: node.id,
                score: 0.0,
            })
    }
}
