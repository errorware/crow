use gpui_kit::Context;

use super::CrowApp;
use crate::host::{host_for, DEFAULT_TIMEOUT};
use crate::vault::{ChangeRecord, ServerRecord};

// ==========================================
// Changes applied to a server (firewall, users, ...)
// ==========================================

/// One command of a host action, run as root.
pub struct HostCommand {
    pub argv: Vec<String>,
    pub stdin: Vec<u8>,
}

impl HostCommand {
    pub fn new(argv: Vec<String>) -> Self {
        Self { argv, stdin: Vec::new() }
    }
}

/// How a command reads in toasts, change records and the journal.
pub fn describe_commands(commands: &[HostCommand]) -> String {
    commands
        .iter()
        .map(|c| c.argv.iter().map(|a| if a.contains(' ') { format!("'{a}'") } else { a.clone() }).collect::<Vec<_>>().join(" "))
        .collect::<Vec<_>>()
        .join(" && ")
}

fn run_all(server: &ServerRecord, commands: &[HostCommand]) -> Result<(), String> {
    let host = host_for(server);
    for cmd in commands {
        let argv: Vec<&str> = cmd.argv.iter().map(String::as_str).collect();
        host.exec_privileged(&argv, &cmd.stdin, DEFAULT_TIMEOUT).map_err(|e| e.to_string())?;
    }
    Ok(())
}

impl CrowApp {
    /// Runs `commands` as root on the active server, in order, stopping at the
    /// first failure; then re-reads the affected state with `refresh` and hands
    /// the outcome to `apply` on the UI thread. Every action leaves a durable
    /// change record and a journal marker, like service restarts do.
    pub fn run_host_action<R: Send + 'static>(
        &mut self,
        kind: &str,
        target: &str,
        commands: Vec<HostCommand>,
        refresh: impl FnOnce(&ServerRecord) -> R + Send + 'static,
        apply: impl FnOnce(&mut CrowApp, R, Result<(), String>, &str, &mut Context<CrowApp>) + 'static,
        cx: &mut Context<Self>,
    ) {
        let Some(srv) = self.fleet.active_server() else { return };
        let summary = describe_commands(&commands);
        let record_id = format!("chg_{}", chrono::Local::now().timestamp_micros());
        let db = self.vault.db();
        if let Ok(db_guard) = db.lock() {
            let _ = db_guard.insert_change_record(&ChangeRecord {
                id: record_id.clone(),
                server_id: srv.id.clone(),
                server_name: srv.name.clone(),
                action_kind: kind.to_string(),
                target: target.to_string(),
                before_state: summary.clone(),
                after_state: None,
                blast_radius: None,
                outcome: "pending".to_string(),
                started_at: chrono::Utc::now().to_rfc3339(),
                completed_at: None,
            });
        }
        cx.spawn(async move |entity, cx| {
            let (result, refreshed) = cx
                .background_executor()
                .spawn(async move {
                    let result = run_all(&srv, &commands);
                    (result, refresh(&srv))
                })
                .await;
            let outcome = if result.is_ok() { "success" } else { "failed" };
            if let Ok(db_guard) = db.lock() {
                let after = result.as_ref().err().cloned();
                let _ = db_guard.update_change_record_outcome(&record_id, outcome, after.as_deref(), &chrono::Utc::now().to_rfc3339());
            }
            let _ = entity.update(cx, |this, cx| {
                this.push_journal_action_marker(format!("crow: {summary} ({outcome})"));
                apply(this, refreshed, result, &summary, cx);
                cx.notify();
            });
        })
        .detach();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commands_are_described_readably() {
        let cmds = vec![
            HostCommand::new(vec!["ufw".into(), "allow".into(), "22/tcp".into()]),
            HostCommand::new(vec!["ufw".into(), "comment".into(), "two words".into()]),
        ];
        assert_eq!(describe_commands(&cmds), "ufw allow 22/tcp && ufw comment 'two words'");
    }
}
