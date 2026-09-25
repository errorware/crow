use gpui_kit::Context;

use super::CrowApp;
use crate::host::{host_for, DEFAULT_TIMEOUT};
use crate::region::{parse_region_probe, REGION_PROBE};
use crate::vault::ServerRecord;

/// Which environment the Fleet list shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum FleetEnvFilter {
    #[default]
    All,
    Prod,
    Stage,
    DevLab,
}

impl FleetEnvFilter {
    pub fn matches(self, env: &str) -> bool {
        match self {
            FleetEnvFilter::All => true,
            FleetEnvFilter::Prod => env == "PROD",
            FleetEnvFilter::Stage => env == "STAGE",
            FleetEnvFilter::DevLab => env == "DEV" || env == "LAB",
        }
    }
}

/// What a detection run found, for the Fleet page's status line.
fn summarize(found: usize, unknown: usize, failed: usize) -> String {
    let mut parts = vec![format!("{found} located")];
    if unknown > 0 {
        parts.push(format!("{unknown} without cloud metadata"));
    }
    if failed > 0 {
        parts.push(format!("{failed} unreachable"));
    }
    parts.join(" · ")
}

impl CrowApp {
    pub fn set_fleet_env_filter(&mut self, filter: FleetEnvFilter, cx: &mut Context<Self>) {
        self.fleet.env_filter = filter;
        cx.notify();
    }

    /// Shows or hides the region chips row (the BY REGION tab).
    pub fn toggle_fleet_region_bar(&mut self, cx: &mut Context<Self>) {
        self.fleet.region_bar_open = !self.fleet.region_bar_open;
        if !self.fleet.region_bar_open {
            self.fleet.region_filter = None;
        }
        cx.notify();
    }

    /// `Some("DE")` shows one country, `Some("")` servers with no known
    /// region, `None` all.
    pub fn set_fleet_region_filter(&mut self, country: Option<String>, cx: &mut Context<Self>) {
        self.fleet.region_filter = country;
        cx.notify();
    }

    /// Asks each server (all of them, or `only`) where it lives, over its
    /// transport, in the background. Manual choices are never overwritten.
    pub fn detect_server_regions(&mut self, only: Option<String>, cx: &mut Context<Self>) {
        if self.fleet.region_detecting {
            return;
        }
        let targets: Vec<ServerRecord> = self
            .fleet
            .servers
            .iter()
            .filter(|s| only.as_ref().is_none_or(|id| &s.id == id))
            .filter(|s| s.region_source != "manual")
            .cloned()
            .collect();
        if targets.is_empty() {
            return;
        }
        self.fleet.region_detecting = true;
        self.fleet.region_note = Some(format!("asking {} server(s)…", targets.len()));
        cx.notify();
        let db = self.vault.db();
        cx.spawn(async move |entity, cx| {
            let note = cx
                .background_executor()
                .spawn(async move {
                    let (mut found, mut unknown, mut failed) = (0, 0, 0);
                    for mut srv in targets {
                        let out = match host_for(&srv).exec(&["sh", "-c", REGION_PROBE], DEFAULT_TIMEOUT) {
                            Ok(o) => o.stdout,
                            Err(_) => {
                                failed += 1;
                                continue;
                            }
                        };
                        match parse_region_probe(&out) {
                            Some(r) => {
                                if r.country.is_empty() { unknown += 1 } else { found += 1 }
                                srv.region_country = r.country;
                                srv.region_city = r.city;
                                srv.region_provider = r.provider;
                                srv.region_code = r.code;
                                srv.region_source = "metadata".into();
                            }
                            None => {
                                unknown += 1;
                                continue;
                            }
                        }
                        if let Ok(db) = db.lock() {
                            let _ = db.upsert_server(&srv);
                        }
                    }
                    summarize(found, unknown, failed)
                })
                .await;
            let _ = entity.update(cx, |this, cx| {
                this.fleet.region_detecting = false;
                this.fleet.region_note = Some(note);
                this.reload_servers();
                cx.notify();
            });
        })
        .detach();
    }

    /// Sets a server's country by hand (`None` clears it). Detection leaves
    /// manual choices alone.
    pub fn set_server_region(&mut self, server_id: &str, country: Option<&str>, cx: &mut Context<Self>) {
        let Some(mut srv) = self.fleet.servers.iter().find(|s| s.id == server_id).cloned() else { return };
        match country {
            Some(cc) => {
                srv.region_country = cc.to_string();
                srv.region_city = String::new();
                srv.region_code = String::new();
                srv.region_source = "manual".into();
            }
            None => {
                srv.region_country = String::new();
                srv.region_city = String::new();
                srv.region_code = String::new();
                srv.region_source = String::new();
            }
        }
        if let Ok(db) = self.vault.db().lock() {
            let _ = db.upsert_server(&srv);
        }
        self.region_picker_open = false;
        self.reload_servers();
        cx.notify();
    }

    pub fn toggle_region_picker(&mut self, cx: &mut Context<Self>) {
        self.region_picker_open = !self.region_picker_open;
        cx.notify();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_filters_and_summary() {
        assert!(FleetEnvFilter::DevLab.matches("LAB") && FleetEnvFilter::DevLab.matches("DEV"));
        assert!(!FleetEnvFilter::Prod.matches("STAGE"));
        assert_eq!(summarize(2, 1, 0), "2 located · 1 without cloud metadata");
    }
}
