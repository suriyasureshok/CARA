use crate::cluster::{
    node::{Node, NodeRole},
    request::{ConsistencyLevel, Request},
    state::ClusterState,
};

use super::router::{Router, RoutingDecision};
use std::collections::HashMap;
use rand::distributions::{Distribution, WeightedIndex};
use rand::thread_rng;

const STRONG_CONSISTENCY_THRESHOLD: f64 = 0.90;
const EVENTUAL_CONSISTENCY_THRESHOLD: f64 = 0.0;

#[derive(Debug)]
pub struct NodeVector {
    pub cpu: f64,

    pub memory: f64,

    pub queue: f64,

    pub latency: f64,

    pub freshness: f64,

    pub role: f64,

    pub health: f64,
}

#[derive(Debug, Clone)]
struct Candidate {
    node_id: usize,
    score: f64,
}

#[derive(Debug)]
pub struct RequestVector {
    pub compute: f64,

    pub latency_importance: f64,

    pub consistency_weight: f64,
}

fn node_to_vector(node: &Node, leader_log: u64, max_queue: usize) -> NodeVector {
    NodeVector {
        cpu: node.cpu_usage,

        memory: node.memory_usage,

        queue: node.normalized_queue(max_queue),

        latency: node.latency_ms / 100.0,

        freshness: node.freshness(leader_log),

        role: match node.role {
            NodeRole::Leader => 1.0,
            NodeRole::Follower => 0.5,
        },

        health: node.health_score,
    }
}

fn request_to_vector(request: &Request) -> RequestVector {
    RequestVector {
        compute: request.compute_need,

        latency_importance: request.latency_sensitivity,

        consistency_weight: match request.consistency {
            ConsistencyLevel::Strong => 1.0,

            ConsistencyLevel::Eventual => 0.2,
        },
    }
}

#[derive(Debug)]
struct Weights {
    cpu: f64,

    queue: f64,

    latency: f64,

    freshness: f64,

    role: f64,
    health: f64,
}

fn adaptive_weights(req: &RequestVector) -> Weights {
    Weights {
        cpu: 0.2 + req.compute * 0.4,

        queue: 0.2 + req.compute * 0.3,

        latency: 0.2 + req.latency_importance * 0.5,

        freshness: 0.1 + req.consistency_weight * 0.5,

        role: 0.1 + req.consistency_weight * 0.3,

        health: 0.30,
    }
}

fn distance(node: &NodeVector, weights: &Weights) -> f64 {
    let role_penalty = 1.0 - node.role;

    weights.cpu * node.cpu
        + weights.queue * node.queue
        + weights.latency * node.latency
        + weights.freshness * (1.0 - node.freshness)
        + weights.role * role_penalty
        + weights.health * (1.0 - node.health)
}

fn freshness_threshold(request: &Request) -> f64 {
    match request.consistency {
        ConsistencyLevel::Strong => STRONG_CONSISTENCY_THRESHOLD,
        ConsistencyLevel::Eventual => EVENTUAL_CONSISTENCY_THRESHOLD,
    }
}

pub struct CaraRouter {
    recent_selections: HashMap<usize, u64>,
    current_tick: u64,
}

impl CaraRouter {
    pub fn new() -> Self {
        Self {
            recent_selections: HashMap::new(),
            current_tick: 0,
        }
    }

    fn diversity_penalty(&self, node_id: usize, current_tick: u64) -> f64 {
        match self.recent_selections.get(&node_id) {
            Some(last_tick) => {
                let age = current_tick.saturating_sub(*last_tick);
                if age >= 10 {
                    0.0
                } else {
                    (10 - age) as f64 * 0.03
                }
            }
            None => 0.0,
        }
    }

    fn score_to_weight(score: f64) -> f64 {
        1.0 / (score + 0.0001)
    }
}

impl Router for CaraRouter {
    fn route(&mut self, request: &Request, cluster: &ClusterState) -> Option<RoutingDecision> {
        // advance local time for each decision
        self.current_tick += 1;

        let req = request_to_vector(request);
        let weights = adaptive_weights(&req);
        let leader_log = cluster.nodes.iter().map(|n| n.log_index).max().unwrap_or(0);
        let max_queue = cluster
            .nodes
            .iter()
            .map(|n| n.queue_length)
            .max()
            .unwrap_or(1);

        let current_tick = self.current_tick;

        let threshold = freshness_threshold(request);

        let mut candidates: Vec<Candidate> = cluster
            .nodes
            .iter()
            .filter(|node| {
                if node.failed {
                    return false;
                }

                let freshness = node.freshness(leader_log);

                freshness >= threshold
            })
            .map(|node| {
                let vec = node_to_vector(node, leader_log, max_queue);

                let base_score = distance(&vec, &weights);

                let penalty = self.diversity_penalty(node.id, current_tick);

                let score = base_score + penalty;

                Candidate { node_id: node.id, score }
            })
            .collect();

        if candidates.is_empty() {
            return None;
        }

        candidates.sort_by(|a, b| a.score.partial_cmp(&b.score).unwrap());

        const TOP_K: usize = 3;

        let top_k = candidates.into_iter().take(TOP_K).collect::<Vec<_>>();

        // debug printing disabled to keep output clean for final reports

        // convert scores to weights (lower score => higher weight)
        let weights_vec: Vec<f64> = top_k.iter().map(|c| Self::score_to_weight(c.score)).collect();

        let dist = WeightedIndex::new(&weights_vec).unwrap();

        let mut rng = thread_rng();

        let idx = dist.sample(&mut rng);

        let chosen = &top_k[idx];

        // update diversity state
        self.recent_selections.insert(chosen.node_id, current_tick);

        Some(RoutingDecision { node_id: chosen.node_id, score: chosen.score })
    }
}
