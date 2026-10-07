use crate::lab::multipass::{Instance, LaunchSpec, Lifecycle, MultipassStatus};
use crate::lab::{EngineStatus, LocalTestNode};

/// Local test-VM lab: detected container engines, discovered nodes and the
/// "new node" modal.
pub struct LocalLabState {
    pub engines: Vec<EngineStatus>,
    pub nodes: Vec<LocalTestNode>,
    pub show_modal: bool,
    pub new_node_distro: String,
    /// Multipass (ERR-119): `None` until it's been checked.
    pub multipass: Option<MultipassStatus>,
    pub vms: Vec<Instance>,
    /// The next VM's size and image; its name is picked when it's launched.
    pub launch: LaunchSpec,
    /// What's running now (a launch, an import, a lifecycle action).
    pub busy: Option<String>,
    pub vm_error: Option<String>,
    /// A VM whose delete-for-good waits for a second click.
    pub confirm_purge: Option<String>,
}

/// One running Multipass job at a time: what it is, for the progress line.
pub fn busy_label(action: Lifecycle, name: &str) -> String {
    match action {
        Lifecycle::Start => format!("Starting {name}…"),
        Lifecycle::Stop => format!("Stopping {name}…"),
        Lifecycle::Restart => format!("Restarting {name}…"),
        Lifecycle::Suspend => format!("Suspending {name}…"),
        Lifecycle::Purge => format!("Deleting {name} and its disk…"),
    }
}

const BIRDS: [&str; 16] = ["raven", "rook", "jay", "magpie", "jackdaw", "finch", "wren", "heron", "kestrel", "swift", "robin", "lark", "owl", "tern", "plover", "dunlin"];

/// A friendly VM name not taken by `taken`: `crow-<bird>-<n>`.
pub fn friendly_vm_name(seed: u64, taken: &[String]) -> String {
    (0..1000u64)
        .map(|i| {
            let n = seed.wrapping_add(i.wrapping_mul(7919));
            format!("crow-{}-{}", BIRDS[(n % BIRDS.len() as u64) as usize], n % 90 + 10)
        })
        .find(|name| !taken.iter().any(|t| t == name))
        .unwrap_or_else(|| format!("crow-vm-{seed}"))
}

impl LocalLabState {
    pub fn new(engines: Vec<EngineStatus>, nodes: Vec<LocalTestNode>) -> Self {
        Self {
            engines,
            nodes,
            show_modal: false,
            new_node_distro: "noble".to_string(),
            multipass: None,
            vms: Vec::new(),
            launch: LaunchSpec { name: friendly_vm_name(rand::random(), &[]), ..LaunchSpec::default() },
            busy: None,
            vm_error: None,
            confirm_purge: None,
        }
    }

    pub fn find_mut(&mut self, node_id: &str) -> Option<&mut LocalTestNode> {
        self.nodes.iter_mut().find(|n| n.id == node_id || n.name == node_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lab::multipass::check_name;

    #[test]
    fn friendly_names_are_valid_and_free() {
        let first = friendly_vm_name(42, &[]);
        assert!(check_name(&first).is_ok(), "{first}");
        let second = friendly_vm_name(42, std::slice::from_ref(&first));
        assert_ne!(first, second);
        assert!(check_name(&second).is_ok());
    }
}
