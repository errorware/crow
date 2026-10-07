use gpui_kit::*;

use gpui_kit::component::input::{InputEvent, InputState};

use super::configs::RISK_CONFIRM_KEYWORD;
use super::host_actions::HostCommand;
use super::CrowApp;
use crate::host::{transport_kind, TransportKind};
use crate::views::firewall::commands::{self, Argv};
use crate::views::firewall::lockout;
use crate::views::firewall::{detect_firewall_status, FirewallBackend, FirewallOperationalState, FirewallStatusSummary, LockoutConfirm, RuleAction, RuleDirection, RuleProtocol};

// ==========================================
// Firewall & Network Security Methods
// ==========================================

/// The add-rule dialog's text fields; they live while it's open.
pub struct FirewallRuleInputs {
    pub port: Entity<InputState>,
    pub source: Entity<InputState>,
    pub comment: Entity<InputState>,
    _events: Vec<Subscription>,
}

/// One of the add-rule dialog's text fields.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuleField {
    Port,
    Source,
    Comment,
}

impl CrowApp {
    /// Creates the add-rule dialog's inputs when it opens (they need the
    /// window), and drops them when it closes. Typing updates the form;
    /// Enter adds the rule.
    pub fn ensure_firewall_rule_inputs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.firewall.show_new_rule_modal {
            self.firewall_rule_inputs = None;
            return;
        }
        if self.firewall_rule_inputs.is_some() {
            return;
        }
        let form = &self.firewall.new_rule;
        let source_now = if form.is_anywhere { String::new() } else { form.source_input.clone() };
        let make = |placeholder: &str, value: String, window: &mut Window, cx: &mut Context<Self>| cx.new(|cx| InputState::new(window, cx).placeholder(placeholder.to_string()).default_value(value));
        let port = make("8080, or a range 8000:8100", form.port_input.clone(), window, cx);
        let source = make("anywhere, or an address / CIDR", source_now, window, cx);
        let comment = make("why this rule exists", form.comment_input.clone(), window, cx);
        let events = [(&port, RuleField::Port), (&source, RuleField::Source), (&comment, RuleField::Comment)]
            .into_iter()
            .map(|(input, field)| {
                cx.subscribe(input, move |this, input, ev: &InputEvent, cx| match ev {
                    InputEvent::Change => {
                        let value = input.read(cx).value().to_string();
                        this.set_firewall_rule_text(field, value);
                        cx.notify();
                    }
                    InputEvent::PressEnter { .. } => this.submit_new_firewall_rule(cx),
                    _ => {}
                })
            })
            .collect();
        port.update(cx, |i, cx| i.focus(window, cx));
        self.firewall_rule_inputs = Some(FirewallRuleInputs { port, source, comment, _events: events });
    }

    fn set_firewall_rule_text(&mut self, field: RuleField, value: String) {
        let form = &mut self.firewall.new_rule;
        match field {
            RuleField::Port => form.port_input = value,
            RuleField::Source => {
                form.is_anywhere = value.trim().is_empty() || value.trim().eq_ignore_ascii_case("anywhere");
                form.source_input = value;
            }
            RuleField::Comment => form.comment_input = value,
        }
    }

    /// A preset in the add-rule dialog: fills the field as if typed.
    pub fn set_firewall_rule_field(&mut self, field: RuleField, value: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.set_firewall_rule_text(field, value.to_string());
        if let Some(inputs) = &self.firewall_rule_inputs {
            let input = match field {
                RuleField::Port => &inputs.port,
                RuleField::Source => &inputs.source,
                RuleField::Comment => &inputs.comment,
            };
            input.update(cx, |i, cx| i.set_value(value.to_string(), window, cx));
        }
        cx.notify();
    }

    pub fn set_firewall_action_filter(&mut self, filter: Option<RuleAction>, cx: &mut Context<Self>) {
        self.firewall.action_filter = filter;
        cx.notify();
    }

    pub fn open_new_firewall_rule_modal(&mut self, cx: &mut Context<Self>) {
        self.firewall.open_new_rule_modal();
        cx.notify();
    }

    pub fn close_new_firewall_rule_modal(&mut self, cx: &mut Context<Self>) {
        self.firewall.show_new_rule_modal = false;
        cx.notify();
    }

    /// Mirrors the in-memory rule set into the `user.rules` config file state so
    /// the pending-diff rail and versioning see firewall edits.
    pub fn sync_firewall_to_config_state(&mut self) {
        if let FirewallOperationalState::Active(ref summary) = self.firewall.status.clone().ufw_only() {
            let content = crate::views::firewall::generate_user_rules_content(&summary.rules);
            if let Some(st) = self.configs.states.get_mut("user.rules") {
                st.update_content(content);
            }
        }
    }

    pub fn toggle_firewall_audit_rail(&mut self, cx: &mut Context<Self>) {
        self.firewall.show_audit_rail = !self.firewall.show_audit_rail;
        cx.notify();
    }

    pub fn toggle_firewall_raw(&mut self, cx: &mut Context<Self>) {
        self.firewall.show_raw = !self.firewall.show_raw;
        cx.notify();
    }

    pub fn stage_firewall_rules(&mut self, message: &str, cx: &mut Context<Self>) {
        self.sync_firewall_to_config_state();
        self.stage_config_version("user.rules", message, cx);
    }

    /// Reads the active server's firewall in the background (startup with a
    /// remote first server, and every tab switch). The result is dropped if
    /// the active server changed meanwhile.
    pub fn refresh_firewall_for_active_server(&mut self, cx: &mut Context<Self>) {
        let Some(srv) = self.fleet.active_server() else { return };
        // A held-back change was for the server being left.
        self.firewall.lockout = None;
        self.firewall.status = FirewallOperationalState::Unmanaged {
            detected_binaries: Vec::new(),
            reason: format!("Reading {}'s firewall…", srv.name),
        };
        let id = srv.id.clone();
        cx.spawn(async move |entity, cx| {
            let status = cx.background_executor().spawn(async move { detect_firewall_status(&srv) }).await;
            let _ = entity.update(cx, |this, cx| {
                if this.fleet.active_server().is_some_and(|s| s.id == id) {
                    this.firewall.status = status;
                    // ufw's own rules file, as load_configs mirrors it.
                    if let FirewallOperationalState::Active(ref summary) = this.firewall.status.clone().ufw_only() {
                        let text = crate::views::firewall::generate_user_rules_content(&summary.rules);
                        this.configs.seed_baseline("user.rules", text, Some("/etc/ufw/user.rules"));
                        this.configs.block_writes("user.rules", crate::app::configs::user_rules_read_only_reason().into());
                    }
                    cx.notify();
                }
            });
        })
        .detach();
    }

    /// The rules Crow read, when it read some it can play changes against.
    fn firewall_summary(&self) -> Option<&FirewallStatusSummary> {
        match &self.firewall.status {
            FirewallOperationalState::Active(s) => Some(s),
            _ => None,
        }
    }

    /// Why `after` needs CONFIRM: it stops the SSH port Crow is connected
    /// through from being allowed (ERR-77). Only SSH servers have one.
    fn firewall_lockout_risk(&self, after: Option<&FirewallStatusSummary>) -> Option<String> {
        let srv = self.fleet.active_server()?;
        if transport_kind(&srv) != TransportKind::Ssh {
            return None;
        }
        // A stored 0 means sshd's default, as everywhere else (ERR-114).
        let port = if srv.port == 0 { 22 } else { srv.port };
        lockout::risk(self.firewall_summary(), after, port)
    }

    /// Runs firewall commands on the active server, unless the lock-out
    /// guard holds them back for CONFIRM. `after` is what the rules would be
    /// (`Some(None)`: unknown); `None` means the change can't lock anyone out.
    fn run_firewall_change(&mut self, commands: Vec<Argv>, after: Option<Option<FirewallStatusSummary>>, closes_modal: bool, cx: &mut Context<Self>) {
        if let Some(after) = after {
            if let Some(reason) = self.firewall_lockout_risk(after.as_ref()) {
                self.firewall.lockout = Some(LockoutConfirm { commands, closes_modal, reason, input: None, error: None });
                cx.notify();
                return;
            }
        }
        self.execute_firewall_commands(commands, closes_modal, cx);
    }

    /// Runs the commands, then shows the firewall's real state.
    fn execute_firewall_commands(&mut self, commands: Vec<Argv>, closes_modal: bool, cx: &mut Context<Self>) {
        let target = commands.first().and_then(|a| a.get(1..)).map(|rest| rest.join(" ")).unwrap_or_default();
        self.firewall.pending = Some(commands.iter().map(|c| commands::describe(c)).collect::<Vec<_>>().join(" && "));
        self.run_host_action(
            "firewall",
            &target,
            commands.into_iter().map(HostCommand::new).collect(),
            |srv| detect_firewall_status(srv),
            move |this, status, result, summary, cx| {
                this.firewall.status = status;
                this.firewall.pending = None;
                this.firewall.toast = Some(match &result {
                    Ok(()) => format!("Ran: {summary}"),
                    Err(e) => format!("Failed: {summary} — {e}"),
                });
                if closes_modal && result.is_ok() {
                    this.firewall.show_new_rule_modal = false;
                }
                this.sync_firewall_to_config_state();
                cx.notify();
            },
            cx,
        );
    }

    /// The typed CONFIRM for a change the guard held back.
    pub fn ensure_firewall_lockout_input(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(confirm) = self.firewall.lockout.as_mut().filter(|c| c.input.is_none()) else { return };
        let input = cx.new(|cx| InputState::new(window, cx).placeholder(RISK_CONFIRM_KEYWORD));
        input.update(cx, |i, cx| i.focus(window, cx));
        cx.subscribe(&input, |this, _, ev: &InputEvent, cx| {
            if matches!(ev, InputEvent::PressEnter { .. }) {
                this.confirm_firewall_lockout(cx);
            }
        })
        .detach();
        confirm.input = Some(input);
    }

    pub fn confirm_firewall_lockout(&mut self, cx: &mut Context<Self>) {
        let Some(pending) = self.firewall.lockout.as_mut() else { return };
        let typed = pending.input.as_ref().map(|i| i.read(cx).value().trim().to_string()).unwrap_or_default();
        if typed != RISK_CONFIRM_KEYWORD {
            pending.error = Some(format!("Type {RISK_CONFIRM_KEYWORD} exactly to apply this change."));
            cx.notify();
            return;
        }
        let pending = self.firewall.lockout.take().expect("checked above");
        self.execute_firewall_commands(pending.commands, pending.closes_modal, cx);
    }

    pub fn cancel_firewall_lockout(&mut self, cx: &mut Context<Self>) {
        self.firewall.lockout = None;
        cx.notify();
    }

    /// ufw and firewalld are the backends Crow changes.
    fn firewall_backend_or_toast(&mut self, cx: &mut Context<Self>) -> Option<FirewallBackend> {
        match self.firewall.status.backend() {
            b @ (FirewallBackend::Ufw | FirewallBackend::Firewalld) => Some(b),
            b => {
                self.firewall.toast = Some(match b {
                    FirewallBackend::NoneDetected => "Not applied: no firewall Crow can change was found on this host.".to_string(),
                    b => format!("Not applied: Crow reads {} but never writes it.", b.label()),
                });
                cx.notify();
                None
            }
        }
    }

    fn firewall_toast(&mut self, msg: String, cx: &mut Context<Self>) {
        self.firewall.toast = Some(msg);
        cx.notify();
    }

    pub fn toggle_firewall_active(&mut self, cx: &mut Context<Self>) {
        if self.firewall_backend_or_toast(cx) != Some(FirewallBackend::Ufw) {
            return;
        }
        let summary = self.firewall_summary().cloned();
        let enabled = summary.as_ref().is_some_and(|s| s.is_active);
        if enabled {
            // Turning the firewall off can't lock anyone out.
            self.run_firewall_change(vec![commands::disable()], None, false, cx);
        } else {
            // ufw's rules aren't read while it's off, so the outcome is unknown.
            self.run_firewall_change(vec![commands::enable()], Some(summary.map(|s| FirewallStatusSummary { is_active: true, ..s })), false, cx);
        }
    }

    pub fn reload_firewall(&mut self, cx: &mut Context<Self>) {
        match self.firewall_backend_or_toast(cx) {
            Some(FirewallBackend::Ufw) => self.run_firewall_change(vec![commands::reload()], None, false, cx),
            Some(_) => {
                // A reload swaps the running rules for the saved ones; when
                // they differ, Crow can't tell what SSH will get.
                let differs = self.firewall_summary().is_some_and(|s| s.notice.is_some());
                let after = if differs { Some(None) } else { None };
                self.run_firewall_change(vec![commands::firewalld_reload()], after, false, cx);
            }
            None => {}
        }
    }

    pub fn toggle_quick_port(&mut self, port: u16, proto: RuleProtocol, label: &str, cx: &mut Context<Self>) {
        let Some(backend) = self.firewall_backend_or_toast(cx) else { return };
        let summary = self.firewall_summary().cloned();
        let after = summary.as_ref().map(|s| match commands::quick_port_rule(Some(s), port) {
            Some(rule) => lockout::without(s, rule),
            None => lockout::with(s, vec![lockout::planned(RuleAction::Allow, &port.to_string(), proto, "")], false),
        });
        let cmds = if backend == FirewallBackend::Ufw {
            Ok(vec![commands::toggle_port(summary.as_ref(), port, proto, label)])
        } else {
            commands::firewalld_toggle_port(summary.as_ref(), port, proto)
        };
        match cmds {
            Ok(cmds) => self.run_firewall_change(cmds, Some(after), false, cx),
            Err(e) => self.firewall_toast(format!("Not applied: {e}"), cx),
        }
    }

    pub fn delete_firewall_rule(&mut self, rule_id: &str, cx: &mut Context<Self>) {
        let Some(backend) = self.firewall_backend_or_toast(cx) else { return };
        let Some(summary) = self.firewall_summary().cloned() else { return };
        let Some(rule) = summary.rules.iter().find(|r| r.id == rule_id).cloned() else { return };
        let after = lockout::without(&summary, &rule);
        let cmds = if backend == FirewallBackend::Ufw { Ok(vec![commands::delete_numbered(rule.number)]) } else { commands::firewalld_remove(&rule) };
        match cmds {
            Ok(cmds) => self.run_firewall_change(cmds, Some(Some(after)), false, cx),
            Err(e) => self.firewall_toast(format!("Not applied: {e}"), cx),
        }
    }

    pub fn submit_new_firewall_rule(&mut self, cx: &mut Context<Self>) {
        let Some(backend) = self.firewall_backend_or_toast(cx) else { return };
        let form = &self.firewall.new_rule;
        let cmds = if backend == FirewallBackend::Ufw { commands::new_rule(form).map(|a| vec![a]) } else { commands::firewalld_new_rule(form) };
        let source = if form.is_anywhere { String::new() } else { form.source_input.clone() };
        let planned = lockout::planned(form.action, &form.port_input.trim().replace('-', ":"), form.protocol, &source);
        let inbound = form.direction == RuleDirection::Inbound;
        let after = self.firewall_summary().map(|s| if inbound { lockout::with(s, vec![planned], backend == FirewallBackend::Firewalld) } else { s.clone() });
        match cmds {
            Ok(cmds) => self.run_firewall_change(cmds, Some(after), true, cx),
            Err(e) => self.firewall_toast(format!("Not applied: {e}"), cx),
        }
    }

    #[allow(dead_code)]
    pub fn flush_firewall_rules(&mut self, cx: &mut Context<Self>) {
        // `ufw reset` also turns ufw off, so nothing is blocked after it.
        self.run_firewall_change(vec![commands::reset()], None, false, cx);
    }
}
