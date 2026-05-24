//! CARA: a similarity-aware, consistency-filtered, diversity-aware routing benchmark.
//!
//! This binary generates comparative benchmark outputs for the routing algorithms in
//! `benchmark_output/`. The full report includes CSV data and PNG plots for latency,
//! rank, throughput, stale response rate, routing latency, and failure avoidance.

mod cluster;
mod benchmark;
mod metrics;
mod routing;
mod simulation;

use benchmark::runner::benchmark_sweep;

fn main() {
    // Run full benchmark sweep and generate final report (CSV + PNGs)
    benchmark_sweep("benchmark_output");

    println!("Benchmark sweep complete. Output written to ./cara/benchmark_output/");
}
