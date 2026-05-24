mod cluster;
mod benchmark;
mod metrics;
mod routing;
mod simulation;

use cluster::{
    node::{Node, NodeRole},
    state::ClusterState,
};

use routing::cara::CaraRouter;
use simulation::engine::SimulationEngine;
use crate::metrics::collector::MetricsCollector;

use routing::{
    leader_only::LeaderOnlyRouter, least_loaded::LeastLoadedRouter, random::RandomRouter,
    round_robin::RoundRobinRouter, router::Router,
};
use benchmark::runner::benchmark_sweep;

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
                health_score: 1.0,
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
                health_score: 1.0,
            },
        ],
    };

    let mut sim = SimulationEngine::new(cluster);

    let request = sim.workload.generate();

    let mut random = RandomRouter;
    let mut rr = RoundRobinRouter::new();
    let mut least = LeastLoadedRouter;
    let mut leader = LeaderOnlyRouter;
    let mut cara = CaraRouter::new();

    println!("Random: {:?}", random.route(&request, &sim.cluster));

    println!("Round Robin: {:?}", rr.route(&request, &sim.cluster));

    println!("Least Loaded: {:?}", least.route(&request, &sim.cluster));

    println!("Leader Only: {:?}", leader.route(&request, &sim.cluster));

    // Run full benchmark sweep and generate final report (CSV + PNGs)
    benchmark_sweep("benchmark_output");

    println!("Benchmark sweep complete. Output written to ./cara/benchmark_output/");
}
