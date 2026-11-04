//! Application state and actions for the Outbound Wire & Egress Traceability
//! monitor (`ERR-216`..`ERR-217`).

use std::sync::Arc;

use gpui_kit::component::input::{InputEvent, InputState};
use gpui_kit::*;

use super::{CrowApp, Screen};
use crate::wire::proc_net::AttributedSocket;
use crate::wire::{WireCategory, WireEvent, WireStatus};

pub const FLAG_WIRE_SILENCED: &str = "wire.silenced";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WireViewMode {
    #[default]
    Ledger,
    KernelSockets,
}

#[derive(Default)]
pub struct WireState {
    pub mode: WireViewMode,
    pub category_filter: Option<WireCategory>,
    pub destination_filter: Option<String>,
    pub status_filter: Option<WireStatus>,
    pub query: String,
    pub selected_seq: Option<u64>,
    pub live_sockets: Vec<AttributedSocket>,
    pub last_socket_scan_ms: i64,
    pub export_message: Option<(bool, String)>,
}

pub struct WireInputs {
    pub search: Entity<InputState>,
    pub _events: Vec<Subscription>,
}

impl CrowApp {
    /// Connects the in-memory `crate::wire` ring buffer to the SQLite `wire_log`
    /// table, restores recent persisted events, and applies any saved Outbound
    /// Silence state.
    pub fn init_wire(&mut self) {
        if let Ok(db) = self.vault.db().lock() {
            let silenced = db.flag(FLAG_WIRE_SILENCED).as_deref() == Some("1");
            crate::wire::set_silenced(silenced);
            if let Ok(rows) = db.list_wire_events(1000) {
                crate::wire::seed_from_vault(rows);
            }
        }
        let db_arc = self.vault.db().clone();
        crate::wire::init_persistence(Arc::new(move |ev: WireEvent| {
            if let Ok(db) = db_arc.lock() {
                let _ = db.insert_wire_event(&ev);
            }
        }));
        self.wire.live_sockets = crate::wire::live_attributed_sockets();
        self.wire.last_socket_scan_ms = chrono::Utc::now().timestamp_millis();
    }

    pub fn ensure_wire_inputs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.wire_inputs.is_some() {
            return;
        }
        let search = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder(t!("wire-search-placeholder"))
                .default_value(&self.wire.query)
        });
        let sub = cx.subscribe(&search, |this, input, ev: &InputEvent, cx| {
            if matches!(ev, InputEvent::Change) {
                this.wire.query = input.read(cx).value().to_string();
                cx.notify();
            }
        });
        self.wire_inputs = Some(WireInputs { search, _events: vec![sub] });
    }

    pub fn open_wire(&mut self, cx: &mut Context<Self>) {
        self.refresh_wire_sockets_now();
        self.set_screen(Screen::Wire, cx);
    }

    pub fn refresh_wire_sockets_now(&mut self) {
        self.wire.live_sockets = crate::wire::live_attributed_sockets();
        self.wire.last_socket_scan_ms = chrono::Utc::now().timestamp_millis();
    }

    pub fn refresh_wire_sockets(&mut self, cx: &mut Context<Self>) {
        self.refresh_wire_sockets_now();
        cx.notify();
    }

    pub fn toggle_wire_silence(&mut self, cx: &mut Context<Self>) {
        let next = !crate::wire::is_silenced();
        crate::wire::set_silenced(next);
        if let Ok(db) = self.vault.db().lock() {
            let _ = db.set_flag(FLAG_WIRE_SILENCED, if next { "1" } else { "0" });
        }
        cx.notify();
    }

    pub fn set_wire_mode(&mut self, mode: WireViewMode, cx: &mut Context<Self>) {
        self.wire.mode = mode;
        if mode == WireViewMode::KernelSockets {
            self.refresh_wire_sockets_now();
        }
        cx.notify();
    }

    pub fn set_wire_category(&mut self, cat: Option<WireCategory>, cx: &mut Context<Self>) {
        self.wire.category_filter = cat;
        cx.notify();
    }

    pub fn set_wire_destination(&mut self, dest: Option<String>, cx: &mut Context<Self>) {
        self.wire.destination_filter = dest;
        cx.notify();
    }

    pub fn select_wire_event(&mut self, seq: Option<u64>, cx: &mut Context<Self>) {
        self.wire.selected_seq = seq;
        cx.notify();
    }

    pub fn clear_wire_history(&mut self, cx: &mut Context<Self>) {
        crate::wire::clear_events();
        if let Ok(db) = self.vault.db().lock() {
            let _ = db.clear_wire_events();
        }
        self.wire.selected_seq = None;
        self.wire.export_message = Some((true, t!("wire-history-cleared").into()));
        cx.notify();
    }

    pub fn export_wire_jsonl(&mut self, cx: &mut Context<Self>) {
        let events = crate::wire::recent_events();
        let jsonl = crate::wire::format_jsonl(&events);
        let dir = dirs::download_dir()
            .or_else(dirs::home_dir)
            .unwrap_or_else(std::env::temp_dir);
        let ts = chrono::Local::now().format("%Y%m%d-%H%M%S");
        let path = dir.join(format!("crow-wire-audit-{ts}.jsonl"));
        match std::fs::write(&path, jsonl.as_bytes()) {
            Ok(()) => {
                self.wire.export_message = Some((
                    true,
                    t!("wire-exported-to", count = events.len(), path = path.display()),
                ));
            }
            Err(e) => {
                self.wire.export_message = Some((
                    false,
                    t!("wire-export-failed", e = e),
                ));
            }
        }
        cx.notify();
    }
}
