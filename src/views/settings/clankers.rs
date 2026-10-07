use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::*;
use crate::theme::*;
use gpui_kit::component::input::{Input, InputState};

use crate::app::{ClankerInputs, CrowApp};
use crate::components::icons::{TablerIcon, inherited_icon};
use crate::components::sparkline::dynamic_sparkline;
use crate::vault::ClankerProviderConfig;
use crate::views::settings::clankers_state::ClankersState;

pub fn render_clankers_view(app: Entity<CrowApp>, clankers: &ClankersState, secrets_blocker: Option<String>, secrets_notice: Option<&str>) -> impl IntoElement {
    let providers = &clankers.providers;
    let total_calls_30d: u64 = providers.iter().map(|p| p.calls_30d).sum();
    let configured_count = providers.iter().filter(|p| !p.api_key.trim().is_empty()).count();
    // What would really be asked: the keyed primary (or the first keyed
    // provider), and a keyed backup that isn't it.
    let keyed = |p: &&ClankerProviderConfig| !p.api_key.trim().is_empty();
    let default_provider = providers.iter().filter(keyed).find(|p| p.is_default).or_else(|| providers.iter().find(keyed)).cloned();
    let backup_provider = providers.iter().filter(keyed).find(|p| p.is_backup && Some(&p.id) != default_provider.as_ref().map(|d| &d.id)).cloned();

    let demo_log = &clankers.demo_log;
    let demo_output = clankers.demo_output.as_ref().map(|r| r.as_ref().map(String::as_str).map_err(String::as_str));
    let demo_loading = clankers.demo_loading;

    div()
        .flex_1()
        .min_w(px(0.0))
        .flex()
        .flex_col()
        .bg(BG_APP)
        // 1. Subheader / Top Bar
        .child(
            div()
                .h(px(40.0))
                .flex_none()
                .flex()
                .items_center()
                .justify_between()
                .px(px(14.0))
                .bg(BG_PANEL)
                .border_b_1()
                .border_color(BORDER_PANEL)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(10.0))
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(12.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(TEXT_PRIMARY)
                                .child("CLANKERS · LLM PROVIDERS & KEYS"),
                        )
                        .child(
                            div()
                                .px(px(6.0))
                                .py(px(2.0))
                                .bg(hex_rgb(0x181822))
                                .border_1()
                                .border_color(BORDER_DEFAULT)
                                .font_family(FONT_MONO)
                                .text_size(px(9.5))
                                .text_color(TEXT_SECONDARY)
                                .child(format!("{}/{} CONFIGURED", configured_count, providers.len())),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(12.0))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(6.0))
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.5))
                                        .text_color(TEXT_DIM)
                                        .child("30d Cumulative Calls:"),
                                )
                                .child(
                                    div()
                                        .px(px(6.0))
                                        .py(px(2.0))
                                        .bg(hex_rgb(0x122416))
                                        .border_1()
                                        .border_color(OK)
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.5))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(OK)
                                        .child(format!("{} calls", total_calls_30d)),
                                ),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(6.0))
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.5))
                                        .text_color(TEXT_DIM)
                                        .child("Primary · backup:"),
                                )
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.5))
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(TEXT_PRIMARY)
                                        .child(format!(
                                            "{} · {}",
                                            default_provider.map(|p| p.display_name).unwrap_or_else(|| "none".into()),
                                            backup_provider.map(|p| p.display_name).unwrap_or_else(|| "no backup".into())
                                        )),
                                ),
                        ),
                ),
        )
        // 2. Scrollable Body Content
        .child(
            div()
                .id("clankers-content-scroll")
                .flex_1()
                .overflow_y_scrollbar()
                .p(px(14.0))
                .flex()
                .flex_col()
                .gap(px(16.0))
                // Section Info Banner
                .child(
                    div()
                        .p(px(12.0))
                        .bg(BG_OVERLAY_PANEL)
                        .border_1()
                        .border_color(BORDER_PANEL)
                        .flex()
                        .items_start()
                        .gap(px(10.0))
                        .child(
                            div()
                                .w(px(20.0))
                                .h(px(20.0))
                                .flex()
                                .items_center()
                                .justify_center()
                                .text_color(TEXT_PRIMARY)
                                .child(inherited_icon(TablerIcon::Cpu, px(16.0))),
                        )
                        .child(
                            div()
                                .flex_1()
                                .flex()
                                .flex_col()
                                .gap(px(3.0))
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(11.5))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(TEXT_PRIMARY)
                                        .child("AI Usability Engine (Log Clarification & ELI5 Assistance)"),
                                )
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.5))
                                        .text_color(TEXT_MUTED)
                                        .line_height(px(15.0))
                                        .child(
                                            "Configure API keys for LLM providers. Crow calls them only when you ask, to explain journal lines in plain English. Keys are stored encrypted in Crow's vault (its key is held by your OS keyring, or locked by your vault password when it's on) and sent through curl's stdin, never the command line. Log lines you send go to that provider."
                                        ),
                                ),
                        ),
                )
                .children(secrets_notice.map(|n| crate::views::settings::providers::notice(n.to_string(), TEXT_SECONDARY, None, Some(app.clone()))))
                .children(secrets_blocker.map(|why| crate::views::settings::providers::notice(format!("Keys can't be saved right now: {why}."), WARN, Some(app.clone()), None)))
                // The sandbox first: its answer grows the box, and at the
                // bottom of the page that happened out of view.
                .child(render_eli5_sandbox(app.clone(), demo_log, demo_output, demo_loading))
                // Providers Grid
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(11.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(TEXT_SECONDARY)
                        .child("SUPPORTED CLANKER PROVIDERS & 30-DAY TELEMETRY"),
                )
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .gap(px(12.0))
                        .children(providers.iter().map(|prov| {
                            render_provider_card(app.clone(), prov, clankers.key_checking.contains(&prov.id), clankers.key_checks.get(&prov.id))
                        })),
                ),
        )
}

fn render_provider_card(app: Entity<CrowApp>, prov: &ClankerProviderConfig, checking: bool, check: Option<&Result<String, String>>) -> impl IntoElement {
    let p_id = prov.id.clone();
    let is_configured = !prov.api_key.trim().is_empty();
    let masked_key = if is_configured {
        // Only the last four characters, like a card number.
        let k = prov.api_key.trim();
        let tail: String = k.chars().rev().take(4).collect::<Vec<_>>().into_iter().rev().collect();
        if k.chars().count() > 12 { format!("set · ends …{tail}") } else { "set".to_string() }
    } else {
        "NO KEY CONFIGURED".to_string()
    };

    let app_edit = app.clone();
    let p_id_edit = p_id.clone();
    let app_default = app.clone();
    let p_id_default = p_id.clone();
    let app_test = app.clone();
    let p_id_test = p_id.clone();
    let app_reset = app.clone();
    let p_id_reset = p_id.clone();

    div()
        .w(px(380.0))
        .flex_none()
        .p(px(12.0))
        .bg(BG_PANEL)
        .border_1()
        .border_color(if prov.is_default && is_configured { OK } else if prov.is_backup && is_configured { hex_rgb(0x8ab4ff) } else { BORDER_DEFAULT })
        .flex()
        .flex_col()
        .gap(px(10.0))
        // Card Header
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(12.5))
                                .font_weight(FontWeight::BOLD)
                                .text_color(TEXT_MAX)
                                .child(prov.display_name.clone()),
                        )
                        // A role means something only with a key to answer with.
                        .children(if prov.is_default && is_configured {
                            Some(
                                div()
                                    .px(px(6.0))
                                    .py(px(1.5))
                                    .bg(OK_BG)
                                    .border_1()
                                    .border_color(OK)
                                    .font_family(FONT_MONO)
                                    .text_size(px(9.0))
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(OK)
                                    .child("★ PRIMARY"),
                            )
                        } else {
                            None
                        })
                        .children((prov.is_backup && is_configured).then(|| {
                            div()
                                .px(px(6.0))
                                .py(px(1.5))
                                .bg(hex_rgba(0x8ab4ff, 0.10))
                                .border_1()
                                .border_color(hex_rgb(0x8ab4ff))
                                .font_family(FONT_MONO)
                                .text_size(px(9.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(hex_rgb(0x8ab4ff))
                                .child("↺ BACKUP")
                        })),
                )
                .child(
                    div()
                        .px(px(6.0))
                        .py(px(2.0))
                        .bg(if is_configured { hex_rgb(0x132216) } else { hex_rgb(0x201416) })
                        .border_1()
                        .border_color(if is_configured { OK } else { BORDER_DEFAULT })
                        .font_family(FONT_MONO)
                        .text_size(px(9.5))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(if is_configured { OK } else { TEXT_DIM })
                        .child(if is_configured { "READY" } else { "NOT SETUP" }),
                ),
        )
        // Key and Model Preview Strip
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(4.0))
                .p(px(8.0))
                .bg(BG_OVERLAY_PANEL)
                .border_1()
                .border_color(hex_rgb(0x1e1e28))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(10.0))
                                .text_color(TEXT_DIM)
                                .child("API Key:"),
                        )
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(10.0))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(if is_configured { TEXT_PRIMARY } else { TEXT_FAINT })
                                .child(masked_key),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(10.0))
                                .text_color(TEXT_DIM)
                                .child("Model:"),
                        )
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(10.0))
                                .text_color(TEXT_SECONDARY)
                                .child(prov.model.clone()),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(9.5))
                                .text_color(TEXT_DIM)
                                .child("Endpoint:"),
                        )
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(9.5))
                                .text_color(TEXT_DIMMER)
                                .child(prov.base_url.clone()),
                        ),
                ),
        )
        // 30-Day Activity Sparkline & Counters
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .p(px(8.0))
                .bg(BG_APP)
                .border_1()
                .border_color(BORDER_DEFAULT)
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(2.0))
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(9.5))
                                .text_color(TEXT_DIM)
                                .child("30-Day Activity:"),
                        )
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(14.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(if prov.calls_30d > 0 { OK } else { TEXT_DIM })
                                .child(format!("{} calls", prov.calls_30d)),
                        )
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(9.0))
                                .text_color(TEXT_FAINT)
                                .child(format!("{} all-time", prov.total_calls)),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_end()
                        .gap(px(4.0))
                        .child(dynamic_sparkline(&prov.daily_history, Some(0.0), None, if prov.calls_30d > 0 { OK } else { TEXT_FAINT }))
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(9.0))
                                .text_color(TEXT_DIMMER)
                                .child(
                                    prov.last_used_at.as_deref()
                                        .map(|ts| {
                                            if ts.len() >= 19 {
                                                format!("Last used: {}", &ts[..10])
                                            } else {
                                                format!("Last used: {}", ts)
                                            }
                                        })
                                        .unwrap_or_else(|| "Never used".into())
                                ),
                        ),
                ),
        )
        // Card Actions
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .pt(px(4.0))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        // Edit Key & Model Button
                        .child(
                            div()
                                .id(ElementId::Name(format!("btn-edit-clanker-{}", prov.id).into()))
                                .px(px(8.0))
                                .py(px(4.0))
                                .bg(BG_CONTROL)
                                .border_1()
                                .border_color(BORDER_DEFAULT)
                                .font_family(FONT_MONO)
                                .text_size(px(10.0))
                                .text_color(TEXT_PRIMARY)
                                .cursor_pointer()
                                .hover(|s| s.bg(BG_ROW_HOVER))
                                .on_click(move |_ev, _window, cx| {
                                    let id = p_id_edit.clone();
                                    app_edit.update(cx, |this, cx| {
                                        this.open_edit_clanker_modal(&id, cx);
                                    });
                                })
                                .child("⚙ Edit Key"),
                        )
                        // Test Call Button
                        .child(
                            div()
                                .id(ElementId::Name(format!("btn-test-clanker-{}", prov.id).into()))
                                .px(px(8.0))
                                .py(px(4.0))
                                .bg(BG_CONTROL)
                                .border_1()
                                .border_color(BORDER_DEFAULT)
                                .font_family(FONT_MONO)
                                .text_size(px(10.0))
                                .text_color(TEXT_SECONDARY)
                                .cursor_pointer()
                                .hover(|s| s.bg(BG_ROW_HOVER))
                                .on_click(move |_ev, _window, cx| {
                                    let id = p_id_test.clone();
                                    app_test.update(cx, |this, cx| {
                                        this.test_clanker_key(&id, cx);
                                    });
                                })
                                .child(if checking { "testing…" } else { "⚡ Test Key" }),
                        ),
                )

                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        // Reset usage
                        .children(if prov.total_calls > 0 {
                            Some(
                                div()
                                    .id(ElementId::Name(format!("btn-reset-clanker-{}", prov.id).into()))
                                    .px(px(6.0))
                                    .py(px(4.0))
                                    .font_family(FONT_MONO)
                                    .text_size(px(9.5))
                                    .text_color(TEXT_FAINT)
                                    .cursor_pointer()
                                    .hover(|s| s.text_color(WARN))
                                    .on_click(move |_ev, _window, cx| {
                                        let id = p_id_reset.clone();
                                        app_reset.update(cx, |this, cx| {
                                            this.reset_clanker_stats(&id, cx);
                                        });
                                    })
                                    .child("Reset"),
                            )
                        } else {
                            None
                        })
                        // Backup: asked when the primary fails.
                        .children((is_configured && !prov.is_default).then(|| {
                            let (app_backup, id, on) = (app.clone(), p_id.clone(), prov.is_backup);
                            div()
                                .id(ElementId::Name(format!("btn-backup-clanker-{}", prov.id).into()))
                                .px(px(8.0))
                                .py(px(4.0))
                                .bg(BG_KEY)
                                .border_1()
                                .border_color(BORDER_DEFAULT)
                                .font_family(FONT_MONO)
                                .text_size(px(10.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(hex_rgb(0x8ab4ff))
                                .cursor_pointer()
                                .hover(|s| s.bg(BG_ROW_HOVER))
                                .on_click(move |_ev, _window, cx| {
                                    let id = id.clone();
                                    app_backup.update(cx, |this, cx| this.set_backup_clanker(if on { None } else { Some(&id) }, cx));
                                })
                                .child(if on { "Remove backup" } else { "Make backup" })
                        }))
                        // Primary: asked first.
                        .children(if !prov.is_default && is_configured {
                            Some(
                                div()
                                    .id(ElementId::Name(format!("btn-default-clanker-{}", prov.id).into()))
                                    .px(px(8.0))
                                    .py(px(4.0))
                                    .bg(BG_KEY)
                                    .border_1()
                                    .border_color(BORDER_DEFAULT)
                                    .font_family(FONT_MONO)
                                    .text_size(px(10.0))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(OK)
                                    .cursor_pointer()
                                    .hover(|s| s.bg(BG_ROW_HOVER))
                                    .on_click(move |_ev, _window, cx| {
                                        let id = p_id_default.clone();
                                        app_default.update(cx, |this, cx| {
                                            this.set_default_clanker(&id, cx);
                                        });
                                    })
                                    .child("Make primary"),
                            )
                        } else {
                            None
                        }),
                ),
        )
        // The last key check, on its own line and wrapped to the card
        // (it cleared itself after a few seconds).
        .children(check.map(|r| {
            let (text, color) = match r {
                Ok(t) => (format!("✓ {t}"), OK),
                Err(e) => (format!("✕ {e}"), CRIT),
            };
            div().w_full().font_family(FONT_MONO).text_size(px(9.5)).line_height(px(14.0)).text_color(color).child(text)
        }))
}

fn render_eli5_sandbox(
    app: Entity<CrowApp>,
    demo_log: &str,
    demo_output: Option<Result<&str, &str>>,
    demo_loading: bool,
) -> impl IntoElement {
    let app_preset1 = app.clone();
    let app_preset2 = app.clone();
    let app_preset3 = app.clone();
    let app_run = app.clone();

    div()
        .mt(px(8.0))
        .p(px(14.0))
        .bg(BG_PANEL)
        .border_1()
        .border_color(BORDER_PANEL)
        .flex()
        .flex_col()
        .gap(px(12.0))
        // Header
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(12.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(TEXT_PRIMARY)
                                .child("AI USABILITY SANDBOX: \"WTF IS THIS LOG TRYING TO SAY?\""),
                        )
                        .child(
                            div()
                                .px(px(6.0))
                                .py(px(1.5))
                                .bg(hex_rgb(0x1a1622))
                                .border_1()
                                .border_color(hex_rgb(0x604080))
                                .font_family(FONT_MONO)
                                .text_size(px(9.0))
                                .text_color(hex_rgb(0xbb9af7))
                                .child("ELI5 ASSISTANT PREVIEW"),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(9.5))
                                .text_color(TEXT_DIM)
                                .child("Load Presets:"),
                        )
                        .child(
                            div()
                                .id("btn-preset-oom")
                                .px(px(6.0))
                                .py(px(2.0))
                                .bg(BG_CONTROL)
                                .border_1()
                                .border_color(BORDER_DEFAULT)
                                .font_family(FONT_MONO)
                                .text_size(px(9.5))
                                .text_color(WARN)
                                .cursor_pointer()
                                .hover(|s| s.bg(BG_ROW_HOVER))
                                .on_click(move |_ev, _window, cx| {
                                    app_preset1.update(cx, |this, cx| {
                                        this.clankers.demo_log = "kernel: [  129.412033] Out of memory: Kill process 28419 (mysqld) score 812 or sacrifice child".into();
                                        this.clankers.demo_output = None;
                                        cx.notify();
                                    });
                                })
                                .child("OOM Killer"),
                        )
                        .child(
                            div()
                                .id("btn-preset-segfault")
                                .px(px(6.0))
                                .py(px(2.0))
                                .bg(BG_CONTROL)
                                .border_1()
                                .border_color(BORDER_DEFAULT)
                                .font_family(FONT_MONO)
                                .text_size(px(9.5))
                                .text_color(CRIT)
                                .cursor_pointer()
                                .hover(|s| s.bg(BG_ROW_HOVER))
                                .on_click(move |_ev, _window, cx| {
                                    app_preset2.update(cx, |this, cx| {
                                        this.clankers.demo_log = "nginx[1482]: segfault at 0 ip 00007f3b48201a08 sp 00007ffe3410 error 4 in libc.so.6".into();
                                        this.clankers.demo_output = None;
                                        cx.notify();
                                    });
                                })
                                .child("Segfault in Libc"),
                        )
                        .child(
                            div()
                                .id("btn-preset-systemd")
                                .px(px(6.0))
                                .py(px(2.0))
                                .bg(BG_CONTROL)
                                .border_1()
                                .border_color(BORDER_DEFAULT)
                                .font_family(FONT_MONO)
                                .text_size(px(9.5))
                                .text_color(TEXT_SECONDARY)
                                .cursor_pointer()
                                .hover(|s| s.bg(BG_ROW_HOVER))
                                .on_click(move |_ev, _window, cx| {
                                    app_preset3.update(cx, |this, cx| {
                                        this.clankers.demo_log = "systemd[1]: postgresql@16-main.service: Main process exited, code=exited, status=1/FAILURE".into();
                                        this.clankers.demo_output = None;
                                        cx.notify();
                                    });
                                })
                                .child("Systemd Failure"),
                        ),
                ),
        )
        // Input Box & Trigger
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(6.0))
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(10.0))
                        .text_color(TEXT_DIM)
                        .child("Sample Raw Journal / Kernel Panic Log Line:"),
                )
                .child(
                    div()
                        .p(px(10.0))
                        .bg(hex_rgb(0x0a0a0f))
                        .border_1()
                        .border_color(hex_rgb(0x2d2d3d))
                        .font_family(FONT_MONO)
                        .text_size(px(11.0))
                        .text_color(TEXT_MAX)
                        .child(demo_log.to_string()),
                )
                .child(
                    div()
                        .flex()
                        .justify_end()
                        .child(
                            div()
                                .id("btn-run-eli5")
                                .px(px(12.0))
                                .py(px(6.0))
                                .bg(hex_rgb(0x183020))
                                .border_1()
                                .border_color(OK)
                                .font_family(FONT_MONO)
                                .text_size(px(11.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(OK)
                                .cursor_pointer()
                                .hover(|s| s.bg(hex_rgb(0x22442c)))
                                .on_click(move |_ev, _window, cx| {
                                    app_run.update(cx, |this, cx| {
                                        this.run_clanker_eli5(cx);
                                    });
                                })
                                .child(if demo_loading { "ASKING…" } else { "TRANSLATE LOG (ELI5) ↵" }),
                        ),
                ),
        )
        // Output Translation Card
        .children(if let Some(out) = demo_output {
            let (out, color) = match out {
                Ok(text) => (text, OK),
                Err(e) => (e, CRIT),
            };
            Some(
                div()
                    .p(px(12.0))
                    .bg(hex_rgb(0x0c0c14))
                    .border_1()
                    .border_color(color)
                    .flex()
                    .flex_col()
                    .gap(px(6.0))
                    .child(
                        div()
                            .font_family(FONT_MONO)
                            .text_size(px(11.0))
                            .text_color(TEXT_PRIMARY)
                            .line_height(px(17.0))
                            .child(out.to_string()),
                    ),
            )
        } else {
            None
        })
}

/// LOAD MODELS and, once loaded, the provider's models as chips that fill
/// the Model field. Listing models generates nothing, so it costs nothing.
fn models_picker(clankers: &ClankersState, app: Entity<CrowApp>) -> impl IntoElement {
    let app_load = app.clone();
    let status = match (&clankers.models, clankers.models_loading) {
        (_, true) => Some(("asking the provider…".to_string(), TEXT_DIM)),
        (Some(Ok(m)), _) => Some((format!("{} models offered for this key", m.len()), TEXT_DIM)),
        (Some(Err(e)), _) => Some((e.clone(), CRIT)),
        (None, false) => None,
    };
    div()
        .flex()
        .flex_col()
        .gap(px(6.0))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(8.0))
                .child(
                    div()
                        .id("btn-load-clanker-models")
                        .px(px(8.0))
                        .py(px(3.0))
                        .bg(BG_KEY)
                        .border_1()
                        .border_color(BORDER_DEFAULT)
                        .font_family(FONT_MONO)
                        .text_size(px(10.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(TEXT_SECONDARY)
                        .cursor_pointer()
                        .hover(|s| s.bg(BG_ROW_HOVER))
                        .on_click(move |_ev, _window, cx| app_load.update(cx, |this, cx| this.load_clanker_models(cx)))
                        .child("LOAD MODELS"),
                )
                .children(status.map(|(t, c)| div().flex_1().min_w(px(0.0)).font_family(FONT_MONO).text_size(px(9.5)).text_color(c).child(t))),
        )
        .children(clankers.models.as_ref().and_then(|m| m.as_ref().ok()).map(|models| {
            div()
                .id("clanker-models")
                .max_h(px(120.0))
                .overflow_y_scrollbar()
                .flex()
                .flex_wrap()
                .gap(px(4.0))
                .children(models.iter().enumerate().map(|(i, m)| {
                    let (app, m2) = (app.clone(), m.clone());
                    div()
                        .id(ElementId::NamedInteger("clanker-model".into(), i as u64))
                        .px(px(6.0))
                        .py(px(2.0))
                        .bg(BG_CONTROL)
                        .border_1()
                        .border_color(BORDER_DEFAULT)
                        .font_family(FONT_MONO)
                        .text_size(px(10.0))
                        .text_color(TEXT_SECONDARY)
                        .cursor_pointer()
                        .hover(|s| s.bg(BG_ROW_HOVER).text_color(TEXT_PRIMARY))
                        .on_click(move |_ev, window, cx| {
                            let m = m2.clone();
                            app.update(cx, |this, cx| this.pick_clanker_model(&m, window, cx))
                        })
                        .child(m.clone())
                }))
        }))
}

pub fn render_clanker_modals(app: Entity<CrowApp>, clankers: &ClankersState, inputs: Option<&ClankerInputs>) -> Option<impl IntoElement> {
    let state = clankers.editing.as_ref()?;
    let inputs = inputs.filter(|i| i.provider_id == state.provider_id);
    let p_name = state.display_name.clone();
    let err = state.error_message.clone();

    let app_backdrop = app.clone();
    let app_close = app.clone();
    let app_submit = app.clone();

    let field = |label: &'static str, input: Option<&Entity<InputState>>| {
        div()
            .flex()
            .flex_col()
            .gap(px(4.0))
            .child(div().font_family(FONT_MONO).text_size(px(10.5)).text_color(TEXT_DIM).child(label))
            .children(input.map(|i| Input::new(i).font_family(FONT_MONO).text_size(px(11.0)).bg(BG_APP).rounded(px(2.0))))
    };

    Some(
        div()
            .id("clanker-modal-scrim")
            .occlude()
            .absolute()
            .inset_0()
            .bg(rgba(0x000000aa))
            .flex()
            .items_center()
            .justify_center()
            .on_click(move |_ev, _window, cx| {
                app_backdrop.update(cx, |this, cx| {
                    this.close_edit_clanker_modal(cx);
                });
            })
            .child(
                div()
                    .id("clanker-modal-box")
                    .w(px(520.0))
                    .p(px(20.0))
                    .bg(BG_PANEL)
                    .border_1()
                    .border_color(BORDER_DEFAULT)
                    .shadow_lg()
                    .flex()
                    .flex_col()
                    .gap(px(14.0))
                    .on_mouse_down(MouseButton::Left, |_ev, _window, cx| cx.stop_propagation()) // keep clicks inside from reaching the backdrop (which closes)
                    // Header
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(8.0))
                                    .child(
                                        div()
                                            .w(px(16.0))
                                            .h(px(16.0))
                                            .text_color(TEXT_PRIMARY)
                                            .child(inherited_icon(TablerIcon::Key, px(14.0))),
                                    )
                                    .child(
                                        div()
                                            .font_family(FONT_MONO)
                                            .text_size(px(13.0))
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(TEXT_MAX)
                                            .child(format!("Configure {} Clanker API Key", p_name)),
                                    ),
                            )
                            .child(
                                div()
                                    .id("btn-close-clanker-modal")
                                    .cursor_pointer()
                                    .text_color(TEXT_DIM)
                                    .hover(|s| s.text_color(TEXT_PRIMARY))
                                    .on_click(move |_ev, _window, cx| {
                                        app_close.update(cx, |this, cx| {
                                            this.close_edit_clanker_modal(cx);
                                        });
                                    })
                                    .child(inherited_icon(TablerIcon::X, px(14.0))),
                            ),
                    )
                    .child(field("API Key (stored encrypted in Crow's vault; never shown again):", inputs.map(|i| &i.key)))
                    .child(field("Model:", inputs.map(|i| &i.model)))
                    .child(models_picker(clankers, app.clone()))
                    .child(field("Custom API Endpoint / Proxy (optional):", inputs.map(|i| &i.base_url)))
                    // Error message
                    .children(if let Some(ref e) = err {
                        Some(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(10.0))
                                .text_color(CRIT)
                                .child(e.clone()),
                        )
                    } else {
                        None
                    })
                    // Action Buttons
                    .child(
                        div()
                            .flex()
                            .justify_end()
                            .gap(px(8.0))
                            .child(
                                div()
                                    .id("btn-submit-clanker")
                                    .px(px(12.0))
                                    .py(px(6.0))
                                    .bg(BG_KEY)
                                    .border_1()
                                    .border_color(BORDER_DEFAULT)
                                    .cursor_pointer()
                                    .hover(|s| s.bg(BG_ROW_HOVER))
                                    .on_click(move |_ev, _window, cx| {
                                        app_submit.update(cx, |this, cx| {
                                            this.submit_edit_clanker(cx);
                                        });
                                    })
                                    .child(
                                        div()
                                            .font_family(FONT_MONO)
                                            .text_size(px(11.0))
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(OK)
                                            .child("SAVE CONFIG ↵"),
                                    ),
                            ),
                    ),
            ),
    )
}

#[cfg(test)]
mod id_guard {
    /// Each provider card's buttons need ids unique to the provider: ids
    /// built from numbers every card shares (history length, call counts)
    /// made every Edit Key the same button, and only the last card's worked.
    #[test]
    fn card_button_ids_name_their_provider() {
        let src = include_str!("clankers.rs");
        assert!(!src.contains(concat!("daily_history.len() as u64 + prov.", "calls_30d")), "build card ids from prov.id");
    }
}
