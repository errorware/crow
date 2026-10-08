//! Where each node sits on the map, in world coordinates (before pan and
//! zoom): Crow with its keys under it, then servers by how many hops they
//! are from Crow (direct, behind one jump host, …), each column sorted by
//! lane (env) and, behind a jump host, kept next to the host it goes
//! through. Every line runs left to right.

use std::collections::HashMap;

use super::{EdgeKind, Graph, NodeKind};

pub const NODE_W: f32 = 210.0;
pub const NODE_H: f32 = 56.0;
const COL_GAP: f32 = 110.0;
const ROW_GAP: f32 = 12.0;
/// Room for a lane's heading above its first node.
const LANE_GAP: f32 = 30.0;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Layout {
    /// Node id → top-left corner.
    pub at: HashMap<String, (f32, f32)>,
    /// Lane headings: (text, x, y).
    pub lanes: Vec<(String, f32, f32)>,
    pub width: f32,
    pub height: f32,
}

/// Hops from Crow: 1 for a server Crow reaches directly, +1 per jump host.
fn depths(g: &Graph) -> HashMap<String, usize> {
    let parent: HashMap<&str, &str> = g.edges.iter().filter(|e| matches!(e.kind, EdgeKind::ViaJump | EdgeKind::Runs)).map(|e| (e.to.as_str(), e.from.as_str())).collect();
    let mut out = HashMap::new();
    for n in g.nodes.iter().filter(|n| !matches!(n.kind, NodeKind::Crow | NodeKind::Key)) {
        let (mut d, mut at) = (1usize, n.id.as_str());
        // A jump chain is short; a cycle (bad data) stops at 8.
        while let Some(p) = parent.get(at) {
            d += 1;
            at = p;
            if d > 8 {
                break;
            }
        }
        out.insert(n.id.clone(), d);
    }
    out
}

pub fn layout(g: &Graph) -> Layout {
    let depth = depths(g);
    let max_depth = depth.values().copied().max().unwrap_or(1);
    let parent: HashMap<&str, &str> = g.edges.iter().filter(|e| matches!(e.kind, EdgeKind::ViaJump | EdgeKind::Runs)).map(|e| (e.to.as_str(), e.from.as_str())).collect();
    // Column 0: Crow, then its keys; 1…: servers by depth.
    let mut columns: Vec<Vec<&super::Node>> = vec![Vec::new(); max_depth + 1];
    for n in &g.nodes {
        let col = match n.kind {
            NodeKind::Crow | NodeKind::Key => 0,
            _ => depth.get(&n.id).copied().unwrap_or(1),
        };
        columns[col].push(n);
    }
    let mut out = Layout::default();
    let mut heights = vec![0.0f32; columns.len()];
    let mut placed: Vec<Vec<(String, f32)>> = vec![Vec::new(); columns.len()];
    let mut lanes: Vec<Vec<(String, f32)>> = vec![Vec::new(); columns.len()];
    for (c, nodes) in columns.iter_mut().enumerate() {
        // Behind a jump host: next to the host (by its row), then by name.
        let row_of = |id: &str, out: &Layout| out.at.get(id).map_or(f32::MAX, |p| p.1);
        let mut keyed: Vec<(f32, String, String, &super::Node)> = nodes
            .iter()
            .map(|n| {
                // Crow heads its column, above the keys.
                let near = if n.kind == NodeKind::Crow { -1.0 } else { parent.get(n.id.as_str()).map_or(0.0, |p| row_of(p, &out)) };
                (near, n.lane.clone(), n.label.to_lowercase(), *n)
            })
            .collect();
        keyed.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)));
        let mut y = 0.0;
        let mut lane: Option<String> = None;
        for (_, l, _, n) in keyed {
            let shows_lanes = c >= 1 || n.kind == NodeKind::Key;
            if shows_lanes && lane.as_deref() != Some(l.as_str()) {
                y += LANE_GAP;
                lanes[c].push((l.clone(), y - 18.0));
                lane = Some(l);
            }
            placed[c].push((n.id.clone(), y));
            y += NODE_H + ROW_GAP;
        }
        heights[c] = (y - ROW_GAP).max(0.0);
        // Positions so far, for the next column's "next to its jump host".
        let x = c as f32 * (NODE_W + COL_GAP);
        for (id, y) in &placed[c] {
            out.at.insert(id.clone(), (x, *y));
        }
    }
    // Center every column on the tallest.
    let tallest = heights.iter().copied().fold(0.0, f32::max);
    for (c, ids) in placed.iter().enumerate() {
        let x = c as f32 * (NODE_W + COL_GAP);
        let shift = (tallest - heights[c]) / 2.0;
        for (id, y) in ids {
            out.at.insert(id.clone(), (x, y + shift));
        }
        for (text, y) in &lanes[c] {
            out.lanes.push((text.clone(), x, y + shift));
        }
    }
    out.width = columns.len() as f32 * (NODE_W + COL_GAP) - COL_GAP;
    out.height = tallest;
    out
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    fn node(id: &str, kind: NodeKind, lane: &str) -> Node {
        Node { id: id.into(), kind, label: id.into(), detail: String::new(), lane: lane.into(), health: Health::Ok, risks: Vec::new(), server_id: None }
    }

    #[test]
    fn columns_follow_hops_and_nodes_never_overlap() {
        let g = Graph {
            nodes: vec![
                node("crow", NodeKind::Crow, "CROW"),
                node("key:a", NodeKind::Key, "KEYS"),
                node("srv:bastion", NodeKind::Server, "PROD"),
                node("srv:web", NodeKind::Server, "PROD"),
                node("srv:db", NodeKind::Server, "PROD"),
                node("srv:lab", NodeKind::Vm, "LAB"),
            ],
            edges: vec![
                Edge { from: "crow".into(), to: "srv:bastion".into(), kind: EdgeKind::Ssh },
                Edge { from: "crow".into(), to: "srv:lab".into(), kind: EdgeKind::Ssh },
                Edge { from: "srv:bastion".into(), to: "srv:web".into(), kind: EdgeKind::ViaJump },
                Edge { from: "srv:bastion".into(), to: "srv:db".into(), kind: EdgeKind::ViaJump },
            ],
        };
        let l = layout(&g);
        let x = |id: &str| l.at[id].0;
        assert_eq!(x("key:a"), x("crow"), "keys sit under Crow");
        assert!(l.at["key:a"].1 > l.at["crow"].1);
        assert!(x("crow") < x("srv:bastion") && x("srv:bastion") < x("srv:web"));
        assert_eq!(x("srv:web"), x("srv:db"));
        assert_eq!(x("srv:bastion"), x("srv:lab"));
        let mut boxes: Vec<(f32, f32)> = l.at.values().copied().collect();
        boxes.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.total_cmp(&b.1)));
        for w in boxes.windows(2) {
            if w[0].0 == w[1].0 {
                assert!(w[1].1 - w[0].1 >= NODE_H, "overlap at {w:?}");
            }
        }
        assert!(l.lanes.iter().any(|(t, ..)| t == "LAB") && l.lanes.iter().any(|(t, ..)| t == "PROD"));
    }
}
