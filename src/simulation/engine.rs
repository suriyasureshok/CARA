use crate::{
    cluster::state::ClusterState,
    metrics::collector::MetricsCollector,
    routing::{least_loaded::LeastLoadedRouter, router::Router},
    simulation::{clock::Clock, workload::WorkloadGenerator},
};

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

        if let Some(decision) = router.route(&request, &self.cluster) {
            if let Some(node) = self
                .cluster
                .nodes
                .iter_mut()
                .find(|n| n.id == decision.node_id)
            {
                let result = node.execute(&request);

                self.metrics.record(&result);
            }
        }

        for node in &mut self.cluster.nodes {
            node.update_state();
        }

        println!("Tick {}", self.clock.current());

        println!("Generated Request: {:?}", request);
    }
}
