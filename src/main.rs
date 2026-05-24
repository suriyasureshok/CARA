mod cluster;
mod metrics;
mod routing;
mod simulation;

use cluster::{
    node::{Node, NodeRole},
    state::ClusterState,
};

use simulation::engine::SimulationEngine;

use routing::{
    leader_only::LeaderOnlyRouter, least_loaded::LeastLoadedRouter, random::RandomRouter,
    round_robin::RoundRobinRouter, router::Router,
};

fn main() {
    let cluster = ClusterState {
        nodes: vec![
            Node {
                id: 1,
                cpu_usage: 0.2,
                memory_usage: 0.3,
                queue_length: 5,
                latency_ms: 10.0,
                log_index: 100,
                role: NodeRole::Leader,
                failed: false,
            },
            Node {
                id: 2,
                cpu_usage: 0.5,
                memory_usage: 0.4,
                queue_length: 7,
                latency_ms: 15.0,
                log_index: 98,
                role: NodeRole::Follower,
                failed: false,
            },
        ],
    };

    let mut sim = SimulationEngine::new(cluster);

    let request = sim.workload.generate();

    let mut random = RandomRouter;
    let mut rr = RoundRobinRouter::new();
    let mut least = LeastLoadedRouter;
    let mut leader = LeaderOnlyRouter;

    println!("Random: {:?}", random.route(&request, &sim.cluster));

    println!("Round Robin: {:?}", rr.route(&request, &sim.cluster));

    println!("Least Loaded: {:?}", least.route(&request, &sim.cluster));

    println!("Leader Only: {:?}", leader.route(&request, &sim.cluster));

    sim.tick();

    println!("Average Latency: {:.2}", sim.metrics.average_latency());

    println!("P95 Latency: {:.2}", sim.metrics.p95_latency());

    println!("P99 Latency: {:.2}", sim.metrics.p99_latency());
}
