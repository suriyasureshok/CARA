use crate::cluster::{
    node::{Node, NodeRole},
    state::ClusterState,
};

use crate::metrics::collector::MetricsCollector;
use crate::simulation::workload::WorkloadGenerator;
use crate::routing::{
    leader_only::LeaderOnlyRouter, least_loaded::LeastLoadedRouter, random::RandomRouter,
    round_robin::RoundRobinRouter, cara::CaraRouter, router::Router,
};

use std::time::Instant;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use plotters::prelude::*;

#[derive(Debug, Clone, Copy)]
pub enum RouterKind {
    Random,
    RoundRobin,
    LeastLoaded,
    LeaderOnly,
    Cara,
}

#[derive(Clone)]
pub struct BenchmarkResult {
    pub avg_latency: f64,
    pub p95: f64,
    pub p99: f64,
    pub throughput: f64,
    pub stale_rate: f64,
    pub routing_ns: f64,
    pub queue_std_dev: f64,
    pub failure_avoidance: f64,
}

fn fresh_cluster(node_count: usize) -> ClusterState {
    let mut nodes = Vec::new();

    for i in 0..node_count {
        nodes.push(Node {
            id: i + 1,
            cpu_usage: 0.2 + ((i as f64 * 0.05) % 1.0),
            memory_usage: 0.3,
            queue_length: 0,
            latency_ms: 10.0 + (i as f64),
            log_index: 100,
            role: if i == 0 { NodeRole::Leader } else { NodeRole::Follower },
            failed: false,
            health_score: 1.0,
        });
    }

    ClusterState { nodes }
}

fn create_router(kind: RouterKind) -> Box<dyn Router> {
    match kind {
        RouterKind::Random => Box::new(RandomRouter),
        RouterKind::RoundRobin => Box::new(RoundRobinRouter::new()),
        RouterKind::LeastLoaded => Box::new(LeastLoadedRouter),
        RouterKind::LeaderOnly => Box::new(LeaderOnlyRouter),
        RouterKind::Cara => Box::new(CaraRouter::new()),
    }
}

pub fn benchmark(kind: RouterKind, requests: usize, node_count: usize) -> BenchmarkResult {
    let mut cluster = fresh_cluster(node_count);

    let mut metrics = MetricsCollector::new();

    let mut workload = WorkloadGenerator::new();

    let mut router = create_router(kind);

    let start_total = Instant::now();

    for _ in 0..requests {
        let request = workload.generate();

        let leader_log = cluster.nodes.iter().map(|n| n.log_index).max().unwrap_or(0);

        let start = Instant::now();

        let decision = router.route(&request, &cluster);

        let routing_ns = start.elapsed().as_nanos();

        metrics.record_routing_ns(routing_ns);

        if let Some(decision) = decision {
            if let Some(node) = cluster.nodes.iter_mut().find(|n| n.id == decision.node_id) {
                let freshness = node.freshness(leader_log);

                if let crate::cluster::request::ConsistencyLevel::Strong = request.consistency {
                    if freshness < 0.90 {
                        metrics.stale_responses += 1;
                    }
                }

                let result = node.execute(&request);

                metrics.record(&result);
            } else {
                metrics.failed_requests += 1;
            }
        } else {
            metrics.failed_requests += 1;
        }

        for node in &mut cluster.nodes {
            node.update_state();
        }
    }

    let _duration = start_total.elapsed();

    let queues: Vec<usize> = cluster.nodes.iter().map(|n| n.queue_length).collect();

    BenchmarkResult {
        avg_latency: metrics.average_latency(),
        p95: metrics.p95_latency(),
        p99: metrics.p99_latency(),
        throughput: metrics.throughput(requests as u64),
        stale_rate: metrics.stale_rate(),
        routing_ns: metrics.avg_routing_ns(),
        queue_std_dev: MetricsCollector::queue_std_dev(&queues),
        failure_avoidance: metrics.failure_avoidance_rate(),
    }
}

pub fn benchmark_all(requests: usize, node_count: usize) {
    let kinds = vec![
        RouterKind::Random,
        RouterKind::RoundRobin,
        RouterKind::LeastLoaded,
        RouterKind::LeaderOnly,
        RouterKind::Cara,
    ];

    for kind in kinds {
        let name = match &kind {
            RouterKind::Random => "Random",
            RouterKind::RoundRobin => "RoundRobin",
            RouterKind::LeastLoaded => "LeastLoaded",
            RouterKind::LeaderOnly => "LeaderOnly",
            RouterKind::Cara => "CARA",
        };

        println!("Running benchmark for {}...", name);

        let res = benchmark(kind, requests, node_count);

        println!("{}\n------\nAvg Latency: {:.2}\nP95: {:.2}\nP99: {:.2}\nThroughput: {:.2}\nStale Rate: {:.4}\nRouting ns avg: {:.2}\nQueue StdDev: {:.2}\nFailure Avoidance: {:.4}\n", name, res.avg_latency, res.p95, res.p99, res.throughput, res.stale_rate, res.routing_ns, res.queue_std_dev, res.failure_avoidance);
    }
}

/// Run a sweep across node counts (4..=20) and request sizes (1k..=10k) and
/// write CSV results and generate plots. Produces a ranking summary as CSV and PNG.
pub fn benchmark_sweep(out_dir: &str) {
    let node_counts: Vec<usize> = (4..=20).collect();
    let request_sizes: Vec<usize> = (1..=10).map(|i| i * 1000).collect();

    let routers = vec![
        RouterKind::Random,
        RouterKind::RoundRobin,
        RouterKind::LeastLoaded,
        RouterKind::LeaderOnly,
        RouterKind::Cara,
    ];

    // ensure out_dir exists
    let path = Path::new(out_dir);
    if !path.exists() {
        std::fs::create_dir_all(path).unwrap();
    }

    let mut csv = File::create(path.join("benchmark_results.csv")).unwrap();
    writeln!(csv, "node_count,requests,router,avg_latency,p95,p99,throughput,stale_rate,routing_ns,queue_std_dev,failure_avoidance").unwrap();

    // accumulate ranking sums
    use std::collections::HashMap;
    let mut rank_sum: HashMap<String, f64> = HashMap::new();
    let mut rank_count: HashMap<String, usize> = HashMap::new();

    for &nc in &node_counts {
        // for plotting: for each router, collect (requests, avg_latency)
        let mut series: HashMap<String, Vec<(usize, f64)>> = HashMap::new();

        for &req in &request_sizes {
            // run each router
            let mut results: Vec<(String, BenchmarkResult)> = Vec::new();
            for r in &routers {
                let kind = *r;
                let res = benchmark(kind, req, nc);
                let name = match kind {
                    RouterKind::Random => "Random",
                    RouterKind::RoundRobin => "RoundRobin",
                    RouterKind::LeastLoaded => "LeastLoaded",
                    RouterKind::LeaderOnly => "LeaderOnly",
                    RouterKind::Cara => "CARA",
                };

                writeln!(csv, "{},{},{:.?},{:.6},{:.6},{:.6},{:.6},{:.6},{:.2},{:.6},{:.6}", nc, req, name, res.avg_latency, res.p95, res.p99, res.throughput, res.stale_rate, res.routing_ns, res.queue_std_dev, res.failure_avoidance).unwrap();

                    results.push((name.to_string(), res.clone()));

                    series.entry(name.to_string()).or_default().push((req, res.avg_latency));
            }

            // ranking by avg_latency (lower is better)
            results.sort_by(|a, b| a.1.avg_latency.partial_cmp(&b.1.avg_latency).unwrap());
            for (i, (name, _)) in results.iter().enumerate() {
                *rank_sum.entry(name.clone()).or_insert(0.0) += (i + 1) as f64;
                *rank_count.entry(name.clone()).or_insert(0) += 1;
            }
        }

        // generate plot for this node count
        let fname = path.join(format!("avg_latency_nodes_{}.png", nc));
        let root = BitMapBackend::new(fname.to_str().unwrap(), (1280, 720)).into_drawing_area();
        root.fill(&WHITE).unwrap();
        let mut chart = ChartBuilder::on(&root)
            .caption(format!("Avg Latency vs Requests (nodes={})", nc), ("sans-serif", 30).into_font())
            .margin(20)
            .x_label_area_size(40)
            .y_label_area_size(60)
            .build_cartesian_2d(1000usize..10000usize, 0f64..200f64)
            .unwrap();

        chart.configure_mesh().x_desc("requests").y_desc("avg latency").draw().unwrap();

        let colors = vec![RED, BLUE, GREEN, MAGENTA, CYAN];
        for (i, r) in routers.iter().enumerate() {
            let name = match r {
                RouterKind::Random => "Random",
                RouterKind::RoundRobin => "RoundRobin",
                RouterKind::LeastLoaded => "LeastLoaded",
                RouterKind::LeaderOnly => "LeaderOnly",
                RouterKind::Cara => "CARA",
            };

            if let Some(points) = series.get(name) {
                let color = colors[i].clone();
                chart
                    .draw_series(LineSeries::new(points.iter().map(|(x, y)| (*x, *y)), color.clone()))
                    .unwrap()
                    .label(name)
                    .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], color.clone()));
            }
        }

        chart.configure_series_labels().border_style(&BLACK).draw().unwrap();
    }

    // write ranking CSV and plot
    let mut rank_csv = File::create(path.join("ranking.csv")).unwrap();
    writeln!(rank_csv, "router,avg_rank").unwrap();
    let mut ranking: Vec<(String, f64)> = Vec::new();
    for (name, sum) in rank_sum.into_iter() {
        let count = rank_count.get(&name).cloned().unwrap_or(1) as f64;
        let avg = sum / count;
        writeln!(rank_csv, "{},{}", name, avg).unwrap();
        ranking.push((name, avg));
    }

    ranking.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    // bar chart for ranking
    let fname = path.join("ranking.png");
    let root = BitMapBackend::new(fname.to_str().unwrap(), (800, 600)).into_drawing_area();
    root.fill(&WHITE).unwrap();
    let max_rank = ranking.iter().map(|r| r.1).fold(0./0., f64::max);
    let mut chart = ChartBuilder::on(&root)
        .caption("Average Rank Across Sweeps", ("sans-serif", 24).into_font())
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(0..ranking.len(), 0f64..(max_rank + 1.0))
        .unwrap();

    chart.configure_mesh().x_labels(ranking.len()).disable_mesh().draw().unwrap();

    chart
        .draw_series(ranking.iter().enumerate().map(|(i, (name, avg))| {
            let x0 = i;
            let x1 = i + 1;
            let bar = Rectangle::new([(x0, 0.0), (x1, *avg)], BLUE.filled());
            bar
        }))
        .unwrap();

    // (labels omitted to avoid coordinate type issues; CSV contains router names and avg ranks)
}
