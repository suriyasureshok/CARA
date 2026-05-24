use crate::cluster::node::Node;

#[derive(Debug)]
pub struct ClusterState {
    pub nodes: Vec<Node>,
}
