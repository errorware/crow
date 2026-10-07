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
    /// Entry keys, most recent first.
    pub recent: Vec<String>,
}

/// How many results the palette lists.
pub const MAX_RESULTS: usize = 12;

impl CrowApp {
    /// Everything the palette can find, built from what Crow has in memory.
    pub fn palette_entries(&self) -> Vec<Entry> {
        let mut out = Vec::new();
        let current = self.fleet.servers.iter().find(|s| s.id == self.fleet.active_tab_id);
        // The current server's pages first: the likeliest jump.
        if let Some(cur) = current {
            for item in nav_items() {
                if let Some(view) = item.view_id {
                    out.push(Entry { category: "PAGE", label: item.label.to_string(), hint: cur.name.clone(), keywords: view.into(), target: Target::Page(view) });
                }
            }
            out.push(Entry { category: "PAGE", label: "Danger Zone".into(), hint: cur.name.clone(), keywords: "danger reboot snapshot".into(), target: Target::Page("danger") });
            for f in &self.configs.files {
                out.push(Entry { category: "CONFIG", label: f.name.clone(), hint: f.full_path.display().to_string(), keywords: f.schema_pack.unwrap_or_default().to_string(), target: Target::ConfigFile(f.name.clone()) });
            }
        }
        for s in &self.fleet.servers {
            let addr = if s.login_user.is_empty() { format!("{}:{}", s.host, s.port) } else { format!("{}@{}:{}", s.login_user, s.host, s.port) };
            let keywords = format!("{} {} {}", s.env, s.group_name, s.tags.join(" "));
            out.push(Entry { category: "SERVER", label: s.name.clone(), hint: addr.clone(), keywords: keywords.clone(), target: Target::Server { id: s.id.clone(), view: "overview" } });
            out.push(Entry { category: "TERMINAL", label: format!("Terminal on {}", s.name), hint: addr, keywords, target: Target::Server { id: s.id.clone(), view: "terminal" } });
        }
        let place = |label: &str, keywords: &str, p: Place| Entry { category: "PLACE", label: label.into(), hint: String::new(), keywords: keywords.into(), target: Target::Screen(p) };
        out.push(place("Fleet", "dashboard servers alerts", Place::Fleet));
        out.push(place("Fleet Map", "topology graph risks posture", Place::FleetMap));
        out.push(place("Fleet Setup & Policies", "groups tags host key policy", Place::FleetSetup));
        out.push(place("Audit Log", "changes history who", Place::Audit));
        out.push(place("Settings", "preferences", Place::Settings));
        out.push(place("SSH Keys", "keys credentials", Place::Keys));
        let action = |label: &str, keywords: &str, a: Action| Entry { category: "ACTION", label: label.into(), hint: String::new(), keywords: keywords.into(), target: Target::Action(a) };
        out.push(action("Add a server", "enroll onboard new ssh", Action::AddServer));
        out.push(action("Local Lab & VMs", "multipass vm launch container podman", Action::LocalLab));
        out.push(action("Check posture now", "security sshd firewall ports scan", Action::CheckPosture));
        if current.is_some() {
            out.push(action("Reconnect to this server", "ssh connection", Action::Reconnect));
        }
        out.push(action("Lock the vault", "lock secure", Action::LockVault));
        out.push(action("About Crow", "version update", Action::About));
        for k in &self.keys.enrolled {
            out.push(Entry { category: "KEY", label: k.name.clone(), hint: k.fingerprint.clone(), keywords: format!("{} {}", k.algorithm, k.comment.clone().unwrap_or_default()), target: Target::Key(k.name.clone()) });
        }
        out
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
        let input = cx.new(|cx| InputState::new(window, cx).placeholder("Search servers, pages, config files, actions, keys…"));
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
            Target::Screen(Place::Audit) => self.set_screen(Screen::Audit, cx),
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
        }
        cx.notify();
    }
}
