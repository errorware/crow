use crate::lab::{EngineStatus, LocalTestNode};

/// Local test-VM lab: detected container engines, discovered nodes and the
/// "new node" modal.
pub struct LocalLabState {
    pub engines: Vec<EngineStatus>,
    pub nodes: Vec<LocalTestNode>,
    pub show_modal: bool,
    pub new_node_distro: String,
}

impl LocalLabState {
    pub fn new(engines: Vec<EngineStatus>, nodes: Vec<LocalTestNode>) -> Self {
        Self { engines, nodes, show_modal: false, new_node_distro: "noble".to_string() }
    }

    pub fn find_mut(&mut self, node_id: &str) -> Option<&mut LocalTestNode> {
        self.nodes.iter_mut().find(|n| n.id == node_id || n.name == node_id)
    }
}
