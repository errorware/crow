use gpui_kit::component::input::{InputEvent, InputState};
use gpui_kit::*;

use super::{CrowApp, Screen, SettingsSection};
use crate::components::sidebar::nav_items;
use crate::palette::{rank, Action, Entry, Place, Target};

/// The command palette's live state (ERR-135).
#[derive(Default)]
pub struct PaletteState {
    /// Made when the palette opens (an input needs the window).
    pub input: Option<Entity<InputState>>,
    _events: Option<Subscription>,
    pub query: String,
    pub selected: usize,
    /// Tab on a server narrows the search to it (Backspace on an empty
    /// query widens it again).
    pub scope: Option<String>,
    /// Entry keys, most recent first.
    pub recent: Vec<String>,
    /// (server id, path) of every config file Crow has read, loaded when
    /// the palette opens (ERR-136).
    pub fleet_files: Vec<(String, String)>,
}

/// How many results the palette lists.
pub const MAX_RESULTS: usize = 12;

impl CrowApp {
    /// Everything the palette can find, built from what Crow has in memory.
    pub fn palette_entries(&self) -> Vec<Entry> {
        if let Some(scope) = &self.palette.scope {
            return self.palette_scoped_entries(scope);
        }
        let mut out = Vec::new();
        let current = self.fleet.servers.iter().find(|s| s.id == self.fleet.active_tab_id);
        // The current server's pages first: the likeliest jump.
        if current.is_some() {
            for item in nav_items().iter().filter(|i| i.view_id.is_none_or(|v| crate::components::sidebar::page_shown(v, &self.plugins.enabled))) {
                // "this server", not its name: typing a server's name should
                // find the server, not every page of the open one.
                if let Some(view) = item.view_id {
                    out.push(Entry { category: t!("app-palette-page"), label: crate::i18n::tx(item.label).to_string(), hint: t!("app-palette-this-server").into(), keywords: view.into(), target: Target::Page(view) });
                }
            }
            out.push(Entry { category: t!("app-palette-page"), label: t!("app-palette-danger-zone").into(), hint: t!("app-palette-this-server").into(), keywords: "danger reboot snapshot".into(), target: Target::Page("danger") });
            out.extend(self.current_server_entries());
        }
        for s in &self.fleet.servers {
            let addr = if s.login_user.is_empty() { format!("{}:{}", s.host, s.port) } else { format!("{}@{}:{}", s.login_user, s.host, s.port) };
            // Also found by how it's doing and where it lives.
            let health = match self.fleet.health(s) {
                crate::views::fleet::state::FleetHealth::Ok => "online",
                crate::views::fleet::state::FleetHealth::Down { .. } => t!("app-palette-offline-unreachable-down"),
                crate::views::fleet::state::FleetHealth::Checking => "",
            };
            let keywords = format!("{} {} {} {health} {} {} {}", s.env, s.group_name, s.tags.join(" "), s.provider_account, s.region_city, s.region_provider);
            out.push(Entry { category: t!("app-palette-server"), label: s.name.clone(), hint: addr.clone(), keywords: keywords.clone(), target: Target::Server { id: s.id.clone(), view: "overview" } });
            out.push(Entry { category: t!("app-palette-terminal"), label: t!("app-palette-terminal-on", name = s.name), hint: addr, keywords, target: Target::Server { id: s.id.clone(), view: "terminal" } });
        }
        let place = |label: &str, keywords: &str, p: Place| Entry { category: t!("app-palette-place"), label: label.into(), hint: String::new(), keywords: keywords.into(), target: Target::Screen(p) };
        out.push(place(t!("app-palette-fleet"), "dashboard servers alerts", Place::Fleet));
        out.push(place(t!("app-palette-fleet-map"), "topology graph risks posture", Place::FleetMap));
        out.push(place(t!("app-palette-fleet-setup-policies"), "groups tags host key policy", Place::FleetSetup));
        out.push(place(t!("app-palette-patching"), "updates upgrade apt dnf apk reboot maintenance window", Place::Patching));
        out.push(place(t!("app-palette-rollouts"), "run command config every server canary batch", Place::Rollouts));
        out.push(place(t!("app-palette-drift-config-search"), "baseline drift search setting find grep", Place::Drift));
        out.push(place(t!("app-palette-audit-log"), "changes history who", Place::Audit));
        out.push(place(t!("app-palette-settings"), "preferences", Place::Settings));
        out.push(place(t!("app-palette-ssh-keys"), "keys credentials", Place::Keys));
        // Every config file Crow has read, on every other server.
        let names: std::collections::HashMap<&str, &str> = self.fleet.servers.iter().map(|s| (s.id.as_str(), s.name.as_str())).collect();
        for (sid, path) in self.palette.fleet_files.iter().filter(|(sid, _)| current.is_none_or(|c| &c.id != sid)) {
            let Some(name) = names.get(sid.as_str()) else { continue };
            out.push(Entry { category: t!("app-palette-fleet-config"), label: path.clone(), hint: name.to_string(), keywords: name.to_string(), target: Target::FleetConfig { server_id: sid.clone(), path: path.clone() } });
        }
        let action = |label: &str, keywords: &str, a: Action| Entry { category: t!("app-palette-action"), label: label.into(), hint: String::new(), keywords: keywords.into(), target: Target::Action(a) };
        out.push(action(t!("app-palette-add-a-server"), "enroll onboard new ssh", Action::AddServer));
        if let Some(label) = self.lab_label() {
            out.push(action(label, "multipass vm launch container podman lab", Action::LocalLab));
        }
        out.push(action(t!("app-palette-check-posture-now"), "security sshd firewall ports scan", Action::CheckPosture));
        if current.is_some() {
            out.push(action(t!("app-palette-reconnect-to-this-server"), "ssh connection", Action::Reconnect));
        }
        out.push(action(t!("app-palette-lock-the-vault"), "lock secure", Action::LockVault));
        if current.is_some() && self.active_provider_actions().is_some_and(|p| p.snapshots) {
            out.push(action(t!("app-palette-take-a-snapshot"), "backup provider image danger", Action::Snapshot));
        }
        out.push(action(t!("app-palette-security-stance-2fa"), "2fa password lock open locked vault", Action::Stance));
        out.push(action(t!("app-palette-about-crow"), "version update", Action::About));
        for k in &self.keys.enrolled {
            out.push(Entry { category: t!("app-palette-key"), label: k.name.clone(), hint: k.fingerprint.clone(), keywords: format!("{} {}", k.algorithm, k.comment.clone().unwrap_or_default()), target: Target::Key(k.name.clone()) });
        }
        out
    }

    /// The open server's config files, services and processes.
    fn current_server_entries(&self) -> Vec<Entry> {
        let mut out = Vec::new();
        for f in &self.configs.files {
            out.push(Entry { category: t!("app-palette-config"), label: f.name.clone(), hint: f.full_path.display().to_string(), keywords: f.schema_pack.unwrap_or_default().to_string(), target: Target::ConfigFile(f.name.clone()) });
        }
        for svc in &self.overview.services {
            out.push(Entry { category: t!("app-palette-service"), label: svc.name.clone(), hint: svc.description.clone(), keywords: svc.status.to_lowercase(), target: Target::Service(svc.name.clone()) });
        }
        for p in self.overview.processes.iter().filter(|p| !p.is_kernel) {
            let cmd: String = p.command.chars().take(80).collect();
            out.push(Entry { category: t!("app-palette-process"), label: cmd, hint: t!("app-palette-pid", pid = p.pid, user = p.user), keywords: p.pid.to_string(), target: Target::Process(p.pid) });
        }
        out
    }

    /// Inside a server (Tab): its pages, a terminal, and, when it's the open
    /// server, its config files, services and processes.
    fn palette_scoped_entries(&self, id: &str) -> Vec<Entry> {
        let Some(srv) = self.fleet.servers.iter().find(|s| s.id == id) else { return Vec::new() };
        let mut out: Vec<Entry> = nav_items()
            .iter()
            .filter(|i| i.view_id.is_none_or(|v| crate::components::sidebar::page_shown(v, &self.plugins.enabled)))
            .filter_map(|item| item.view_id.map(|view| Entry { category: t!("app-palette-page"), label: crate::i18n::tx(item.label).to_string(), hint: srv.name.clone(), keywords: view.into(), target: Target::Server { id: srv.id.clone(), view } }))
            .collect();
        out.push(Entry { category: t!("app-palette-page"), label: t!("app-palette-danger-zone").into(), hint: srv.name.clone(), keywords: "danger reboot snapshot".into(), target: Target::Server { id: srv.id.clone(), view: "danger" } });
        if srv.id == self.fleet.active_tab_id {
            out.extend(self.current_server_entries());
        }
        out
    }

    /// Tab: narrows the search to the selected server.
    pub fn palette_tab(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let results = self.palette_results();
        let Some(Target::Server { id, .. }) = results.get(self.palette.selected).map(|e| e.target.clone()) else { return };
        self.palette.scope = Some(id);
        self.palette.selected = 0;
        self.palette.query.clear();
        if let Some(input) = self.palette.input.clone() {
            input.update(cx, |i, cx| i.set_value("", window, cx));
        }
        cx.notify();
    }

    /// Backspace on an empty query: back out of a server's scope. True when
    /// it did (the key is used up).
    pub fn palette_unscope(&mut self, cx: &mut Context<Self>) -> bool {
        if self.palette.query.is_empty() && self.palette.scope.take().is_some() {
            self.palette.selected = 0;
            cx.notify();
            return true;
        }
        false
    }

    /// The name of the server the search is narrowed to.
    pub fn palette_scope_name(&self) -> Option<String> {
        let id = self.palette.scope.as_ref()?;
        self.fleet.servers.iter().find(|s| &s.id == id).map(|s| s.name.clone())
    }

    /// The results for the current query, as the palette lists them.
    pub fn palette_results(&self) -> Vec<Entry> {
        let entries = self.palette_entries();
        rank(&entries, &self.palette.query, &self.palette.recent).into_iter().take(MAX_RESULTS).cloned().collect()
    }

    /// Makes the palette's input the first time it renders open.
    pub fn ensure_palette_input(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.palette.input.is_some() {
            return;
        }
        let input = cx.new(|cx| InputState::new(window, cx).placeholder(t!("app-palette-search-servers-pages-config-files")));
        input.update(cx, |i, cx| i.focus(window, cx));
        let events = cx.subscribe(&input, |this, input, ev: &InputEvent, cx| match ev {
            InputEvent::Change => {
                this.palette.query = input.read(cx).value().to_string();
                this.palette.selected = 0;
                cx.notify();
            }
            InputEvent::PressEnter { .. } => this.palette_run_selected(cx),
            _ => {}
        });
        self.palette.input = Some(input);
        self.palette._events = Some(events);
    }

    pub fn palette_move(&mut self, delta: i32, cx: &mut Context<Self>) {
        let n = self.palette_results().len();
        if n > 0 {
            self.palette.selected = (self.palette.selected as i32 + delta).rem_euclid(n as i32) as usize;
            cx.notify();
        }
    }

    pub fn palette_run_selected(&mut self, cx: &mut Context<Self>) {
        let results = self.palette_results();
        if let Some(e) = results.get(self.palette.selected).or(results.first()).cloned() {
            self.palette_run(e, cx);
        }
    }

    pub fn palette_run(&mut self, entry: Entry, cx: &mut Context<Self>) {
        let key = entry.key();
        self.palette.recent.retain(|k| *k != key);
        self.palette.recent.insert(0, key);
        self.palette.recent.truncate(20);
        self.close_palette(cx);
        match entry.target {
            Target::Server { id, view } => {
                self.switch_tab(&id, cx);
                self.set_view(view, cx);
            }
            Target::Page(view) => {
                self.set_screen(Screen::Server, cx);
                self.set_view(view, cx);
            }
            Target::ConfigFile(name) => {
                self.set_screen(Screen::Server, cx);
                self.set_view("config", cx);
                self.select_managed_file(&name, cx);
            }
            Target::Screen(Place::Fleet) => self.set_screen(Screen::Fleet, cx),
            Target::Screen(Place::FleetSetup) => {
                self.set_screen(Screen::FleetSetup, cx);
                self.set_setup_page(crate::views::fleet::state::SetupPage::Policies, cx);
            }
            Target::Screen(Place::FleetMap) => self.open_topology(cx),
            Target::Screen(Place::Audit) => self.open_audit(cx),
            Target::Screen(p @ (Place::Patching | Place::Rollouts | Place::Drift)) => {
                use crate::views::fleet::state::SetupPage;
                self.set_screen(Screen::FleetSetup, cx);
                self.set_setup_page(match p { Place::Patching => SetupPage::Patching, Place::Rollouts => SetupPage::Rollouts, _ => SetupPage::Drift }, cx);
            }
            Target::FleetConfig { server_id, path } => {
                let server = self.fleet.servers.iter().find(|s| s.id == server_id).map(|s| s.name.clone()).unwrap_or_default();
                self.open_search_hit(&crate::app::drift::SearchHit { server_id, server, path, line: None }, cx);
            }
            Target::Screen(Place::Settings) => self.set_screen(Screen::Settings, cx),
            Target::Screen(Place::Keys) | Target::Key(_) => {
                self.set_screen(Screen::Settings, cx);
                self.set_settings_section(SettingsSection::Keys, cx);
            }
            Target::Action(Action::AddServer) => self.start_onboarding(cx),
            Target::Action(Action::LocalLab) => {
                self.set_screen(Screen::Fleet, cx);
                if !self.local_lab.show_modal {
                    self.toggle_local_lab_modal(cx);
                }
            }
            Target::Action(Action::CheckPosture) => {
                self.open_topology(cx);
                self.check_posture(cx);
            }
            Target::Action(Action::Reconnect) => self.reconnect_active_server(cx),
            Target::Action(Action::LockVault) => self.lock(cx),
            Target::Action(Action::About) => self.open_about_modal(cx),
            Target::Action(Action::Snapshot) => {
                self.set_screen(Screen::Server, cx);
                self.set_view("danger", cx);
            }
            Target::Action(Action::Stance) => self.toggle_stance_panel(cx),
            Target::Service(name) => {
                self.set_screen(Screen::Server, cx);
                self.set_view("services", cx);
                self.focus_service(&name, cx);
            }
            Target::Process(pid) => {
                self.set_screen(Screen::Server, cx);
                self.set_view("processes", cx);
                self.focus_process(pid, cx);
            }
        }
        cx.notify();
    }
}
