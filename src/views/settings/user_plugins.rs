//! Settings → Plugins → INSTALLED PLUGINS (ERR-177): the plugins in the
//! plugins folder, and the review that comes before accepting one. The
//! review shows what `crow --check-plugin` prints: every command, where it
//! runs, as root or not, and what it can change.

use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::app::user_plugins::{trust_label, UserPluginsState};
use crate::app::CrowApp;
use crate::plugin_host::check::command_line;
use crate::plugin_host::folder::{Installed, Trust};
use crate::plugin_host::manifest::{Action, Effect, FieldKind, Manifest, Place, Runs, Template, Var};
use crate::theme::*;

fn button(id: impl Into<ElementId>, label: &'static str, color: Rgba) -> Stateful<Div> {
    div()
        .id(id.into())
        .flex_none()
        .px(px(10.0))
        .py(px(5.0))
        .border_1()
        .border_color(color)
        .text_color(color)
        .text_size(px(10.5))
        .font_weight(FontWeight::BOLD)
        .cursor_pointer()
        .hover(|s| s.bg(BG_ROW_HOVER))
        .child(label)
}

fn trust_color(t: Trust) -> Rgba {
    match t {
        Trust::Accepted => OK,
        Trust::New | Trust::Changed => WARN,
        Trust::Refused => TEXT_DIM,
        Trust::Invalid => CRIT,
    }
}

pub fn user_plugins_section(state: &UserPluginsState, app: Entity<CrowApp>) -> impl IntoElement {
    let (app_open, app_reload) = (app.clone(), app.clone());
    div()
        .flex()
        .flex_col()
        .gap(px(8.0))
        .font_family(FONT_MONO)
        .child(
            div()
                .flex()
                .flex_wrap()
                .items_center()
                .gap(px(8.0))
                .child(div().text_size(px(10.5)).font_weight(FontWeight::BOLD).text_color(TEXT_SECONDARY).child(t!("settings-user-plugins-installed")))
                .child(div().text_size(px(9.5)).text_color(TEXT_FAINT).child(crate::plugin_host::folder::plugins_dir().display().to_string()))
                .child(div().flex_1())
                .child(button("btn-open-plugins-folder", t!("settings-user-plugins-open-folder"), TEXT_SECONDARY).on_click(move |_ev, _w, cx| app_open.update(cx, |this, cx| this.open_plugins_folder(cx))))
                .child(button("btn-reload-plugins", if state.scanning { t!("settings-user-plugins-reading") } else { t!("settings-user-plugins-reload") }, TEXT_SECONDARY).on_click(move |_ev, _w, cx| app_reload.update(cx, |this, cx| this.scan_user_plugins(cx)))),
        )
        .child(div().text_size(px(10.0)).line_height(px(15.0)).text_color(TEXT_DIM).child(t!("settings-user-plugins-about")))
        .children(state.message.clone().map(|(ok, m)| div().text_size(px(10.0)).text_color(if ok { OK } else { CRIT }).child(m)))
        .when(state.installed.is_empty(), |d| {
            d.child(div().p(px(12.0)).bg(BG_PANEL).border_1().border_color(BORDER_PANEL).text_size(px(10.5)).text_color(TEXT_DIM).child(if state.scanning { t!("settings-user-plugins-reading-folder") } else { t!("settings-user-plugins-none") }))
        })
        .children(state.installed.iter().enumerate().map(|(i, p)| {
            let open = state.reviewing.as_deref() == Some(p.id.as_str());
            let confirming = state.confirming.as_deref() == Some(p.id.as_str());
            div()
                .flex()
                .flex_col()
                .bg(BG_PANEL)
                .border_1()
                .border_color(if open { BORDER_STRONG } else { BORDER_PANEL })
                .child(row(i, p, open, app.clone()))
                .when(open, |d| {
                    let setup = (state.slot(&p.id, crate::app::user_plugins::SETUP, crate::app::user_plugins::HERE), state.setup_hashes.get(&p.id) == Some(&p.hash));
                    d.child(review(p, confirming, setup, app.clone()))
                })
        }))
}

fn row(i: usize, p: &Installed, open: bool, app: Entity<CrowApp>) -> impl IntoElement {
    let id = p.id.clone();
    let m = p.report.manifest.as_ref();
    let color = trust_color(p.trust);
    div()
        .id(("user-plugin-row", i))
        .flex()
        .items_center()
        .gap(px(10.0))
        .px(px(12.0))
        .py(px(9.0))
        .cursor_pointer()
        .hover(|s| s.bg(BG_ROW_HOVER))
        .on_click(move |_ev, _w, cx| app.update(cx, |this, cx| this.toggle_user_plugin_review(&id, cx)))
        .child(div().flex_none().size(px(7.0)).rounded_full().bg(color))
        .child(div().flex_none().text_size(px(12.0)).font_weight(FontWeight::BOLD).text_color(TEXT_PRIMARY).child(p.name().to_string()))
        .children(m.map(|m| div().flex_none().text_size(px(10.0)).text_color(TEXT_DIM).child(m.plugin.version.clone())))
        .child(div().flex_none().px(px(5.0)).py(px(1.0)).bg(BG_CHIP).text_size(px(9.0)).text_color(TEXT_TERTIARY).child(p.id.clone()))
        .children(m.map(|m| div().min_w(px(0.0)).truncate().text_size(px(10.0)).text_color(TEXT_FAINT).child(t!("settings-user-plugins-by", author = m.plugin.author))))
        .child(div().flex_1())
        .child(div().flex_none().text_size(px(10.0)).font_weight(FontWeight::BOLD).text_color(color).child(trust_label(p.trust)))
        .child(div().flex_none().text_size(px(10.0)).text_color(TEXT_SECONDARY).child(if open { t!("settings-user-plugins-close") } else if p.trust == Trust::Invalid { t!("settings-user-plugins-why") } else { t!("settings-user-plugins-review") }))
}

fn heading(text: &'static str) -> Div {
    div().pt(px(6.0)).text_size(px(9.5)).font_weight(FontWeight::BOLD).text_color(TEXT_FAINT).child(text)
}

fn code(text: String) -> Div {
    div().px(px(8.0)).py(px(5.0)).bg(BG_WINDOW).border_1().border_color(BORDER_ROW).text_size(px(10.5)).text_color(TEXT_PRIMARY).child(text)
}

fn env_rows(env: &std::collections::BTreeMap<String, Template>) -> Vec<Div> {
    env.iter()
        .map(|(k, v)| {
            let secret = v.vars().any(|v| matches!(v, Var::Secret(_)));
            div()
                .flex()
                .gap(px(8.0))
                .text_size(px(10.0))
                .child(div().text_color(TEXT_DIM).child(format!("{k}={v}")))
                .when(secret, |d| d.child(div().text_color(TEXT_FAINT).child(t!("settings-user-plugins-secret-from-vault"))))
        })
        .collect()
}

fn effect_chip(e: Effect) -> impl IntoElement {
    let (label, color) = match e {
        Effect::Read => (t!("settings-user-plugins-effect-read"), TEXT_TERTIARY),
        Effect::Change => (t!("settings-user-plugins-effect-change"), WARN),
        Effect::Danger => (t!("settings-user-plugins-effect-danger"), CRIT),
    };
    div().flex_none().w(px(64.0)).px(px(5.0)).py(px(1.0)).border_1().border_color(color).text_color(color).text_size(px(9.0)).font_weight(FontWeight::BOLD).flex().justify_center().child(label)
}

fn where_text(a: &Action) -> &'static str {
    match (a.on, a.root) {
        (Runs::Server, true) => t!("settings-user-plugins-on-each-server-root"),
        (Runs::Server, false) => t!("settings-user-plugins-on-each-server"),
        (Runs::Local, _) => t!("settings-user-plugins-on-this-machine"),
    }
}

fn runs_line(command: Option<&[Template]>, script: Option<&str>, interpreter: Option<crate::plugin_host::manifest::Interpreter>, args: &[Template]) -> String {
    match (command, script) {
        (Some(c), _) => command_line(c),
        (None, Some(s)) => {
            let prog = interpreter.map(|i| i.program()).unwrap_or("?");
            if args.is_empty() { format!("{prog} {s}") } else { format!("{prog} {s} {}", command_line(args)) }
        }
        (None, None) => String::new(),
    }
}

fn action_block(a: &Action) -> impl IntoElement {
    let mut extras: Vec<String> = Vec::new();
    if let Some(d) = &a.dry_run {
        extras.push(t!("settings-user-plugins-dry-run-first", action = d));
    }
    if a.effect == Effect::Danger {
        extras.push(t!("settings-user-plugins-typed-keyword", keyword = a.keyword.as_deref().unwrap_or("CONFIRM")));
    }
    if a.login_check {
        extras.push(t!("settings-user-plugins-login-check").to_string());
    }
    for f in &a.inputs {
        extras.push(t!("settings-user-plugins-input", key = f.key));
    }
    div()
        .flex()
        .flex_col()
        .gap(px(4.0))
        .py(px(6.0))
        .border_t_1()
        .border_color(BORDER_ROW)
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(8.0))
                .child(effect_chip(a.effect))
                .child(div().flex_none().text_size(px(11.0)).font_weight(FontWeight::BOLD).text_color(TEXT_PRIMARY).child(a.label.clone()))
                .child(div().flex_none().text_size(px(9.5)).text_color(TEXT_FAINT).child(a.id.clone()))
                .child(div().flex_1())
                .child(div().flex_none().text_size(px(10.0)).text_color(if a.root { WARN } else { TEXT_DIM }).child(where_text(a))),
        )
        .child(code(runs_line(a.command.as_deref(), a.script.as_deref(), a.interpreter, &a.args)))
        .children(env_rows(&a.env))
        .when(!extras.is_empty(), |d| d.child(div().text_size(px(10.0)).text_color(TEXT_DIM).child(extras.join(" · "))))
}

fn manifest_parts(m: &Manifest) -> Vec<AnyElement> {
    let mut out: Vec<AnyElement> = Vec::new();
    let needs: Vec<String> = [(t!("settings-user-plugins-needs-local"), &m.requires.local), (t!("settings-user-plugins-needs-server"), &m.requires.server), (t!("settings-user-plugins-needs-os"), &m.requires.os)]
        .iter()
        .filter(|(_, v)| !v.is_empty())
        .map(|(k, v)| format!("{k}: {}", v.join(", ")))
        .collect();
    if !needs.is_empty() {
        out.push(div().text_size(px(10.0)).text_color(TEXT_DIM).child(needs.join(" · ")).into_any_element());
    }
    if let Some(s) = &m.setup {
        out.push(heading(t!("settings-user-plugins-setup")).into_any_element());
        out.push(code(runs_line(s.command.as_deref(), s.script.as_deref(), s.interpreter, &s.args)).into_any_element());
        out.extend(env_rows(&s.env).into_iter().map(IntoElement::into_any_element));
    }
    out.push(heading(t!("settings-user-plugins-actions")).into_any_element());
    out.extend(m.actions.iter().map(|a| action_block(a).into_any_element()));
    if !m.settings.is_empty() {
        out.push(heading(t!("settings-user-plugins-settings")).into_any_element());
        out.push(
            div()
                .text_size(px(10.0))
                .text_color(TEXT_DIM)
                .child(m.settings.iter().map(|s| if s.kind == FieldKind::Secret { t!("settings-user-plugins-setting-secret", label = s.label) } else { s.label.clone() }).collect::<Vec<_>>().join(" · "))
                .into_any_element(),
        );
    }
    let shows: Vec<String> = m
        .pages
        .iter()
        .map(|p| if p.place == Place::Server { t!("settings-user-plugins-page-server", label = p.label) } else { t!("settings-user-plugins-page-fleet", label = p.label) })
        .chain(m.badge.iter().map(|b| t!("settings-user-plugins-badge", every = b.every.to_string())))
        .collect();
    if !shows.is_empty() {
        out.push(heading(t!("settings-user-plugins-shows")).into_any_element());
        out.push(div().text_size(px(10.0)).text_color(TEXT_DIM).child(shows.join(" · ")).into_any_element());
    }
    out
}

/// INSTALL / REINSTALL for a plugin's setup, whether it's done for these
/// files, and its console while it runs.
fn install_row(p: &Installed, setup: (Option<&crate::app::user_plugins::RunSlot>, bool), app: Entity<CrowApp>) -> impl IntoElement {
    let (slot, installed) = setup;
    let running = slot.is_some_and(|s| s.running);
    let id = p.id.clone();
    let app_console = app.clone();
    div()
        .flex()
        .flex_col()
        .gap(px(6.0))
        .child(heading(t!("settings-user-plugins-setup-heading")))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(10.0))
                .child(div().text_size(px(10.5)).text_color(if installed { OK } else { WARN }).child(if installed { t!("settings-user-plugins-installed-for-these") } else { t!("settings-user-plugins-not-installed") }))
                .child(div().flex_1())
                .child(
                    button("btn-install-plugin", if running { t!("settings-user-plugins-installing") } else if installed { t!("settings-user-plugins-reinstall") } else { t!("settings-user-plugins-install") }, if installed { TEXT_SECONDARY } else { WARN })
                        .when(!running, |d| d.on_click(move |_ev, _w, cx| app.update(cx, |this, cx| this.run_plugin_setup(&id, cx)))),
                ),
        )
        .children(slot.map(|s| crate::views::plugin_page::console(&p.id, crate::app::user_plugins::SETUP, crate::app::user_plugins::HERE, s, app_console.clone())))
}

fn review(p: &Installed, confirming: bool, state_setup: (Option<&crate::app::user_plugins::RunSlot>, bool), app: Entity<CrowApp>) -> impl IntoElement {
    let app_install = app.clone();
    let r = &p.report;
    let (app_accept, app_refuse, app_revoke, app_open) = (app.clone(), app.clone(), app.clone(), app);
    let (id_accept, id_refuse, id_revoke) = (p.id.clone(), p.id.clone(), p.id.clone());
    let files = r.files.iter().map(|f| if f.binary { t!("settings-user-plugins-file-binary", path = f.rel) } else { f.rel.clone() }).collect::<Vec<_>>().join("  ");
    let can_accept = matches!(p.trust, Trust::New | Trust::Changed | Trust::Refused);
    div()
        .flex()
        .flex_col()
        .gap(px(6.0))
        .px(px(12.0))
        .pb(px(12.0))
        .border_t_1()
        .border_color(BORDER_ROW)
        .children(r.manifest.as_ref().map(|m| {
            div()
                .pt(px(8.0))
                .flex()
                .flex_col()
                .gap(px(4.0))
                .child(div().text_size(px(10.5)).line_height(px(15.0)).text_color(TEXT_SECONDARY).child(m.plugin.about.clone()))
                .child(div().text_size(px(10.0)).text_color(TEXT_FAINT).child(t!("settings-user-plugins-meta", crow = m.plugin.crow, files = r.files.len(), license = m.plugin.license.clone().unwrap_or_else(|| "—".into()))))
        }))
        .when(p.trust == Trust::Changed, |d| d.child(div().p(px(8.0)).bg(WARN_BG).text_size(px(10.0)).text_color(WARN_INK).child(t!("settings-user-plugins-changed-since"))))
        .children(r.manifest.as_ref().map(|m| div().flex().flex_col().gap(px(4.0)).children(manifest_parts(m))))
        .when(!r.warnings.is_empty(), |d| {
            d.child(heading(t!("settings-user-plugins-warnings"))).children(r.warnings.iter().map(|w| div().text_size(px(10.0)).text_color(WARN).child(w.clone())))
        })
        .when(!r.refusals.is_empty(), |d| {
            d.child(heading(t!("settings-user-plugins-refused-heading"))).children(r.refusals.iter().map(|w| div().text_size(px(10.0)).text_color(CRIT).child(w.clone())))
        })
        .when(p.trust == Trust::Accepted && r.manifest.as_ref().is_some_and(|m| m.setup.is_some()), |d| d.child(install_row(p, state_setup, app_install)))
        .child(heading(t!("settings-user-plugins-files")))
        .child(div().text_size(px(10.0)).text_color(TEXT_DIM).child(files))
        .child(div().text_size(px(9.5)).text_color(TEXT_FAINT).child(p.hash.clone()))
        .child(
            div()
                .pt(px(6.0))
                .flex()
                .flex_wrap()
                .items_center()
                .gap(px(8.0))
                .when(can_accept, |d| {
                    d.child(
                        button("btn-accept-plugin", if confirming { t!("settings-user-plugins-confirm-accept") } else { t!("settings-user-plugins-accept") }, if confirming { CRIT } else { OK })
                            .on_click(move |_ev, _w, cx| app_accept.update(cx, |this, cx| this.accept_user_plugin(&id_accept, cx))),
                    )
                })
                .when(matches!(p.trust, Trust::New | Trust::Changed), |d| {
                    d.child(button("btn-refuse-plugin", t!("settings-user-plugins-refuse"), TEXT_SECONDARY).on_click(move |_ev, _w, cx| app_refuse.update(cx, |this, cx| this.refuse_user_plugin(&id_refuse, cx))))
                })
                .when(p.trust == Trust::Accepted, |d| {
                    d.child(button("btn-revoke-plugin", t!("settings-user-plugins-revoke"), TEXT_SECONDARY).on_click(move |_ev, _w, cx| app_revoke.update(cx, |this, cx| this.revoke_user_plugin(&id_revoke, cx))))
                })
                .child(button("btn-plugin-folder", t!("settings-user-plugins-open-folder"), TEXT_DIM).on_click(move |_ev, _w, cx| app_open.update(cx, |this, cx| this.open_plugins_folder(cx)))),
        )
        .when(confirming, |d| {
            d.child(div().text_size(px(10.0)).text_color(CRIT).child(t!("settings-user-plugins-confirm-explain")))
                .children(crate::components::english_beside::english_beside(t_beside!("settings-user-plugins-confirm-explain")))
        })
}
