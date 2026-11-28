//! Settings → Plugins (ERR-138): every built-in integration, off until
//! switched on, with its category, capabilities and whether it works here.

use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::app::plugins::PluginsState;
use crate::app::{CrowApp, SettingsSection};
use crate::components::icons::TablerIcon;
use crate::plugins::{BuiltinPlugin, PluginStatus, BUILTIN};
use crate::theme::*;

const GROUPS: [(&str, &str); 4] = [
    ("provider.vms", "LOCAL VIRTUAL MACHINES"),
    ("provider.containers", "CONTAINERS"),
    ("provider.hosts", "CLOUD PROVIDERS"),
    ("provider.notify", "NOTIFICATIONS"),
];

type EmailParts<'a> = (&'a crate::app::notify::EmailState, Option<&'a crate::app::notify::EmailInputs>);

pub fn render_plugins_view(app: Entity<CrowApp>, plugins: &PluginsState, provider_accounts: &[crate::vault::ProviderAccount], email: EmailParts) -> impl IntoElement {
    let on = BUILTIN.iter().filter(|p| plugins.enabled.contains(p.id)).count();
    let app_check = app.clone();
    div()
        .flex_1()
        .min_w(px(0.0))
        .flex()
        .flex_col()
        .bg(bg_app())
        .child(
            div()
                .h(px(40.0))
                .flex_none()
                .flex()
                .items_center()
                .gap(px(10.0))
                .px(px(14.0))
                .bg(bg_panel())
                .border_b_1()
                .border_color(border_panel())
                .child(div().font_family(FONT_MONO).text_size(font_px(12.0)).font_weight(FontWeight::BOLD).text_color(text_primary()).child(t!("settings-plugins-plugins-what-crow-connects-to")))
                .child(crate::components::button::badge(text_secondary()).text_size(font_px(9.5)).child(t!("settings-plugins-on", on = on, count = BUILTIN.len())))
                .child(div().flex_1())
                .child(
                    div()
                        .id("btn-plugins-recheck")
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        .h(px(26.0))
                        .px(px(10.0))
                        .border_1()
                        .border_color(border_default())
                        .font_family(FONT_MONO)
                        .text_size(font_px(10.5))
                        .text_color(text_secondary())
                        .cursor_pointer()
                        .hover(|s| s.bg(bg_row_hover()).text_color(text_primary()))
                        .on_click(move |_ev, _window, cx| app_check.update(cx, |this, cx| this.check_all_plugins(cx)))
                        .child(crate::components::icons::inherited_icon(TablerIcon::Refresh, font_px(12.0)))
                        .child(t!("settings-plugins-re-check")),
                ),
        )
        .child(
            div()
                .id("plugins-scroll")
                .flex_1()
                .overflow_y_scrollbar()
                .p(px(14.0))
                .flex()
                .flex_col()
                .gap(px(16.0))
                .child(div().p(px(12.0)).bg(bg_overlay_panel()).border_1().border_color(border_panel()).font_family(FONT_MONO).text_size(font_px(10.5)).line_height(px(15.0)).text_color(text_muted()).child(
                    t!("settings-plugins-crow-does-nothing-with-these"),
                ))
                .child(super::user_plugins::user_plugins_section(&plugins.user, app.clone()))
                .children(GROUPS.iter().map(|(category, title)| {
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(8.0))
                        .child(div().flex().items_center().gap(px(8.0)).child(div().font_family(FONT_MONO).text_size(font_px(10.5)).font_weight(FontWeight::BOLD).text_color(text_secondary()).child(crate::i18n::tx(title))).child(div().font_family(FONT_MONO).text_size(font_px(9.5)).text_color(text_faint()).child(*category)))
                        .children(BUILTIN.iter().filter(|p| p.category == *category).map(|p| card(p, plugins, provider_accounts, app.clone(), email)))
                })),
        )
}

fn card(p: &'static BuiltinPlugin, plugins: &PluginsState, accounts: &[crate::vault::ProviderAccount], app: Entity<CrowApp>, email: EmailParts) -> impl IntoElement {
    let on = plugins.enabled.contains(p.id);
    let status = plugins.status.get(p.id);
    let (text, color) = match status {
        _ if plugins.checking.contains(p.id) => (t!("settings-plugins-checking").to_string(), text_dimmer()),
        Some(PluginStatus::Ready(v)) => (v.clone(), theme().ok),
        Some(PluginStatus::Problem { summary, .. }) => (summary.clone(), theme().warn),
        Some(PluginStatus::Missing { .. }) => (t!("settings-plugins-not-installed-on-this-machine").to_string(), text_dim()),
        Some(PluginStatus::Unavailable) => (t!("settings-plugins-not-in-this-build").to_string(), text_dim()),
        None => (t!("settings-plugins-not-checked-yet").to_string(), text_dimmer()),
    };
    let steps = match status {
        Some(PluginStatus::Problem { steps, .. }) | Some(PluginStatus::Missing { steps }) => steps.clone(),
        _ => Vec::new(),
    };
    let account_count = accounts.iter().filter(|a| a.plugin == p.id).count();
    let is_provider = p.category == "provider.hosts";
    let (app_switch, app_providers, id) = (app.clone(), app.clone(), p.id);
    div()
        .id(ElementId::Name(format!("plugin-card-{}", p.id).into()))
        .flex()
        .flex_col()
        .gap(px(8.0))
        .p(px(12.0))
        .bg(bg_panel())
        .border_1()
        .border_color(if on { theme().ok.opacity(0.5) } else { border_panel() })
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(10.0))
                .child(
                    div()
                        .id(ElementId::Name(format!("plugin-switch-{}", p.id).into()))
                        .cursor_pointer()
                        .on_click(move |_ev, _window, cx| app_switch.update(cx, |this, cx| this.set_plugin_enabled(id, !on, cx)))
                        .child(
                            div()
                                .w(px(30.0))
                                .h(px(16.0))
                                .rounded_full()
                                .p(px(2.0))
                                .flex()
                                .when(on, |d| d.justify_end())
                                .bg(if on { theme().ok_bg } else { bg_control() })
                                .border_1()
                                .border_color(if on { theme().ok } else { border_strong() })
                                .child(div().size(px(10.0)).rounded_full().bg(if on { theme().ok } else { text_dim() })),
                        ),
                )
                .child(div().font_family(FONT_MONO).text_size(font_px(12.5)).font_weight(FontWeight::BOLD).text_color(if on { text_primary() } else { text_secondary() }).child(crate::i18n::tx(p.name)))
                .child(div().flex_1())
                .child(div().flex().items_center().gap(px(6.0)).font_family(FONT_MONO).text_size(font_px(10.0)).child(div().size(px(6.0)).rounded_full().bg(color)).child(div().text_color(color).child(text))),
        )
        .child(div().font_family(FONT_MONO).text_size(font_px(10.5)).line_height(px(15.0)).text_color(text_muted()).child(crate::i18n::tx(p.about)))
        .child(div().flex().flex_wrap().gap(px(4.0)).children(p.capabilities.iter().map(|c| {
            div().px(px(5.0)).py(px(1.0)).bg(bg_chip()).font_family(FONT_MONO).text_size(font_px(9.0)).text_color(text_tertiary()).child(*c)
        })))
        // Accounts live with the providers; say how many and link there.
        .children((is_provider && on).then(|| {
            div()
                .id(ElementId::Name(format!("plugin-accounts-{}", p.id).into()))
                .font_family(FONT_MONO)
                .text_size(font_px(10.0))
                .text_color(accent())
                .cursor_pointer()
                .hover(|s| s.text_color(text_primary()))
                .on_click(move |_ev, _window, cx| app_providers.update(cx, |this, cx| this.set_settings_section(SettingsSection::Providers, cx)))
                .child(match account_count {
                    0 => t!("settings-plugins-no-account-yet-set-one").to_string(),
                    1 => t!("settings-plugins-1-account-providers").to_string(),
                    n => t!("settings-plugins-accounts-providers", n = n),
                })
        }))
        .children(steps.iter().enumerate().map(|(i, s)| crate::views::fleet::lab_modal::setup_step(i, s, app.clone())))
        .children((p.id == "email" && on).then(|| super::email_form::email_form(app.clone(), email.0, email.1)))
}
