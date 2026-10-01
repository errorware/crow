//! Terminal panes (ERR-93): one per server, its session started once the
//! view knows how many cells fit, pumped while any session is alive.

use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use gpui_kit::*;

use super::CrowApp;
use crate::terminal::input::{self, Mods};
use crate::terminal::launch::launch_for;
use crate::terminal::session::{GridSize, Session};

pub struct TerminalPane {
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

impl CrowApp {
    /// The active server's pane, created on first use.
    pub fn terminal_pane(&mut self, cx: &mut Context<Self>) -> Option<&mut TerminalPane> {
        let srv = self.fleet.active_server()?;
        if !self.terminals.contains_key(&srv.id) {
            self.terminals.insert(
                srv.id.clone(),
                TerminalPane { server_id: srv.id.clone(), session: None, error: None, focus: cx.focus_handle(), target: Rc::new(Cell::new(None)), focus_pending: true },
            );
            self.ensure_terminal_pump(cx);
        }
        self.terminals.get_mut(&srv.id)
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
        for pane in self.terminals.values_mut() {
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

    /// A key pressed in the active server's terminal. Returns false for keys
    /// left to Crow (⌘ shortcuts, Ctrl+Shift chords).
    pub fn terminal_key(&mut self, ev: &KeyDownEvent, cx: &mut Context<Self>) -> bool {
        let m = &ev.keystroke.modifiers;
        let key = ev.keystroke.key.as_str();
        // Paste: ⌘V, Ctrl+Shift+V.
        if (m.platform || (m.control && m.shift)) && key.eq_ignore_ascii_case("v") {
            let text = cx.read_from_clipboard().and_then(|c| c.text());
            if let (Some(text), Some(pane)) = (text, self.terminal_pane(cx)) {
                if let Some(s) = pane.session.as_ref() {
                    s.scroll_to_bottom();
                    s.write(input::paste(&text, s.mode().contains(alacritty_terminal::term::TermMode::BRACKETED_PASTE)));
                }
            }
            return true;
        }
        if m.platform || (m.control && m.shift) {
            return false;
        }
        let Some(pane) = self.terminal_pane(cx) else { return false };
        // An ended session starts again on Enter.
        if pane.session.as_ref().is_some_and(|s| s.exited.is_some()) || pane.error.is_some() {
            if key == "enter" {
                pane.session = None;
                pane.error = None;
                cx.notify();
            }
            return true;
        }
        let Some(session) = pane.session.as_ref() else { return true };
        let app_cursor = session.mode().contains(alacritty_terminal::term::TermMode::APP_CURSOR);
        let mods = Mods { ctrl: m.control, alt: m.alt, shift: m.shift };
        if let Some(bytes) = input::encode(key, ev.keystroke.key_char.as_deref(), mods, app_cursor) {
            session.scroll_to_bottom();
            session.write(bytes);
        }
        true
    }

    /// Mouse wheel over the terminal: scrolls the history (lines, + = up).
    pub fn terminal_scroll(&mut self, lines: i32, cx: &mut Context<Self>) {
        if let Some(s) = self.terminal_pane(cx).and_then(|p| p.session.as_ref()) {
            s.scroll(lines);
            cx.notify();
        }
    }
}
