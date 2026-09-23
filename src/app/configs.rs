use gpui_kit::*;

use super::CrowApp;
use crow_config_core::edit::ConfigDocument;
use crow_config_core::ConfigPlugin;
use crow_config_schemas::PgHbaPlugin;
use crate::config::{crawl_machine_configs, load_config_file_state};
use crate::journal::retention::JournalRetentionConfig;
use crate::theme::{CRIT, OK};
use crate::views::config::cron_editor::default_cron_jobs;
use crate::views::firewall::FirewallOperationalState;
use crate::theme::WARN;

// ==========================================
// Config files: discovery, staging, versions
// ==========================================

/// Startup smoke check that crow-config-core and its pg_hba schema plugin
/// parse a representative file; logs the parsed rule count.
pub fn log_config_core_self_check() {
    let plugin = PgHbaPlugin::new();
    let sample_pg_hba = r#"
# PostgreSQL Client Authentication Configuration File
local   all             postgres                                peer
host    all             all             127.0.0.1/32            scram-sha-256
host    all             all             ::1/128                 scram-sha-256
host    acme_prod       acme_app        10.0.4.19/32            scram-sha-256
host    all             all             0.0.0.0/0               md5
host    all             all             10.0.4.0/24             scram-sha-256
"#;
    if let Ok(doc) = ConfigDocument::parse(&plugin, sample_pg_hba) {
        if let Ok(ir) = doc.to_ir() {
            println!(
                "Crow Core: parsed {} pg_hba rules from schema plugin: {}",
                ir.rows.len(),
                plugin.manifest().plugin.name
            );
        }
    }
}

impl CrowApp {
    pub fn default_author(&self) -> String {
        "Nelson <nelson@errorware.net>".to_string()
    }

    pub fn toggle_config_history(&mut self, cx: &mut Context<Self>) {
        self.configs.show_history = !self.configs.show_history;
        cx.notify();
    }

    pub fn select_managed_file(&mut self, filename: &str, cx: &mut Context<Self>) {
        self.configs.selected_file = filename.to_string();
        cx.notify();
    }

    pub fn crawl_system_configs(&mut self, cx: &mut Context<Self>) {
        let discovered = crawl_machine_configs(self.fleet.local_distro_family);
        for f in &discovered {
            if !self.configs.states.contains_key(&f.name) {
                self.configs.states.insert(f.name.clone(), load_config_file_state(f));
            }
        }
        self.configs.files = discovered;
        cx.notify();
    }

    #[allow(dead_code)]
    pub fn set_config_search_query(&mut self, query: String, cx: &mut Context<Self>) {
        self.configs.search_query = query;
        cx.notify();
    }

    pub fn revert_managed_config(&mut self, file: &str, cx: &mut Context<Self>) {
        if file == "pg_hba.conf" {
            for r in &mut self.configs.hba_rules {
                if r.num == "09" || r.num == "10" {
                    r.method = "trust";
                    r.risk = "CRITICAL";
                    r.risk_color = CRIT;
                    r.is_expanded = false;
                }
            }
        } else if file == "journald.conf" {
            self.journal.retention = JournalRetentionConfig::default();
        } else if file == "crontab" {
            self.configs.cron_jobs = default_cron_jobs();
        } else if file == "user.rules" {
            if let Some(state) = self.configs.states.get(file) {
                let rules = crate::views::firewall::parse_user_rules_content(&state.baseline_content);
                if let FirewallOperationalState::Active(ref mut summary) = self.firewall.status {
                    summary.rules = rules;
                }
            }
            self.firewall.toast = Some("Reverted firewall rules to baseline".to_string());
        }
        if let Some(state) = self.configs.states.get_mut(file) {
            state.revert();
            cx.notify();
        }
    }

    pub fn stage_config_version(&mut self, file: &str, description: &str, cx: &mut Context<Self>) {
        let author = self.default_author();
        if let Some(state) = self.configs.states.get_mut(file) {
            state.stage_revision(author, description.to_string());
            let _ = state.save_to_disk();
            if file == "pg_hba.conf" {
                for r in &mut self.configs.hba_rules {
                    if r.risk == "EDITED" {
                        r.risk = "OK";
                        r.risk_color = OK;
                    }
                }
            } else if file == "journald.conf" {
                self.journal.show_retention_modal = false;
            } else if file == "user.rules" {
                self.firewall.toast = Some(format!("Audit commit created: {}", description));
            }
            cx.notify();
        }
    }

    pub fn rollback_config_revision(&mut self, file: &str, version: usize, cx: &mut Context<Self>) {
        if let Some(state) = self.configs.states.get_mut(file) {
            if state.rollback_to_revision(version) {
                let _ = state.save_to_disk();
                if file == "user.rules" {
                    let rules = crate::views::firewall::parse_user_rules_content(&state.current_content);
                    if let FirewallOperationalState::Active(ref mut summary) = self.firewall.status {
                        summary.rules = rules;
                    }
                    self.firewall.toast = Some(format!("Rolled back firewall to revision v{}", version));
                }
                cx.notify();
            }
        }
    }

    pub fn apply_journal_boundaries(&mut self, cx: &mut Context<Self>) {
        self.stage_config_version("journald.conf", "Applied journald retention boundaries", cx);
    }

    pub fn toggle_rule_expand(&mut self, num: &str, cx: &mut Context<Self>) {
        for r in &mut self.configs.hba_rules {
            if r.num == num {
                r.is_expanded = !r.is_expanded;
            }
        }
        cx.notify();
    }

    pub fn set_rule_method(&mut self, rule_num: &str, method: &'static str, cx: &mut Context<Self>) {
        for r in &mut self.configs.hba_rules {
            if r.num == rule_num {
                r.method = method;
                r.risk = if method == "scram-sha-256" || method == "cert" {
                    "OK"
                } else if method == "trust" {
                    "CRITICAL"
                } else {
                    "REVIEW"
                };
                r.risk_color = if r.risk == "OK" {
                    OK
                } else if r.risk == "CRITICAL" {
                    CRIT
                } else {
                    WARN
                };
            }
        }
        self.configs.sync_hba();
        cx.notify();
    }

    // --- SSH Key Management Subsystem ---
}
