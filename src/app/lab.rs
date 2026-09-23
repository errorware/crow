use gpui_kit::*;

use super::{CrowApp, Screen};
use crate::components::titlebar::ServerTab;
use crate::lab::{detect_local_engines, enroll_local_node_into_db, scan_local_test_nodes, start_local_node, stop_local_node, LocalLabEngine, LocalTestNode};
use crate::metrics::collector::{sample_server, CollectorPreviousState};
use crate::theme::OK;
use crate::vault::ServerRecord;
use crate::views::overview::collector::{collect_processes_for_server, collect_services_for_server, collect_sockets_for_server};

// ==========================================
// Local Lab & Test VMs
// ==========================================

impl CrowApp {
    pub fn toggle_local_lab_modal(&mut self, cx: &mut Context<Self>) {
        self.local_lab.show_modal = !self.local_lab.show_modal;
        if self.local_lab.show_modal {
            self.local_lab.engines = detect_local_engines();
            self.local_lab.nodes = scan_local_test_nodes(&self.servers);
        }
        cx.notify();
    }

    pub fn refresh_lab_nodes(&mut self, cx: &mut Context<Self>) {
        self.local_lab.engines = detect_local_engines();
        self.local_lab.nodes = scan_local_test_nodes(&self.servers);
        cx.notify();
    }

    pub fn set_new_lab_distro(&mut self, distro: &str, cx: &mut Context<Self>) {
        self.local_lab.new_node_distro = distro.to_string();
        cx.notify();
    }

    pub fn start_lab_node(&mut self, node_id: &str, cx: &mut Context<Self>) {
        if let Some(node) = self.local_lab.find_mut(node_id) {
            let _ = start_local_node(node);
            node.state = "running".to_string();
        }
        cx.notify();
    }

    pub fn stop_lab_node(&mut self, node_id: &str, cx: &mut Context<Self>) {
        if let Some(node) = self.local_lab.find_mut(node_id) {
            let _ = stop_local_node(node);
            node.state = "stopped".to_string();
        }
        cx.notify();
    }

    pub fn enroll_lab_node(&mut self, node_name: &str, cx: &mut Context<Self>) {
        if let Some(node) = self.local_lab.nodes.iter().find(|n| n.name == node_name).cloned() {
            self.enroll_and_open_lab_node(&node);
        }
        self.local_lab.nodes = scan_local_test_nodes(&self.servers);
        cx.notify();
    }

    pub fn create_lab_node(&mut self, cx: &mut Context<Self>) {
        let distro = self.local_lab.new_node_distro.clone();
        let name = format!("crow-lab-{}", &distro);
        let port = 2222;

        // Try starting existing local container or ensure test container is running
        let _ = std::process::Command::new("podman")
            .args(["start", "completo-node-1"])
            .output();

        let node = LocalTestNode {
            id: format!("local-{}", name),
            name: name.clone(),
            engine: LocalLabEngine::Podman,
            image: format!("docker.io/library/{}:latest", distro),
            state: "running".into(),
            ssh_port: Some(port),
            is_enrolled: false,
        };

        self.enroll_and_open_lab_node(&node);
        self.local_lab.nodes = scan_local_test_nodes(&self.servers);
        cx.notify();
    }

    /// Enrolls a lab node into the vault and switches to its overview tab.
    fn enroll_and_open_lab_node(&mut self, node: &LocalTestNode) {
        let record = match self.vault.db().lock() {
            Ok(db) => match enroll_local_node_into_db(node, &db) {
                Ok(record) => record,
                Err(_) => return,
            },
            Err(_) => return,
        };
        self.open_enrolled_server(record);
        self.local_lab.show_modal = false;
    }

    /// Adds a freshly enrolled server, seeds its metrics and collectors, and
    /// makes it the active tab on the overview.
    pub fn open_enrolled_server(&mut self, record: ServerRecord) {
        self.servers.push(record.clone());
        let mut local_prev = CollectorPreviousState::default();
        let m = sample_server(&record, None, &mut local_prev);
        self.metrics_store.insert(record.id.clone(), m.clone());
        self.metrics_store.insert(record.name.clone(), m);
        if !self.tabs.iter().any(|t| t.id == record.id) {
            self.tabs.push(ServerTab {
                id: record.id.clone(),
                name: record.name.clone(),
                status_color: OK,
                is_active: true,
            });
        }
        self.active_tab_id = record.id.clone();
        self.services = collect_services_for_server(&record);
        self.processes = collect_processes_for_server(&record);
        self.sockets = collect_sockets_for_server(&record);
        self.screen = Screen::Server;
        self.active_view = "overview".to_string();
    }
}
