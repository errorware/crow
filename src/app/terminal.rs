//! Terminal workspaces (ERR-93, ERR-95): per server, tabs of split panes,
//! each pane its own session. Sessions live while Crow runs, whatever is
//! on screen, and end when the vault locks.

use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use gpui_kit::*;

use super::CrowApp;
use crate::terminal::input::{self, Mods};
use crate::terminal::launch::launch_for;
use crate::terminal::layout::{Axis, Node, PaneId};
use crate::terminal::session::{GridSize, Session};

pub struct TerminalPane {
    pub id: PaneId,
    pub server_id: String,
    pub session: Option<Session>,
    /// Why there's no session (the server can't take one), if so.
    pub error: Option<String>,
    pub focus: FocusHandle,
    /// The grid the view last measured; the pump resizes the session to it.
    pub target: Rc<Cell<Option<GridSize>>>,
    /// Focus the pane on the next render.
    pub focus_pending: bool,
}

pub struct TerminalTab {
    pub root: Node,
    pub focused: PaneId,
    /// A name the user gave the tab; `None` follows the shell's title.
    pub name: Option<String>,
}

/// A tab being renamed in place.
pub struct TabRename {
    server_id: String,
    pub tab: usize,
    pub input: Entity<gpui_kit::component::input::InputState>,
    _events: Subscription,
}

/// A server's terminals.
pub struct TerminalWorkspace {
    pub tabs: Vec<TerminalTab>,
    pub active: usize,
    pub panes: HashMap<PaneId, TerminalPane>,
}

impl TerminalWorkspace {
    pub fn focused_pane(&self) -> Option<&TerminalPane> {
        self.panes.get(&self.tabs.get(self.active)?.focused)
    }

    fn focused_pane_mut(&mut self) -> Option<&mut TerminalPane> {
        let id = self.tabs.get(self.active)?.focused;
        self.panes.get_mut(&id)
    }

    /// A tab's label: the name it was given, else the shell's title for its
    /// focused pane, else "shell N".
    pub fn tab_title(&self, i: usize) -> String {
        let tab = &self.tabs[i];
        if let Some(name) = &tab.name {
            return name.clone();
        }
        self.panes
            .get(&tab.focused)
            .and_then(|p| p.session.as_ref())
            .and_then(|s| s.title.clone())
            .filter(|t| !t.trim().is_empty())
            .unwrap_or_else(|| format!("shell {}", i + 1))
    }

    /// Whether a pane in tab `i` rang the bell since it was last focused.
    pub fn tab_rang(&self, i: usize) -> bool {
        self.tabs[i].root.panes().iter().any(|p| self.panes.get(p).and_then(|p| p.session.as_ref()).is_some_and(|s| s.bell))
    }
}

fn next_pane_id() -> PaneId {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    NEXT.fetch_add(1, Ordering::Relaxed)
}

impl CrowApp {
    fn new_pane(server_id: &str, cx: &mut Context<Self>) -> TerminalPane {
        TerminalPane { id: next_pane_id(), server_id: server_id.into(), session: None, error: None, focus: cx.focus_handle(), target: Rc::new(Cell::new(None)), focus_pending: true }
    }

    /// The active server's workspace, created with one tab on first use.
    pub fn terminal_workspace(&mut self, cx: &mut Context<Self>) -> Option<&mut TerminalWorkspace> {
        let srv = self.fleet.active_server()?;
        if !self.terminals.contains_key(&srv.id) {
            let pane = Self::new_pane(&srv.id, cx);
            let tab = TerminalTab { root: Node::Pane(pane.id), focused: pane.id, name: None };
            self.terminals.insert(srv.id.clone(), TerminalWorkspace { tabs: vec![tab], active: 0, panes: HashMap::from([(pane.id, pane)]) });
            self.ensure_terminal_pump(cx);
        }
        self.terminals.get_mut(&srv.id)
    }

    pub fn terminal_new_tab(&mut self, cx: &mut Context<Self>) {
        let Some(srv) = self.fleet.active_server() else { return };
        let pane = Self::new_pane(&srv.id, cx);
        if let Some(ws) = self.terminal_workspace(cx) {
            ws.tabs.push(TerminalTab { root: Node::Pane(pane.id), focused: pane.id, name: None });
            ws.active = ws.tabs.len() - 1;
            ws.panes.insert(pane.id, pane);
        }
        cx.notify();
    }

    /// Splits the focused pane: `Axis::Row` puts the new pane to its right,
    /// `Axis::Column` below it.
    pub fn terminal_split(&mut self, axis: Axis, cx: &mut Context<Self>) {
        let Some(srv) = self.fleet.active_server() else { return };
        let pane = Self::new_pane(&srv.id, cx);
        if let Some(ws) = self.terminal_workspace(cx) {
            let Some(tab) = ws.tabs.get_mut(ws.active) else { return };
            if tab.root.split(tab.focused, axis, pane.id) {
                tab.focused = pane.id;
                ws.panes.insert(pane.id, pane);
            }
        }
        cx.notify();
    }

    /// Closes a pane (its session ends); the last pane of a tab closes the
    /// tab. `None` closes the focused pane.
    pub fn terminal_close_pane(&mut self, pane: Option<PaneId>, cx: &mut Context<Self>) {
        // Tab indices may shift; a rename in progress doesn't survive that.
        self.terminal_rename = None;
        let Some(ws) = self.terminal_workspace(cx) else { return };
        let Some(tab) = ws.tabs.get_mut(ws.active) else { return };
        let id = pane.unwrap_or(tab.focused);
        if tab.root.remove(id) {
            if tab.focused == id {
                tab.focused = tab.root.panes()[0];
            }
            let f = tab.focused;
            if let Some(p) = ws.panes.get_mut(&f) {
                p.focus_pending = true;
            }
        } else {
            ws.tabs.remove(ws.active);
            ws.active = ws.active.min(ws.tabs.len().saturating_sub(1));
            if let Some(p) = ws.focused_pane_mut() {
                p.focus_pending = true;
            }
        }
        ws.panes.remove(&id);
        cx.notify();
    }

    /// Edits tab `i`'s name in place (double-click). Enter or clicking away
    /// keeps it; Escape cancels; an empty name goes back to the shell's title.
    pub fn start_tab_rename(&mut self, i: usize, window: &mut Window, cx: &mut Context<Self>) {
        use gpui_kit::component::input::{InputEvent, InputState};
        let Some(current) = self.terminal_workspace(cx).filter(|ws| i < ws.tabs.len()).map(|ws| ws.tab_title(i)) else { return };
        let input = cx.new(|cx| InputState::new(window, cx).default_value(current).placeholder("tab name"));
        input.update(cx, |inp, cx| inp.focus(window, cx));
        let events = cx.subscribe(&input, |this, _input, ev: &InputEvent, cx| {
            if matches!(ev, InputEvent::PressEnter { .. } | InputEvent::Blur) {
                this.finish_tab_rename(true, cx);
            }
        });
        let server_id = self.fleet.active_server().map(|s| s.id).unwrap_or_default();
        self.terminal_rename = Some(TabRename { server_id, tab: i, input, _events: events });
        cx.notify();
    }

    pub fn finish_tab_rename(&mut self, keep: bool, cx: &mut Context<Self>) {
        let Some(r) = self.terminal_rename.take() else { return };
        let text = r.input.read(cx).value().trim().to_string();
        let same_server = self.fleet.active_server().is_some_and(|s| s.id == r.server_id);
        let keep = keep && same_server;
        if let Some(ws) = self.terminal_workspace(cx) {
            if keep {
                if let Some(tab) = ws.tabs.get_mut(r.tab) {
                    tab.name = (!text.is_empty()).then_some(text);
                }
            }
            if let Some(p) = ws.focused_pane_mut() {
                p.focus_pending = true;
            }
        }
        cx.notify();
    }

    pub fn terminal_select_tab(&mut self, i: usize, cx: &mut Context<Self>) {
        if let Some(ws) = self.terminal_workspace(cx) {
            if i < ws.tabs.len() {
                ws.active = i;
                if let Some(p) = ws.focused_pane_mut() {
                    p.focus_pending = true;
                }
            }
        }
        cx.notify();
    }

    pub fn terminal_focus_pane(&mut self, id: PaneId, cx: &mut Context<Self>) {
        if let Some(ws) = self.terminal_workspace(cx) {
            if let Some(tab) = ws.tabs.get_mut(ws.active).filter(|t| t.root.panes().contains(&id)) {
                tab.focused = id;
            }
            if let Some(p) = ws.panes.get_mut(&id) {
                p.focus_pending = true;
                if let Some(s) = p.session.as_mut() {
                    s.bell = false;
                }
            }
        }
        cx.notify();
    }

    fn terminal_cycle(&mut self, forward: bool, cx: &mut Context<Self>) {
        let next = self.terminal_workspace(cx).and_then(|ws| ws.tabs.get(ws.active)).map(|t| t.root.cycle(t.focused, forward));
        if let Some(id) = next {
            self.terminal_focus_pane(id, cx);
        }
    }

    /// Ends every session (the vault locked).
    pub fn close_all_terminals(&mut self) {
        self.terminals.clear();
    }

    fn ensure_terminal_pump(&mut self, cx: &mut Context<Self>) {
        if self.terminal_pump.is_some() {
            return;
        }
        self.terminal_pump = Some(cx.spawn(async move |entity, cx| loop {
            cx.background_executor().timer(Duration::from_millis(16)).await;
            let alive = entity.update(cx, |this, cx| this.pump_terminals(cx));
            if !matches!(alive, Ok(true)) {
                break;
            }
        }));
    }

    /// Starts sessions whose size is known, applies resizes, and redraws when
    /// output arrived. Returns whether there's anything left to pump.
    fn pump_terminals(&mut self, cx: &mut Context<Self>) -> bool {
        let mut dirty = false;
        let servers = self.fleet.servers.clone();
        for pane in self.terminals.values_mut().flat_map(|ws| ws.panes.values_mut()) {
            let Some(size) = pane.target.get() else { continue };
            match pane.session.as_mut() {
                Some(s) => {
                    s.resize(size);
                    dirty |= s.pump();
                }
                None if pane.error.is_none() => {
                    let Some(srv) = servers.iter().find(|s| s.id == pane.server_id) else { continue };
                    match launch_for(srv).and_then(|l| Session::spawn(&l, size).map_err(|e| format!("couldn't start a terminal: {e}"))) {
                        Ok(s) => pane.session = Some(s),
                        Err(e) => pane.error = Some(e),
                    }
                    dirty = true;
                }
                None => {}
            }
        }
        if dirty {
            cx.notify();
        }
        if self.terminals.is_empty() {
            self.terminal_pump = None;
            return false;
        }
        true
    }

    /// A key pressed in the focused pane. Returns false for keys left to
    /// Crow (other ⌘ shortcuts and Ctrl+Shift chords).
    pub fn terminal_key(&mut self, ev: &KeyDownEvent, cx: &mut Context<Self>) -> bool {
        let m = &ev.keystroke.modifiers;
        let key = ev.keystroke.key.to_lowercase();
        // Workspace chords: ⌘ on macOS, Ctrl+Shift elsewhere.
        let chord = if cfg!(target_os = "macos") { m.platform } else { m.control && m.shift };
        if chord {
            match (key.as_str(), m.shift) {
                ("t", _) => self.terminal_new_tab(cx),
                ("w", _) => self.terminal_close_pane(None, cx),
                ("d", true) if cfg!(target_os = "macos") => self.terminal_split(Axis::Column, cx),
                ("d", _) => self.terminal_split(Axis::Row, cx),
                ("s", _) => self.terminal_split(Axis::Column, cx),
                ("right" | "]", _) => self.terminal_cycle(true, cx),
                ("left" | "[", _) => self.terminal_cycle(false, cx),
                ("v", _) => self.terminal_paste(cx),
                _ => return false,
            }
            return true;
        }
        if m.platform {
            return false;
        }
        let Some(pane) = self.terminal_workspace(cx).and_then(|ws| ws.focused_pane_mut()) else { return false };
        // An ended session starts again on Enter.
        if pane.session.as_ref().is_some_and(|s| s.exited.is_some()) || pane.error.is_some() {
            if key == "enter" {
                pane.session = None;
                pane.error = None;
                cx.notify();
            }
            return true;
        }
        let Some(session) = pane.session.as_mut() else { return true };
        session.bell = false;
        let app_cursor = session.mode().contains(alacritty_terminal::term::TermMode::APP_CURSOR);
        let mods = Mods { ctrl: m.control, alt: m.alt, shift: m.shift };
        if let Some(bytes) = input::encode(&key, ev.keystroke.key_char.as_deref(), mods, app_cursor) {
            session.scroll_to_bottom();
            session.write(bytes);
        }
        true
    }

    fn terminal_paste(&mut self, cx: &mut Context<Self>) {
        let text = cx.read_from_clipboard().and_then(|c| c.text());
        if let (Some(text), Some(s)) = (text, self.terminal_workspace(cx).and_then(|ws| ws.focused_pane()).and_then(|p| p.session.as_ref())) {
            s.scroll_to_bottom();
            s.write(input::paste(&text, s.mode().contains(alacritty_terminal::term::TermMode::BRACKETED_PASTE)));
        }
    }

    /// Mouse wheel over a pane: scrolls its history (lines, + = up).
    pub fn terminal_scroll(&mut self, pane: PaneId, lines: i32, cx: &mut Context<Self>) {
        if let Some(s) = self.terminal_workspace(cx).and_then(|ws| ws.panes.get(&pane)).and_then(|p| p.session.as_ref()) {
            s.scroll(lines);
            cx.notify();
        }
    }
}
