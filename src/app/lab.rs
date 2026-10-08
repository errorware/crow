use gpui_kit::*;

use super::{CrowApp, Screen};
use crate::components::titlebar::ServerTab;
use crate::lab::multipass::{self, Lifecycle};
use crate::lab::{
    enroll_local_node_into_db,
    scan_local_test_nodes,
    start_local_node,
    stop_local_node,
    LocalTestNode,
};
use crate::metrics::collector::{sample_server, CollectorStates};
use crate::vault::ServerRecord;
use crate::views::overview::collector::{
    collect_processes_for_server,
    collect_services_for_server,
    collect_sockets_for_server,
};

// ==========================================
// Local Lab & Test VMs
// ==========================================

impl CrowApp {
    pub fn toggle_local_lab_modal(&mut self, cx: &mut Context<Self>) {
        self.local_lab.show_modal = !self.local_lab.show_modal;
        if self.local_lab.show_modal {
            self.refresh_lab(cx);
        }
        cx.notify();
    }

    /// Re-reads what the enabled lab plugins have, and only those (ERR-138).
    fn refresh_lab(&mut self, cx: &mut Context<Self>) {
        for id in crate::plugins::CONTAINER_PLUGINS.into_iter().chain(["multipass"]) {
            if self.plugin_enabled(id) {
                self.check_plugin(id, cx);
            }
        }
        self.local_lab.nodes = if self.lab_enabled() { scan_local_test_nodes(&self.fleet.servers) } else { Vec::new() };
        if self.plugin_enabled("multipass") {
            self.refresh_multipass(cx);
        }
    }

    // ==========================================
    // Multipass VMs (ERR-119)
    // ==========================================

    /// Re-checks Multipass and lists its VMs, off the UI thread.
    pub fn refresh_multipass(&mut self, cx: &mut Context<Self>) {
        cx.spawn(async move |entity, cx| {
            let (status, vms) = cx
                .background_executor()
                .spawn(async move {
                    let status = multipass::detect();
                    let vms = if status.is_ready() { multipass::list() } else { Ok(Vec::new()) };
                    (status, vms)
                })
                .await;
            let _ = entity.update(cx, |this, cx| {
                this.local_lab.multipass = Some(status);
                match vms {
                    Ok(vms) => this.local_lab.vms = vms,
                    Err(e) => this.local_lab.vm_error = Some(e),
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub fn set_vm_size(&mut self, cpus: u8, memory_gb: u16, disk_gb: u16, cx: &mut Context<Self>) {
        let l = &mut self.local_lab.launch;
        (l.cpus, l.memory_gb, l.disk_gb) = (cpus.clamp(1, 16), memory_gb.clamp(1, 64), disk_gb.clamp(5, 500));
        cx.notify();
    }

    pub fn set_vm_image(&mut self, image: &str, cx: &mut Context<Self>) {
        self.local_lab.launch.image = image.to_string();
        cx.notify();
    }

    pub fn reroll_vm_name(&mut self, cx: &mut Context<Self>) {
        let taken: Vec<String> = self.local_lab.vms.iter().map(|v| v.name.clone()).collect();
        self.local_lab.launch.name = crate::views::fleet::lab_state::friendly_vm_name(rand::random(), &taken);
        cx.notify();
    }

    /// Runs `job` (blocking) in the background with Crow's key, then saves
    /// the server it returns into the fleet and opens it. The lab closes
    /// right away; `busy` (shown in the status bar) follows the job's
    /// progress, so it's clear it's working.
    fn run_vm_enrollment(
        &mut self,
        label: String,
        job: impl FnOnce(&crate::vault::SshKeyRecord, &(dyn Fn(String) + Sync)) -> Result<ServerRecord, String> + Send + 'static,
        cx: &mut Context<Self>,
    ) {
        if self.local_lab.busy.is_some() {
            return;
        }
        let key = match self.ensure_crow_key(cx) {
            Ok(k) => k,
            Err(e) => {
                self.local_lab.vm_error = Some(format!("Crow's key: {e}"));
                cx.notify();
                return;
            }
        };
        self.local_lab.busy = Some(label.clone());
        self.local_lab.vm_error = None;
        self.local_lab.show_modal = false;
        cx.notify();
        let progress = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
        let done = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        // Copies the job's latest step into the status bar while it runs.
        let (ticker_progress, ticker_done) = (progress.clone(), done.clone());
        cx.spawn(async move |entity, cx| {
            while !ticker_done.load(std::sync::atomic::Ordering::Relaxed) {
                cx.background_executor().timer(std::time::Duration::from_millis(400)).await;
                let step = ticker_progress.lock().map(|p| p.clone()).unwrap_or_default();
                let alive = entity.update(cx, |this, cx| {
                    if this.local_lab.busy.is_some() && !step.is_empty() {
                        this.local_lab.busy = Some(format!("{label} · {step}"));
                        cx.notify();
                    }
                });
                if alive.is_err() {
                    break;
                }
            }
        })
        .detach();
        cx.spawn(async move |entity, cx| {
            let job_progress = progress.clone();
            let result = cx
                .background_executor()
                .spawn(async move {
                    let report = move |line: String| {
                        if let Ok(mut p) = job_progress.lock() {
                            *p = line;
                        }
                    };
                    job(&key, &report)
                })
                .await;
            done.store(true, std::sync::atomic::Ordering::Relaxed);
            let _ = entity.update(cx, |this, cx| {
                this.local_lab.busy = None;
                match result {
                    Ok(record) => this.save_vm_server(record, cx),
                    Err(e) => {
                        this.keys.toast = Some(format!("Multipass: {e}"));
                        this.local_lab.vm_error = Some(e);
                    }
                }
                this.refresh_multipass(cx);
                cx.notify();
            });
        })
        .detach();
    }

    fn save_vm_server(&mut self, record: ServerRecord, cx: &mut Context<Self>) {
        if let Ok(db) = self.vault.db().lock() {
            if let Err(e) = db.upsert_server(&record) {
                self.local_lab.vm_error = Some(format!("couldn't save {} in the vault: {e}", record.name));
                return;
            }
            if let Some(kid) = &record.key_id {
                let _ = db.attach_server_to_key(kid, &record.name);
            }
        }
        self.refresh_keys(cx);
        self.keys.toast = Some(format!("{} is in the fleet ({})", record.name, record.host));
        self.fleet.servers.retain(|s| s.id != record.id);
        self.local_lab.show_modal = false;
        self.open_enrolled_server(record);
    }

    /// Launches a VM with Crow's key in its cloud-init and enrolls it.
    pub fn launch_multipass_vm(&mut self, cx: &mut Context<Self>) {
        let spec = self.local_lab.launch.clone();
        if let Err(e) = spec.check() {
            self.local_lab.vm_error = Some(e);
            cx.notify();
            return;
        }
        let label = format!("Launching {}", spec.name);
        self.run_vm_enrollment(label, move |key, progress| multipass::launch_and_enroll(&spec, key, progress), cx);
        self.reroll_vm_name(cx);
    }

    /// Adds Crow's key to a VM Crow didn't make and enrolls it.
    pub fn import_multipass_vm(&mut self, name: &str, cx: &mut Context<Self>) {
        let name = name.to_string();
        self.run_vm_enrollment(format!("Importing {name}"), move |key, progress| multipass::import(&name, key, progress), cx);
    }

    /// Start, stop, restart, suspend; delete for good only after
    /// `confirm_purge` names this VM (the second click).
    pub fn run_vm_lifecycle(&mut self, action: Lifecycle, name: &str, cx: &mut Context<Self>) {
        if self.local_lab.busy.is_some() {
            return;
        }
        if action == Lifecycle::Purge && self.local_lab.confirm_purge.as_deref() != Some(name) {
            self.local_lab.confirm_purge = Some(name.to_string());
            cx.notify();
            return;
        }
        self.local_lab.confirm_purge = None;
        self.local_lab.busy = Some(crate::views::fleet::lab_state::busy_label(action, name));
        self.local_lab.vm_error = None;
        cx.notify();
        let vm = name.to_string();
        cx.spawn(async move |entity, cx| {
            let job_vm = vm.clone();
            let result = cx.background_executor().spawn(async move { multipass::run(action, &job_vm) }).await;
            let _ = entity.update(cx, |this, cx| {
                this.local_lab.busy = None;
                match result {
                    Ok(()) if action == Lifecycle::Purge => {
                        // The VM is gone: its fleet entry goes to the archive.
                        if let Some(id) = this.fleet.servers.iter().find(|s| multipass::vm_of(s) == Some(vm.as_str())).map(|s| s.id.clone()) {
                            this.archive_server(&id, cx);
                        }
                        this.keys.toast = Some(format!("{vm} deleted"));
                    }
                    Ok(()) => this.keys.toast = Some(format!("{vm}: {} done", action.label().to_lowercase())),
                    Err(e) => {
                        let msg = format!("{} {vm}: {e}", action.label());
                        this.keys.toast = Some(msg.clone());
                        this.local_lab.vm_error = Some(msg);
                    }
                }
                this.refresh_multipass(cx);
                cx.notify();
            });
        })
        .detach();
    }

    pub fn cancel_vm_purge(&mut self, cx: &mut Context<Self>) {
        self.local_lab.confirm_purge = None;
        cx.notify();
    }

    pub fn refresh_lab_nodes(&mut self, cx: &mut Context<Self>) {
        self.refresh_lab(cx);
        self.reload_configs_for_active_server(cx);
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
        self.local_lab.nodes = scan_local_test_nodes(&self.fleet.servers);
        self.reload_configs_for_active_server(cx);
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
        self.fleet.servers.push(record.clone());
        self.sync_ssh_directory();
        let mut local_prev = CollectorStates::default();
        let m = sample_server(&record, None, &mut local_prev);
        self.fleet.metrics_store.insert(record.id.clone(), m.clone());
        self.fleet.metrics_store.insert(record.name.clone(), m);
        if !self.fleet.tabs.iter().any(|t| t.id == record.id) {
            self.fleet.tabs.push(ServerTab {
                id: record.id.clone(),
                name: record.name.clone(),
                is_active: true,
            });
        }
        self.fleet.active_tab_id = record.id.clone();
        self.overview.services = collect_services_for_server(&record);
        self.overview.processes = collect_processes_for_server(&record);
        self.overview.sockets = collect_sockets_for_server(&record);
        self.screen = Screen::Server;
        self.active_view = "overview".to_string();
    }
}
