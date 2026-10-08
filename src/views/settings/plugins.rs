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
                .child(div().font_family(FONT_MONO).text_size(px(12.0)).font_weight(FontWeight::BOLD).text_color(TEXT_PRIMARY).child("PLUGINS · WHAT CROW CONNECTS TO"))
                .child(div().px(px(6.0)).py(px(2.0)).border_1().border_color(BORDER_DEFAULT).font_family(FONT_MONO).text_size(px(9.5)).text_color(TEXT_SECONDARY).child(format!("{on}/{} ON", BUILTIN.len())))
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
                        .border_color(BORDER_DEFAULT)
                        .font_family(FONT_MONO)
                        .text_size(px(10.5))
                        .text_color(TEXT_SECONDARY)
                        .cursor_pointer()
                        .hover(|s| s.bg(BG_ROW_HOVER).text_color(TEXT_PRIMARY))
                        .on_click(move |_ev, _window, cx| app_check.update(cx, |this, cx| this.check_all_plugins(cx)))
                        .child(crate::components::icons::inherited_icon(TablerIcon::Refresh, px(12.0)))
                        .child("RE-CHECK"),
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
                .child(div().p(px(12.0)).bg(BG_OVERLAY_PANEL).border_1().border_color(BORDER_PANEL).font_family(FONT_MONO).text_size(px(10.5)).line_height(px(15.0)).text_color(TEXT_MUTED).child(
                    "Crow does nothing with these until you switch them on. A plugin that's on adds what it brings where it belongs (Multipass adds + LAUNCH VM to the Fleet; a cloud provider adds import, power and snapshots); one that's off isn't probed or shown.",
                ))
                .children(GROUPS.iter().map(|(category, title)| {
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(8.0))
                        .child(div().flex().items_center().gap(px(8.0)).child(div().font_family(FONT_MONO).text_size(px(10.5)).font_weight(FontWeight::BOLD).text_color(TEXT_SECONDARY).child(*title)).child(div().font_family(FONT_MONO).text_size(px(9.5)).text_color(TEXT_FAINT).child(*category)))
                        .children(BUILTIN.iter().filter(|p| p.category == *category).map(|p| card(p, plugins, provider_accounts, app.clone(), email)))
                })),
        )
}

fn card(p: &'static BuiltinPlugin, plugins: &PluginsState, accounts: &[crate::vault::ProviderAccount], app: Entity<CrowApp>, email: EmailParts) -> impl IntoElement {
    let on = plugins.enabled.contains(p.id);
    let status = plugins.status.get(p.id);
    let (text, color) = match status {
        _ if plugins.checking.contains(p.id) => ("checking…".to_string(), TEXT_DIMMER),
        Some(PluginStatus::Ready(v)) => (v.clone(), OK),
        Some(PluginStatus::Problem { summary, .. }) => (summary.clone(), WARN),
        Some(PluginStatus::Missing { .. }) => ("not installed on this machine".to_string(), TEXT_DIM),
        Some(PluginStatus::Unavailable) => ("not in this build".to_string(), TEXT_DIM),
        None => ("not checked yet".to_string(), TEXT_DIMMER),
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
        .bg(BG_PANEL)
        .border_1()
        .border_color(if on { OK.opacity(0.5) } else { BORDER_PANEL })
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
                                .bg(if on { OK_BG } else { BG_CONTROL })
                                .border_1()
                                .border_color(if on { OK } else { BORDER_STRONG })
                                .child(div().size(px(10.0)).rounded_full().bg(if on { OK } else { TEXT_DIM })),
                        ),
                )
                .child(div().font_family(FONT_MONO).text_size(px(12.5)).font_weight(FontWeight::BOLD).text_color(if on { TEXT_PRIMARY } else { TEXT_SECONDARY }).child(p.name))
                .child(div().flex_1())
                .child(div().flex().items_center().gap(px(6.0)).font_family(FONT_MONO).text_size(px(10.0)).child(div().size(px(6.0)).rounded_full().bg(color)).child(div().text_color(color).child(text))),
        )
        .child(div().font_family(FONT_MONO).text_size(px(10.5)).line_height(px(15.0)).text_color(TEXT_MUTED).child(p.about))
        .child(div().flex().flex_wrap().gap(px(4.0)).children(p.capabilities.iter().map(|c| {
            div().px(px(5.0)).py(px(1.0)).bg(BG_CHIP).font_family(FONT_MONO).text_size(px(9.0)).text_color(TEXT_TERTIARY).child(*c)
        })))
        // Accounts live with the providers; say how many and link there.
        .children((is_provider && on).then(|| {
            div()
                .id(ElementId::Name(format!("plugin-accounts-{}", p.id).into()))
                .font_family(FONT_MONO)
                .text_size(px(10.0))
                .text_color(hex_rgb(0x8ab4ff))
                .cursor_pointer()
                .hover(|s| s.text_color(TEXT_PRIMARY))
                .on_click(move |_ev, _window, cx| app_providers.update(cx, |this, cx| this.set_settings_section(SettingsSection::Providers, cx)))
                .child(match account_count {
                    0 => "No account yet · set one up in Providers →".to_string(),
                    1 => "1 account · Providers →".to_string(),
                    n => format!("{n} accounts · Providers →"),
                })
        }))
        .children(steps.iter().enumerate().map(|(i, s)| crate::views::fleet::lab_modal::setup_step(i, s, app.clone())))
        .children((p.id == "email" && on).then(|| super::email_form::email_form(app.clone(), email.0, email.1)))
}
