//! A terminal tab's panes (ERR-95): a binary tree of splits. GPUI-free.

pub type PaneId = u64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    /// Side by side (a vertical divider).
    Row,
    /// Stacked (a horizontal divider).
    Column,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Node {
    Pane(PaneId),
    Split { axis: Axis, first: Box<Node>, second: Box<Node> },
}

impl Node {
    /// Panes in reading order (left to right, top to bottom).
    pub fn panes(&self) -> Vec<PaneId> {
        match self {
            Node::Pane(id) => vec![*id],
            Node::Split { first, second, .. } => {
                let mut v = first.panes();
                v.extend(second.panes());
                v
            }
        }
    }

    /// Splits `target` along `axis`, with `new` after it. False if `target`
    /// isn't in this tree.
    pub fn split(&mut self, target: PaneId, axis: Axis, new: PaneId) -> bool {
        match self {
            Node::Pane(id) if *id == target => {
                *self = Node::Split { axis, first: Box::new(Node::Pane(target)), second: Box::new(Node::Pane(new)) };
                true
            }
            Node::Pane(_) => false,
            Node::Split { first, second, .. } => first.split(target, axis, new) || second.split(target, axis, new),
        }
    }

    /// Removes `target`; its sibling takes the split's place. Returns false
    /// when `target` is the only pane (the tab itself closes then).
    pub fn remove(&mut self, target: PaneId) -> bool {
        let Node::Split { first, second, .. } = self else { return false };
        if **first == Node::Pane(target) {
            *self = (**second).clone();
            return true;
        }
        if **second == Node::Pane(target) {
            *self = (**first).clone();
            return true;
        }
        first.remove(target) || second.remove(target)
    }

    /// The pane after (or before) `id` in reading order, wrapping around.
    pub fn cycle(&self, id: PaneId, forward: bool) -> PaneId {
        let panes = self.panes();
        let Some(i) = panes.iter().position(|p| *p == id) else { return panes[0] };
        let n = panes.len();
        panes[if forward { (i + 1) % n } else { (i + n - 1) % n }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_and_collapse() {
        let mut t = Node::Pane(1);
        assert!(t.split(1, Axis::Row, 2));
        assert!(t.split(2, Axis::Column, 3));
        assert_eq!(t.panes(), vec![1, 2, 3]);
        assert!(t.remove(2));
        assert_eq!(t, Node::Split { axis: Axis::Row, first: Box::new(Node::Pane(1)), second: Box::new(Node::Pane(3)) });
        assert!(t.remove(1));
        assert_eq!(t, Node::Pane(3));
        assert!(!t.remove(3), "the last pane closes the tab, not the tree");
        assert!(!t.split(9, Axis::Row, 10), "unknown pane");
    }

    #[test]
    fn cycling_wraps() {
        let mut t = Node::Pane(1);
        t.split(1, Axis::Row, 2);
        t.split(2, Axis::Row, 3);
        assert_eq!((t.cycle(3, true), t.cycle(1, false), t.cycle(2, true)), (1, 3, 3));
    }
}
