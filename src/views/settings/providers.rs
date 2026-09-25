//! Settings → Providers: one card per compiled-in provider plugin, with its
//! category, capabilities, connection state and a settings form rendered
//! from its manifest.

use gpui_kit::component::input::Input;
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crow_config_core::FieldType;
use crow_provider_core::ProviderFactory;

use crate::app::providers::{ProviderFormInputs, ProvidersState};
use crate::app::{CrowApp, SettingsSection};
use crate::components::icons::{inherited_icon, TablerIcon};
use crate::theme::*;

pub fn render_providers_view(app: Entity<CrowApp>, state: &ProvidersState, inputs: Option<&ProviderFormInputs>, secrets_blocker: Option<String>, secrets_notice: Option<&str>, open_stance: bool) -> impl IntoElement {
    let factories = crate::providers::factories();
    let configured = factories.iter().filter(|f| state.account(&(f.manifest)().plugin.name).is_some()).count();

    div()
        .flex_1()
        .min_w(px(0.0))
        .flex()
        .flex_col()
        .bg(BG_APP)
        .child(
            div()
                .h(px(40.0))
                .flex_none()
                .flex()
                .items_center()
                .gap(px(10.0))
                .px(px(14.0))
                .bg(BG_PANEL)
                .border_b_1()
                .border_color(BORDER_PANEL)
                .child(div().font_family(FONT_MONO).text_size(px(12.0)).font_weight(FontWeight::BOLD).text_color(TEXT_PRIMARY).child("PROVIDERS · WHERE YOUR SERVERS LIVE"))
                .child(
                    div()
                        .px(px(6.0))
                        .py(px(2.0))
                        .border_1()
                        .border_color(BORDER_DEFAULT)
                        .font_family(FONT_MONO)
                        .text_size(px(9.5))
                        .text_color(TEXT_SECONDARY)
                        .child(format!("{configured}/{} SET UP", factories.len())),
                ),
        )
        .child(
            div()
                .id("providers-scroll")
                .flex_1()
                .overflow_y_scrollbar()
                .p(px(14.0))
                .flex()
                .flex_col()
                .gap(px(14.0))
                .child(intro())
                .children(secrets_notice.map(|n| notice(n.to_string(), TEXT_SECONDARY, None, Some(app.clone()))))
                .children(secrets_blocker.map(|why| notice(format!("Tokens can't be saved right now: {why}."), WARN, Some(app.clone()), None)))
                .children(factories.into_iter().map(|f| card(f, state, inputs, open_stance, app.clone()))),
        )
}

fn intro() -> impl IntoElement {
    div()
        .p(px(12.0))
        .bg(BG_OVERLAY_PANEL)
        .border_1()
        .border_color(BORDER_PANEL)
        .flex()
        .gap(px(10.0))
        .child(div().text_color(TEXT_PRIMARY).child(inherited_icon(TablerIcon::Cloud, px(16.0))))
        .child(
            div()
                .flex_1()
                .min_w(px(0.0))
                .font_family(FONT_MONO)
                .text_size(px(10.5))
                .line_height(px(15.0))
                .text_color(TEXT_MUTED)
                .child("Connect the cloud accounts your servers run on. Crow reads your instances from them and, where the provider supports it, can power servers and take snapshots from outside SSH. Tokens are stored encrypted in Crow's vault: its key is held by your OS keyring (Keychain, Secret Service) or, with a vault password, locked by that password. They reach the provider through curl's stdin, never the command line."),
        )
}

/// A one-line notice; with `link`, it links to Vault & Security, with
/// `dismiss`, it has a ✕ that clears the secrets notice.
pub(crate) fn notice(text: String, color: Rgba, link: Option<Entity<CrowApp>>, dismiss: Option<Entity<CrowApp>>) -> impl IntoElement {
    div()
        .p(px(10.0))
        .border_1()
        .border_color(color.opacity(0.5))
        .bg(color.opacity(0.06))
        .flex()
        .items_center()
        .gap(px(10.0))
        .font_family(FONT_MONO)
        .text_size(px(10.5))
        .child(div().flex_1().min_w(px(0.0)).text_color(color).child(text))
        .children(dismiss.map(|app| {
            div()
                .id("secrets-notice-dismiss")
                .cursor_pointer()
                .text_color(TEXT_DIMMER)
                .hover(|s| s.text_color(TEXT_PRIMARY))
                .on_click(move |_ev, _window, cx| app.update(cx, |this, cx| this.dismiss_secrets_notice(cx)))
                .child("✕")
        }))
        .children(link.map(|app| {
            div()
                .id("secrets-vault-link")
                .cursor_pointer()
                .text_color(TEXT_PRIMARY)
                .hover(|s| s.underline())
                .on_click(move |_ev, _window, cx| app.update(cx, |this, cx| this.set_settings_section(SettingsSection::Security, cx)))
                .child("Vault & Security →")
        }))
}

fn card(factory: ProviderFactory, state: &ProvidersState, inputs: Option<&ProviderFormInputs>, open_stance: bool, app: Entity<CrowApp>) -> impl IntoElement {
    let manifest = (factory.manifest)();
    let plugin = manifest.plugin.name.clone();
    let account = state.account(&plugin);
    let editing = state.editing.as_deref() == Some(plugin.as_str());
    let checking = account.is_some_and(|a| state.checking.contains(&a.id));

    let status: (String, Rgba) = match account {
        None => ("not set up".into(), TEXT_FAINT),
        Some(_) if checking => ("testing…".into(), TEXT_SECONDARY),
        Some(a) => match (&a.last_check, a.last_check_ok) {
            (Some(msg), true) => (format!("connected · {msg}"), OK),
            (Some(msg), false) => (msg.clone(), CRIT),
            (None, _) => ("saved · not tested yet".into(), TEXT_SECONDARY),
        },
    };

    let header = div()
        .flex()
        .items_center()
        .gap(px(10.0))
        .child(div().font_family(FONT_MONO).text_size(px(12.5)).font_weight(FontWeight::BOLD).text_color(TEXT_PRIMARY).child(manifest.display_name().to_string()))
        .child(tag(manifest.plugin.category.clone().unwrap_or_default(), TEXT_DIMMER))
        .children(manifest.plugin.capabilities.iter().map(|c| tag(c.clone(), TEXT_SECONDARY)))
        .child(div().flex_1())
        .children(account.and_then(|a| a.last_check_at.as_deref()).map(|t| div().font_family(FONT_MONO).text_size(px(9.0)).text_color(TEXT_FAINTER).child(format!("checked {}", short_time(t)))));

    let status_line = div().font_family(FONT_MONO).text_size(px(10.5)).text_color(status.1).child(status.0);

    let (a_edit, a_test, a_remove, p_edit, p_test, p_remove) = (app.clone(), app.clone(), app.clone(), plugin.clone(), plugin.clone(), plugin.clone());
    let confirming = state.confirm_remove.as_deref() == Some(plugin.as_str());
    let actions = div()
        .flex()
        .gap(px(6.0))
        .when(!editing, |d| {
            d.child(button(format!("provider-edit-{plugin}"), if account.is_some() { "EDIT" } else { "SET UP" }.into(), OK, true, move |cx| {
                let p = p_edit.clone();
                a_edit.update(cx, |this, cx| this.open_provider_form(&p, cx));
            }))
        })
        .when(account.is_some(), |d| {
            d.child(button(format!("provider-test-{plugin}"), "TEST CONNECTION".into(), TEXT_SECONDARY, !checking, move |cx| {
                let p = p_test.clone();
                a_test.update(cx, |this, cx| this.test_provider(&p, cx));
            }))
            .child(button(format!("provider-remove-{plugin}"), if confirming { "CLICK AGAIN TO REMOVE".into() } else { "REMOVE".into() }, CRIT, true, move |cx| {
                let p = p_remove.clone();
                a_remove.update(cx, |this, cx| this.remove_provider_clicked(&p, cx));
            }))
        });

    div()
        .p(px(12.0))
        .bg(BG_PANEL)
        .border_1()
        .border_color(if editing { BORDER_DEFAULT } else { BORDER_PANEL })
        .flex()
        .flex_col()
        .gap(px(8.0))
        .child(header)
        .child(status_line)
        .children((open_stance && can_act(manifest)).then(|| {
            div()
                .font_family(FONT_MONO)
                .text_size(px(10.0))
                .text_color(WARN)
                .child("Crow is OPEN: anyone at this session can use this token to power servers off or snapshot them. See the stance badge.")
        }))
        .children(state.errors.get(&plugin).map(|e| div().font_family(FONT_MONO).text_size(px(10.5)).text_color(CRIT).child(format!("Not saved: {e}"))))
        .children(editing.then(|| form(&factory, state, inputs, app.clone())))
        .child(actions)
}

fn form(factory: &ProviderFactory, state: &ProvidersState, inputs: Option<&ProviderFormInputs>, app: Entity<CrowApp>) -> impl IntoElement {
    let manifest = (factory.manifest)();
    let plugin = manifest.plugin.name.clone();
    let account_id = state.account(&plugin).map(|a| a.id.clone());
    let inputs = inputs.filter(|i| i.plugin == plugin);
    let (a_save, a_cancel) = (app.clone(), app.clone());

    div()
        .mt(px(4.0))
        .pt(px(10.0))
        .border_t_1()
        .border_color(BORDER_ROW)
        .flex()
        .flex_col()
        .gap(px(10.0))
        .children(manifest.fields.iter().map(|def| {
            let input = inputs.and_then(|i| i.fields.iter().find(|(k, _, _)| k == &def.name)).map(|(_, _, input)| input.clone());
            let secret = def.field_type == FieldType::Secret;
            let set = secret && account_id.as_ref().is_some_and(|a| state.is_secret_set(a, &def.name));
            let clearable = set && def.required != Some(true);
            let (app, plugin, key) = (app.clone(), plugin.clone(), def.name.clone());
            div()
                .flex()
                .gap(px(14.0))
                .child(
                    div()
                        .w(px(220.0))
                        .flex_none()
                        .flex()
                        .flex_col()
                        .gap(px(2.0))
                        .font_family(FONT_MONO)
                        .child(
                            div()
                                .flex()
                                .gap(px(6.0))
                                .text_size(px(11.0))
                                .text_color(TEXT_PRIMARY)
                                .child(def.label.clone().unwrap_or_else(|| def.name.clone()))
                                .children((def.required == Some(true)).then(|| div().text_color(WARN).child("*")))
                                .children(secret.then(|| div().text_size(px(9.0)).text_color(TEXT_FAINT).child("secret · vault"))),
                        )
                        .children(def.help.clone().map(|h| div().text_size(px(9.5)).line_height(px(13.0)).text_color(TEXT_FAINT).child(h))),
                )
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .children(input.map(|i| div().flex_1().max_w(px(420.0)).child(Input::new(&i).font_family(FONT_MONO).text_size(px(11.0)).bg(BG_APP).rounded(px(2.0)))))
                        .children(clearable.then(|| {
                            button(format!("provider-clear-{plugin}-{key}"), "CLEAR".into(), TEXT_DIMMER, true, move |cx| {
                                let (p, k) = (plugin.clone(), key.clone());
                                app.update(cx, |this, cx| this.clear_provider_secret(&p, &k, cx));
                            })
                        })),
                )
        }))
        .child(
            div()
                .flex()
                .gap(px(6.0))
                .child(button(format!("provider-save-{plugin}"), "SAVE & TEST".into(), OK, true, move |cx| a_save.update(cx, |this, cx| this.save_provider_form(cx))))
                .child(button(format!("provider-cancel-{plugin}"), "CANCEL".into(), TEXT_SECONDARY, true, move |cx| a_cancel.update(cx, |this, cx| this.close_provider_form(cx)))),
        )
}

/// Whether the provider's token can act on servers, not just read.
fn can_act(manifest: &crow_config_core::PluginManifest) -> bool {
    use crow_provider_core::capabilities::{INSTANCES_POWER, SNAPSHOTS};
    manifest.plugin.capabilities.iter().any(|c| c == INSTANCES_POWER || c == SNAPSHOTS)
}

fn tag(text: String, color: Rgba) -> impl IntoElement {
    div().px(px(5.0)).py(px(1.0)).border_1().border_color(BORDER_DEFAULT).font_family(FONT_MONO).text_size(px(9.0)).text_color(color).child(text)
}

fn button(id: String, label: String, color: Rgba, enabled: bool, on_click: impl Fn(&mut App) + 'static) -> Stateful<Div> {
    div()
        .id(SharedString::from(id))
        .px(px(9.0))
        .py(px(4.0))
        .border_1()
        .border_color(if enabled { color.opacity(0.6) } else { BORDER_DEFAULT })
        .text_color(if enabled { color } else { TEXT_FAINTER })
        .font_family(FONT_MONO)
        .text_size(px(10.0))
        .font_weight(FontWeight::BOLD)
        .when(enabled, |d| d.cursor_pointer().hover(|s| s.bg(BG_ROW_HOVER)).on_click(move |_ev, _window, cx| on_click(cx)))
        .child(label)
}

/// RFC 3339 → "2026-09-25 17:03".
fn short_time(t: &str) -> String {
    chrono::DateTime::parse_from_rfc3339(t).map(|d| d.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M").to_string()).unwrap_or_else(|_| t.to_string())
}
