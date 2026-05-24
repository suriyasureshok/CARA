use crate::{
    cluster::state::ClusterState,
    metrics::collector::MetricsCollector,
    routing::{least_loaded::LeastLoadedRouter, router::Router},
    simulation::{clock::Clock, workload::WorkloadGenerator},
};
use std::time::Instant;
use crate::cluster::request::ConsistencyLevel;

pub struct SimulationEngine {
    pub clock: Clock,

    pub cluster: ClusterState,

    pub workload: WorkloadGenerator,

    pub metrics: MetricsCollector,
}

impl SimulationEngine {
    pub fn new(cluster: ClusterState) -> Self {
        Self {
            clock: Clock::new(),

            cluster,

            workload: WorkloadGenerator::new(),

            metrics: MetricsCollector::new(),
        }
    }

    pub fn tick(&mut self) {
        self.clock.advance();

        let request = self.workload.generate();

        let mut router = LeastLoadedRouter;

        // compute leader_log before taking mutable borrows of nodes
        let leader_log = self.cluster.nodes.iter().map(|n| n.log_index).max().unwrap_or(0);

        let start = Instant::now();

        let decision = router.route(&request, &self.cluster);

        let routing_time = start.elapsed();

        self.metrics.record_routing_ns(routing_time.as_nanos());

        if let Some(decision) = decision {
            if let Some(node) = self
                .cluster
                .nodes
                .iter_mut()
                .find(|n| n.id == decision.node_id)
            {
                let freshness = node.freshness(leader_log);

                let threshold = match request.consistency {
                    ConsistencyLevel::Strong => 0.90,
                    ConsistencyLevel::Eventual => 0.0,
                };

                if let crate::cluster::request::ConsistencyLevel::Strong = request.consistency {
                    if freshness < threshold {
                        self.metrics.stale_responses += 1;
                    }
                }

                let result = node.execute(&request);

                self.metrics.record(&result);
            }
        }

        for node in &mut self.cluster.nodes {
            node.update_state();
        }
    }
}
