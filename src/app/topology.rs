use std::collections::HashMap;

use gpui_kit::*;

use super::CrowApp;
use crate::topology::{self, posture, Graph, Health};
use crate::views::fleet::state::{FleetHealth, SetupPage};
use crate::views::topology::state::zoom_around;

// ==========================================
// Fleet map (ERR-120)
// ==========================================

impl CrowApp {
    /// The fleet as Crow knows it now, as a graph.
    pub fn topology_graph(&self) -> Graph {
        let health: HashMap<String, Health> = self
            .fleet
            .servers
            .iter()
            .map(|s| {
                let h = match self.fleet.health(s) {
                    FleetHealth::Ok => Health::Ok,
                    FleetHealth::Down { .. } => Health::Down,
                    FleetHealth::Checking => Health::Unknown,
                };
                (s.id.clone(), h)
            })
            .collect();
        topology::build(&topology::Inputs {
            servers: &self.fleet.servers,
            keys: &self.keys.enrolled,
            alerts: &self.topology.alerts,
            health: &health,
            posture: &self.topology.posture,
            containers: &self.containers.by_server,
            accepted: &self.hardening.accepted,
            now: chrono::Utc::now().timestamp(),
        })
    }

    /// Fleet Setup & Policies, on its FLEET MAP tab.
    pub fn open_topology(&mut self, cx: &mut Context<Self>) {
        self.set_screen(super::Screen::FleetSetup, cx);
        self.fleet.setup_page = SetupPage::Map;
        self.reload_topology_alerts();
        if chrono::Utc::now().timestamp() - self.topology.posture_checked_at >= posture::POSTURE_EVERY_SECS {
            self.check_posture(cx);
        }
        cx.notify();
    }

    pub(crate) fn reload_topology_alerts(&mut self) {
        if let Ok(db) = self.vault.db().lock() {
            self.topology.alerts = db.list_alerts(i64::MAX).unwrap_or_default().into_iter().filter(|a| a.resolved_at.is_none()).collect();
        }
    }

    /// Reads every reachable server's posture in the background.
    pub fn check_posture(&mut self, cx: &mut Context<Self>) {
        if self.topology.scanning {
            return;
        }
        let servers: Vec<_> = self.fleet.servers.iter().filter(|s| self.fleet.health(s) == FleetHealth::Ok).cloned().collect();
        self.topology.scanning = true;
        cx.notify();
        cx.spawn(async move |entity, cx| {
            let found = cx
                .background_executor()
                .spawn(async move {
                    let now = chrono::Utc::now().timestamp();
                    servers.iter().map(|s| (s.id.clone(), posture::read_posture(crate::host::host_for(s).as_ref(), now))).collect::<Vec<_>>()
                })
                .await;
            let _ = entity.update(cx, |this, cx| {
                for (id, p) in found {
                    // Couldn't read: keep what was known.
                    if let Some(p) = p {
                        this.topology.posture.insert(id, p);
                    }
                }
                this.topology.posture_checked_at = chrono::Utc::now().timestamp();
                this.topology.scanning = false;
                this.reload_topology_alerts();
                cx.notify();
            });
        })
        .detach();
    }

    pub fn set_setup_page(&mut self, page: SetupPage, cx: &mut Context<Self>) {
        if page == SetupPage::Map {
            return self.open_topology(cx);
        }
        if page == SetupPage::Patching {
            self.scan_fleet_updates(false, cx);
        }
        if page == SetupPage::Certificates {
            self.refresh_certificates(false, cx);
        }
        if page == SetupPage::Hardening {
            if chrono::Utc::now().timestamp() - self.topology.posture_checked_at >= posture::POSTURE_EVERY_SECS {
                self.check_posture(cx);
            }
        }
        if page == SetupPage::Drift {
            self.refresh_drift();
            self.check_drift(false, cx);
        }
        self.fleet.setup_page = page;
        cx.notify();
    }

    pub fn topology_select(&mut self, id: Option<String>, cx: &mut Context<Self>) {
        self.topology.selected = id;
        cx.notify();
    }

    pub fn topology_drag_start(&mut self, at: (f32, f32), cx: &mut Context<Self>) {
        self.topology.drag = Some((at, self.topology.pan));
        cx.notify();
    }

    pub fn topology_drag_to(&mut self, at: (f32, f32), cx: &mut Context<Self>) {
        if let Some((start, pan)) = self.topology.drag {
            self.topology.pan = (pan.0 + at.0 - start.0, pan.1 + at.1 - start.1);
            cx.notify();
        }
    }

    /// Ends a drag; one that didn't move was a click on empty map, which
    /// clears the selection.
    pub fn topology_drag_end(&mut self, at: Option<(f32, f32)>, cx: &mut Context<Self>) {
        if let Some((start, _)) = self.topology.drag.take() {
            if at.is_some_and(|a| (a.0 - start.0).abs() + (a.1 - start.1).abs() < 4.0) {
                self.topology.selected = None;
            }
            cx.notify();
        }
    }

    pub fn topology_zoom(&mut self, factor: f32, at: (f32, f32), cx: &mut Context<Self>) {
        let (pan, zoom) = zoom_around(self.topology.pan, self.topology.zoom, factor, at);
        (self.topology.pan, self.topology.zoom) = (pan, zoom);
        cx.notify();
    }

    pub fn topology_reset_view(&mut self, cx: &mut Context<Self>) {
        (self.topology.pan, self.topology.zoom) = ((40.0, 40.0), 1.0);
        cx.notify();
    }

    pub fn topology_set_lane(&mut self, lane: Option<String>, cx: &mut Context<Self>) {
        self.topology.lane = lane;
        cx.notify();
    }

    pub fn topology_toggle_risky(&mut self, cx: &mut Context<Self>) {
        self.topology.risky_only = !self.topology.risky_only;
        cx.notify();
    }

    /// The selected server's own page (or its terminal).
    pub fn topology_open_server(&mut self, server_id: &str, view: &str, cx: &mut Context<Self>) {
        self.switch_tab(server_id, cx);
        self.set_view(view, cx);
    }
}
