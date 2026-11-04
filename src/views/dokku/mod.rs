//! The server's Dokku PaaS view (`ERR-203`..`ERR-210`): single-screen control
//! plane for Dokku apps, Procfile scaling, backing services, staged secrets,
//! domains & Let's Encrypt TLS, deploy keys, and live logs.

use std::collections::{BTreeMap, HashSet};

use gpui_kit::component::input::{Input, InputState};
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::app::dokku::{
    AppLifecycleAction, DokkuDrawerTab, DokkuInputs, DokkuSelection, DokkuState, DokkuTopPanel,
    PendingDokkuAction, ServiceLifecycleAction,
};
use crate::app::CrowApp;
use crate::components::icons::{inherited_icon, TablerIcon};
use crate::containers::{human_size, Scan as ContainerScan};
use crate::dokku::{
    BackingService, DokkuApp, DokkuScan, ServiceKind, PLUGIN_CATALOG,
};
use crate::theme::*;
use crate::vault::SshKeyRecord;
use crate::views::overview::service_inspector::{render_eli5, WAND};

const W_APP_MIN: f32 = 140.0;
const W_STATE: f32 = 120.0;
const W_SCALE: f32 = 150.0;
const W_USAGE: f32 = 120.0;
const W_DOMAINS: f32 = 210.0;
const W_SERVICES: f32 = 160.0;
const W_DEPLOY: f32 = 120.0;

fn mono(size: f32, color: Rgba) -> Div {
    div().font_family(FONT_MONO).text_size(px(size)).text_color(color)
}

fn button(id: impl Into<SharedString>, label: impl Into<SharedString>, color: Rgba) -> Stateful<Div> {
    div()
        .id(ElementId::Name(id.into()))
        .flex()
        .items_center()
        .gap(px(5.0))
        .h(px(22.0))
        .px(px(8.0))
        .border_1()
        .border_color(color.opacity(0.5))
        .font_family(FONT_MONO)
        .text_size(px(9.5))
        .font_weight(FontWeight::BOLD)
        .text_color(color)
        .cursor_pointer()
        .hover(move |s| s.bg(color.opacity(0.12)))
        .child(label.into())
}

fn chip(id: impl Into<SharedString>, label: impl Into<SharedString>, active: bool) -> Stateful<Div> {
    div()
        .id(ElementId::Name(id.into()))
        .flex()
        .items_center()
        .gap(px(5.0))
        .h(px(24.0))
        .px(px(9.0))
        .border_1()
        .border_color(if active { TEXT_PRIMARY } else { BORDER_DEFAULT })
        .bg(if active { BG_CONTROL_ALT } else { BG_CONTROL })
        .font_family(FONT_MONO)
        .text_size(px(10.0))
        .font_weight(if active { FontWeight::BOLD } else { FontWeight::NORMAL })
        .text_color(if active { TEXT_PRIMARY } else { TEXT_SECONDARY })
        .cursor_pointer()
        .hover(|s| s.bg(BG_ROW_HOVER).text_color(TEXT_PRIMARY))
        .child(label.into())
}

fn text_input(i: Option<&Entity<InputState>>) -> impl IntoElement {
    div()
        .flex_1()
        .min_w(px(0.0))
        .children(i.map(|i| Input::new(i).font_family(FONT_MONO).text_size(px(11.0)).bg(BG_APP).rounded(px(2.0))))
}

fn app_state_color(a: &DokkuApp) -> Rgba {
    if a.degraded() {
        CRIT
    } else if a.maintenance {
        WARN
    } else if a.running {
        OK
    } else {
        TEXT_DIM
    }
}

fn app_state_label(a: &DokkuApp) -> &'static str {
    if a.degraded() {
        t!("dokku-state-degraded")
    } else if a.maintenance {
        t!("dokku-state-maintenance")
    } else if a.running {
        t!("dokku-state-running")
    } else if a.total_desired() == 0 {
        t!("dokku-state-unscaled")
    } else {
        t!("dokku-state-stopped")
    }
}

pub fn dokku_view(
    st: &DokkuState,
    inputs: Option<&DokkuInputs>,
    server_name: &str,
    server_host: &str,
    enrolled_keys: &[SshKeyRecord],
    containers_scan: Option<&ContainerScan>,
    app: Entity<CrowApp>,
) -> impl IntoElement {
    let scan = st.current();
    let selected_app = scan.and_then(|s| st.selected_app(s));
    let selected_svc = scan.and_then(|s| st.selected_service(s));

    div()
        .size_full()
        .flex()
        .child(
            div()
                .flex_1()
                .min_w(px(0.0))
                .h_full()
                .flex()
                .flex_col()
                .bg(BG_APP)
                .child(header(st, server_name, app.clone()))
                .child(
                    div()
                        .id("dokku-scroll")
                        .flex_1()
                        .overflow_y_scrollbar()
                        .p(px(14.0))
                        .flex()
                        .flex_col()
                        .gap(px(14.0))
                        .children(st.busy.clone().map(|b| mono(10.5, WARN).child(t!("dokku-running-cmd", b = b))))
                        .children(st.result.clone().map(|(ok, m)| {
                            mono(10.5, if ok { OK } else { CRIT }).line_height(px(15.0)).child(m)
                        }))
                        .child(match &st.scan {
                            None => mono(11.0, TEXT_DIM).child(t!("dokku-reading-server")).into_any_element(),
                            Some(Err(e)) => mono(11.0, CRIT)
                                .child(t!("dokku-couldnt-read", e = e))
                                .into_any_element(),
                            Some(Ok(scan)) if !scan.installed => {
                                bootstrap_panel(st, app.clone()).into_any_element()
                            }
                            Some(Ok(scan)) => body(
                                scan,
                                st,
                                inputs,
                                server_host,
                                enrolled_keys,
                                containers_scan,
                                app.clone(),
                            )
                            .into_any_element(),
                        }),
                ),
        )
        .children(selected_app.zip(scan).map(|(a, s)| {
            app_drawer(a, s, st, inputs, server_host, containers_scan, app.clone()).into_any_element()
        }))
        .children(selected_svc.zip(scan).map(|(svc, s)| {
            service_drawer(svc, s, st, containers_scan, app.clone()).into_any_element()
        }))
}

fn header(st: &DokkuState, server_name: &str, app: Entity<CrowApp>) -> impl IntoElement {
    let scan = st.current();
    let installed = scan.is_some_and(|s| s.installed);
    let (a_glob, a_plug, a_keys, a_svc, a_app, a_ref) = (
        app.clone(),
        app.clone(),
        app.clone(),
        app.clone(),
        app.clone(),
        app,
    );

    div()
        .h(px(40.0))
        .flex_none()
        .flex()
        .items_center()
        .gap(px(8.0))
        .px(px(14.0))
        .bg(BG_PANEL)
        .border_b_1()
        .border_color(BORDER_PANEL)
        .child(
            mono(12.0, TEXT_PRIMARY)
                .font_weight(FontWeight::BOLD)
                .child(t!("dokku-header-title", server_name = server_name)),
        )
        .children(
            scan.filter(|s| !s.version.is_empty())
                .map(|s| {
                    div()
                        .px(px(6.0))
                        .py(px(2.0))
                        .bg(BG_CHIP)
                        .font_family(FONT_MONO)
                        .text_size(px(9.5))
                        .text_color(TEXT_TERTIARY)
                        .child(format!("v{}", s.version))
                }),
        )
        .children(
            scan.and_then(|s| s.global_domains.first())
                .map(|d| {
                    div()
                        .px(px(6.0))
                        .py(px(2.0))
                        .bg(BG_CHIP)
                        .font_family(FONT_MONO)
                        .text_size(px(9.5))
                        .text_color(TEXT_SECONDARY)
                        .child(format!("*.{d}"))
                }),
        )
        .children(scan.filter(|s| s.installed).map(|s| {
            let running = s.apps.iter().filter(|a| a.running).count();
            crate::components::button::badge(if s.apps.iter().any(|a| a.degraded()) {
                CRIT
            } else {
                TEXT_SECONDARY
            })
            .text_size(px(9.5))
            .child(t!("dokku-apps-running-count", running = running, total = s.apps.len()))
        }))
        .child(div().flex_1())
        .when(installed, |d| {
            let plug_count = scan.map(|s| s.plugins.len()).unwrap_or(0);
            let key_count = scan.map(|s| s.deploy_keys.len()).unwrap_or(0);
            d.child(
                chip(
                    "btn-dokku-top-global",
                    t!("dokku-btn-global-tls"),
                    st.top_panel == Some(DokkuTopPanel::GlobalSetup),
                )
                .on_click(move |_ev, w, cx| {
                    a_glob.update(cx, |this, cx| this.toggle_dokku_top_panel(DokkuTopPanel::GlobalSetup, w, cx))
                }),
            )
            .child(
                chip(
                    "btn-dokku-top-plugins",
                    t!("dokku-btn-plugins", count = plug_count),
                    st.top_panel == Some(DokkuTopPanel::Plugins),
                )
                .on_click(move |_ev, w, cx| {
                    a_plug.update(cx, |this, cx| this.toggle_dokku_top_panel(DokkuTopPanel::Plugins, w, cx))
                }),
            )
            .child(
                chip(
                    "btn-dokku-top-keys",
                    t!("dokku-btn-deploy-keys", count = key_count),
                    st.top_panel == Some(DokkuTopPanel::DeployKeys),
                )
                .on_click(move |_ev, w, cx| {
                    a_keys.update(cx, |this, cx| this.toggle_dokku_top_panel(DokkuTopPanel::DeployKeys, w, cx))
                }),
            )
            .child(
                chip(
                    "btn-dokku-top-new-svc",
                    t!("dokku-btn-new-service"),
                    st.top_panel == Some(DokkuTopPanel::NewService),
                )
                .on_click(move |_ev, w, cx| {
                    a_svc.update(cx, |this, cx| this.toggle_dokku_top_panel(DokkuTopPanel::NewService, w, cx))
                }),
            )
            .child(
                chip(
                    "btn-dokku-top-new-app",
                    t!("dokku-btn-new-app"),
                    st.top_panel == Some(DokkuTopPanel::NewApp),
                )
                .on_click(move |_ev, w, cx| {
                    a_app.update(cx, |this, cx| this.toggle_dokku_top_panel(DokkuTopPanel::NewApp, w, cx))
                }),
            )
        })
        .child(
            div()
                .id("btn-dokku-refresh")
                .flex()
                .items_center()
                .gap(px(6.0))
                .h(px(24.0))
                .px(px(10.0))
                .border_1()
                .border_color(BORDER_DEFAULT)
                .font_family(FONT_MONO)
                .text_size(px(10.0))
                .text_color(TEXT_SECONDARY)
                .cursor_pointer()
                .hover(|s| s.bg(BG_ROW_HOVER).text_color(TEXT_PRIMARY))
                .on_click(move |_ev, _window, cx| a_ref.update(cx, |this, cx| this.refresh_dokku(cx)))
                .child(inherited_icon(TablerIcon::Refresh, px(12.0)))
                .child(if st.loading {
                    t!("dokku-reading")
                } else {
                    t!("dokku-refresh")
                }),
        )
}

fn bootstrap_panel(st: &DokkuState, app: Entity<CrowApp>) -> impl IntoElement {
    let is_pending = st.pending == Some(PendingDokkuAction::BootstrapDokku);
    let app_req = app.clone();
    div()
        .flex()
        .flex_col()
        .gap(px(10.0))
        .p(px(16.0))
        .bg(BG_PANEL)
        .border_1()
        .border_color(BORDER_PANEL)
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(8.0))
                .child(inherited_icon(TablerIcon::Rocket, px(15.0)).text_color(WARN))
                .child(
                    mono(12.0, TEXT_PRIMARY)
                        .font_weight(FontWeight::BOLD)
                        .child(t!("dokku-not-installed-title")),
                ),
        )
        .child(
            mono(10.5, TEXT_SECONDARY)
                .line_height(px(16.0))
                .child(t!("dokku-not-installed-desc")),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(8.0))
                .child(
                    button("btn-dokku-bootstrap", t!("dokku-btn-bootstrap"), OK).on_click(
                        move |_ev, _w, cx| {
                            app_req.update(cx, |this, cx| {
                                this.request_dokku_action(PendingDokkuAction::BootstrapDokku, cx)
                            })
                        },
                    ),
                ),
        )
        .when(is_pending, |d| {
            d.child(confirm_banner(
                t!("dokku-confirm-bootstrap"),
                "curl -fsSL https://get.docker.com | sh && DOKKU_TAG=v0.35.15 bash bootstrap.sh".into(),
                app,
            ))
        })
}

fn confirm_banner(question: impl Into<SharedString>, command: String, app: Entity<CrowApp>) -> Div {
    let (app_yes, app_no) = (app.clone(), app);
    div()
        .flex()
        .items_center()
        .gap(px(8.0))
        .p(px(8.0))
        .bg(WARN.opacity(0.06))
        .border_1()
        .border_color(WARN.opacity(0.4))
        .child(
            div()
                .flex_1()
                .min_w(px(0.0))
                .flex()
                .flex_col()
                .gap(px(2.0))
                .child(mono(10.5, TEXT_PRIMARY).child(question.into()))
                .child(mono(9.5, TEXT_DIM).child(command)),
        )
        .child(
            button("btn-dokku-confirm-yes", t!("dokku-confirm"), WARN)
                .on_click(move |_ev, _w, cx| app_yes.update(cx, |this, cx| this.confirm_dokku_pending(cx))),
        )
        .child(
            button("btn-dokku-confirm-no", t!("dokku-cancel"), TEXT_DIM)
                .on_click(move |_ev, _w, cx| app_no.update(cx, |this, cx| this.cancel_dokku_pending(cx))),
        )
}

fn body(
    scan: &DokkuScan,
    st: &DokkuState,
    inputs: Option<&DokkuInputs>,
    server_host: &str,
    enrolled_keys: &[SshKeyRecord],
    containers_scan: Option<&ContainerScan>,
    app: Entity<CrowApp>,
) -> impl IntoElement {
    let compact = st.selected.is_some();
    div()
        .flex()
        .flex_col()
        .gap(px(14.0))
        .children(st.top_panel.map(|panel| {
            top_panel_view(panel, scan, st, inputs, enrolled_keys, app.clone()).into_any_element()
        }))
        // Applications section
        .child(
            div()
                .flex()
                .flex_col()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .pb(px(6.0))
                        .child(
                            mono(10.5, TEXT_SECONDARY)
                                .font_weight(FontWeight::BOLD)
                                .child(t!("dokku-section-apps")),
                        )
                        .child(mono(9.5, TEXT_FAINT).child(format!("({})", scan.apps.len()))),
                )
                .when(scan.apps.is_empty(), |d| {
                    let app_new = app.clone();
                    d.child(
                        div()
                            .p(px(14.0))
                            .bg(BG_PANEL)
                            .border_1()
                            .border_color(BORDER_PANEL)
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(mono(10.5, TEXT_DIM).child(t!("dokku-no-apps-yet")))
                            .child(
                                button("btn-dokku-empty-new-app", t!("dokku-btn-new-app"), OK)
                                    .on_click(move |_ev, w, cx| {
                                        app_new.update(cx, |this, cx| {
                                            this.toggle_dokku_top_panel(DokkuTopPanel::NewApp, w, cx)
                                        })
                                    }),
                            ),
                    )
                })
                .when(!scan.apps.is_empty(), |d| {
                    d.child(apps_table_head(compact)).children(
                        scan.apps
                            .iter()
                            .map(|a| app_row(a, st, server_host, containers_scan, app.clone())),
                    )
                }),
        )
        // Backing Services section
        .child(
            div()
                .flex()
                .flex_col()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .pb(px(6.0))
                        .child(
                            mono(10.5, TEXT_SECONDARY)
                                .font_weight(FontWeight::BOLD)
                                .child(t!("dokku-section-services")),
                        )
                        .child(mono(9.5, TEXT_FAINT).child(format!("({})", scan.services.len()))),
                )
                .when(scan.services.is_empty(), |d| {
                    let app_svc = app.clone();
                    d.child(
                        div()
                            .p(px(14.0))
                            .bg(BG_PANEL)
                            .border_1()
                            .border_color(BORDER_PANEL)
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(mono(10.5, TEXT_DIM).child(t!("dokku-no-services-yet")))
                            .child(
                                button("btn-dokku-empty-new-svc", t!("dokku-btn-new-service"), OK)
                                    .on_click(move |_ev, w, cx| {
                                        app_svc.update(cx, |this, cx| {
                                            this.toggle_dokku_top_panel(DokkuTopPanel::NewService, w, cx)
                                        })
                                    }),
                            ),
                    )
                })
                .when(!scan.services.is_empty(), |d| {
                    d.child(services_table_head(compact)).children(
                        scan.services
                            .iter()
                            .map(|s| service_row(s, st, containers_scan, app.clone())),
                    )
                }),
        )
}

fn top_panel_view(
    panel: DokkuTopPanel,
    scan: &DokkuScan,
    st: &DokkuState,
    inputs: Option<&DokkuInputs>,
    enrolled_keys: &[SshKeyRecord],
    app: Entity<CrowApp>,
) -> impl IntoElement {
    let panel_box = || {
        div()
            .flex()
            .flex_col()
            .gap(px(10.0))
            .p(px(12.0))
            .bg(BG_PANEL)
            .border_1()
            .border_color(BORDER_PANEL)
    };

    match panel {
        DokkuTopPanel::GlobalSetup => {
            let (a_dom, a_le) = (app.clone(), app);
            panel_box()
                .child(
                    mono(10.5, TEXT_PRIMARY)
                        .font_weight(FontWeight::BOLD)
                        .child(t!("dokku-panel-global-title")),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .child(mono(10.0, TEXT_SECONDARY).w(px(150.0)).child(t!("dokku-global-vhost-domain")))
                        .child(text_input(inputs.map(|i| &i.global_domain)))
                        .child(
                            button("btn-dokku-save-global-domain", t!("dokku-btn-set-domain"), OK)
                                .on_click(move |_ev, _w, cx| {
                                    a_dom.update(cx, |this, cx| this.save_dokku_global_domain(cx))
                                }),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .child(mono(10.0, TEXT_SECONDARY).w(px(150.0)).child(t!("dokku-le-contact-email")))
                        .child(text_input(inputs.map(|i| &i.letsencrypt_email)))
                        .child(
                            button(
                                "btn-dokku-save-le-email",
                                t!("dokku-btn-save-le-cron"),
                                OK,
                            )
                            .on_click(move |_ev, _w, cx| {
                                a_le.update(cx, |this, cx| this.save_dokku_letsencrypt_global(cx))
                            }),
                        ),
                )
                .child(
                    mono(9.5, TEXT_FAINT).child(if scan.letsencrypt_cron {
                        t!("dokku-le-cron-active")
                    } else {
                        t!("dokku-le-cron-inactive")
                    }),
                )
        }
        DokkuTopPanel::Plugins => panel_box()
            .child(
                mono(10.5, TEXT_PRIMARY)
                    .font_weight(FontWeight::BOLD)
                    .child(t!("dokku-panel-plugins-title")),
            )
            .child(mono(9.5, TEXT_DIM).child(t!("dokku-panel-plugins-desc")))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(6.0))
                    .children(PLUGIN_CATALOG.iter().map(|entry| {
                        let installed = scan.has_plugin(entry.id);
                        let ver = scan
                            .plugins
                            .iter()
                            .find(|p| p.name == entry.id)
                            .map(|p| p.version.clone())
                            .unwrap_or_default();
                        let app_inst = app.clone();
                        let (id, url) = (entry.id, entry.url);
                        div()
                            .flex()
                            .items_center()
                            .gap(px(10.0))
                            .px(px(8.0))
                            .py(px(6.0))
                            .bg(BG_APP)
                            .border_1()
                            .border_color(BORDER_PANEL)
                            .child(
                                mono(10.5, TEXT_PRIMARY)
                                    .font_weight(FontWeight::BOLD)
                                    .w(px(120.0))
                                    .child(entry.label),
                            )
                            .child(mono(10.0, TEXT_TERTIARY).flex_1().child(entry.about))
                            .children((!ver.is_empty()).then(|| mono(9.5, TEXT_FAINT).child(ver)))
                            .child(if installed {
                                crate::components::button::badge(OK)
                                    .text_size(px(9.0))
                                    .child(t!("dokku-plugin-installed"))
                                    .into_any_element()
                            } else {
                                button(
                                    format!("btn-dokku-install-plugin-{id}"),
                                    t!("dokku-btn-install"),
                                    OK,
                                )
                                .on_click(move |_ev, _w, cx| {
                                    app_inst.update(cx, |this, cx| this.install_dokku_plugin(id, url, cx))
                                })
                                .into_any_element()
                            })
                    })),
            ),
        DokkuTopPanel::DeployKeys => {
            let authorized_fps: HashSet<&str> = scan
                .deploy_keys
                .iter()
                .map(|k| k.fingerprint.as_str())
                .collect();
            panel_box()
                .child(
                    mono(10.5, TEXT_PRIMARY)
                        .font_weight(FontWeight::BOLD)
                        .child(t!("dokku-panel-keys-title")),
                )
                .child(mono(9.5, TEXT_DIM).child(t!("dokku-panel-keys-desc")))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(6.0))
                        .children(enrolled_keys.iter().map(|k| {
                            let is_auth = authorized_fps.contains(k.fingerprint.as_str())
                                || scan.deploy_keys.iter().any(|dk| dk.name == k.name);
                            let app_auth = app.clone();
                            let (kname, pubkey) = (k.name.clone(), k.public_key.clone());
                            div()
                                .flex()
                                .items_center()
                                .gap(px(10.0))
                                .px(px(8.0))
                                .py(px(6.0))
                                .bg(BG_APP)
                                .border_1()
                                .border_color(BORDER_PANEL)
                                .child(inherited_icon(TablerIcon::Key, px(12.0)).text_color(TEXT_DIM))
                                .child(
                                    mono(10.5, TEXT_PRIMARY)
                                        .font_weight(FontWeight::BOLD)
                                        .w(px(140.0))
                                        .child(k.name.clone()),
                                )
                                .child(
                                    mono(9.5, TEXT_FAINT)
                                        .flex_1()
                                        .overflow_hidden()
                                        .text_ellipsis()
                                        .child(k.fingerprint.clone()),
                                )
                                .child(if is_auth {
                                    crate::components::button::badge(OK)
                                        .text_size(px(9.0))
                                        .child(t!("dokku-key-authorized"))
                                        .into_any_element()
                                } else {
                                    button(
                                        format!("btn-dokku-auth-key-{}", k.id),
                                        t!("dokku-btn-authorize-key"),
                                        OK,
                                    )
                                    .on_click(move |_ev, _w, cx| {
                                        app_auth.update(cx, |this, cx| {
                                            this.authorize_dokku_ssh_key(&kname, &pubkey, cx)
                                        })
                                    })
                                    .into_any_element()
                                })
                        }))
                        .children(scan.deploy_keys.iter().map(|dk| {
                            let app_rm = app.clone();
                            let dk_name = dk.name.clone();
                            div()
                                .flex()
                                .items_center()
                                .gap(px(10.0))
                                .px(px(8.0))
                                .py(px(5.0))
                                .border_t_1()
                                .border_color(BORDER_PANEL)
                                .child(mono(10.0, TEXT_SECONDARY).w(px(140.0)).child(dk.name.clone()))
                                .child(mono(9.5, TEXT_FAINT).flex_1().child(dk.fingerprint.clone()))
                                .child(
                                    button(
                                        format!("btn-dokku-rm-key-{}", dk.name),
                                        t!("dokku-btn-revoke"),
                                        CRIT,
                                    )
                                    .on_click(move |_ev, _w, cx| {
                                        app_rm.update(cx, |this, cx| {
                                            this.remove_dokku_ssh_key(&dk_name, cx)
                                        })
                                    }),
                                )
                        })),
                )
        }
        DokkuTopPanel::NewApp => {
            let app_create = app;
            panel_box()
                .child(
                    mono(10.5, TEXT_PRIMARY)
                        .font_weight(FontWeight::BOLD)
                        .child(t!("dokku-panel-new-app-title")),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .child(text_input(inputs.map(|i| &i.new_app_name)))
                        .child(
                            button("btn-dokku-create-app-go", t!("dokku-btn-create-app"), OK)
                                .on_click(move |_ev, _w, cx| {
                                    app_create.update(cx, |this, cx| this.create_dokku_app(cx))
                                }),
                        ),
                )
        }
        DokkuTopPanel::NewService => {
            let app_create = app.clone();
            panel_box()
                .child(
                    mono(10.5, TEXT_PRIMARY)
                        .font_weight(FontWeight::BOLD)
                        .child(t!("dokku-panel-new-service-title")),
                )
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .gap(px(6.0))
                        .children(ServiceKind::ALL.iter().map(|&k| {
                            let app_k = app.clone();
                            chip(
                                format!("btn-dokku-svc-kind-{}", k.id()),
                                k.label(),
                                st.new_service_kind == k,
                            )
                            .on_click(move |_ev, _w, cx| {
                                app_k.update(cx, |this, cx| this.set_dokku_new_service_kind(k, cx))
                            })
                        })),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .child(text_input(inputs.map(|i| &i.new_service_name)))
                        .child(
                            button(
                                "btn-dokku-create-service-go",
                                t!("dokku-btn-create-service"),
                                OK,
                            )
                            .on_click(move |_ev, _w, cx| {
                                app_create.update(cx, |this, cx| this.create_dokku_service(cx))
                            }),
                        ),
                )
        }
    }
}

fn apps_table_head(compact: bool) -> impl IntoElement {
    let cell = |w: Option<f32>, text: &'static str| {
        let d = mono(9.0, TEXT_FAINT).font_weight(FontWeight::SEMIBOLD).child(text);
        match w {
            Some(w) => d.w(px(w)).flex_none(),
            None => d.flex_1().min_w(px(W_APP_MIN)),
        }
    };
    div()
        .flex()
        .items_center()
        .h(px(24.0))
        .px(px(10.0))
        .gap(px(10.0))
        .bg(BG_SUBHEAD)
        .border_1()
        .border_color(BORDER_PANEL)
        .child(cell(None, t!("dokku-col-app")))
        .child(cell(Some(W_STATE), t!("dokku-col-state")))
        .child(cell(Some(W_SCALE), t!("dokku-col-scale")))
        .child(cell(Some(W_USAGE), t!("dokku-col-cpu-mem")))
        .when(!compact, |d| {
            d.child(cell(Some(W_DOMAINS), t!("dokku-col-domains")))
                .child(cell(Some(W_SERVICES), t!("dokku-col-services")))
                .child(cell(Some(W_DEPLOY), t!("dokku-col-deploy")))
        })
}

fn app_row(
    a: &DokkuApp,
    st: &DokkuState,
    _server_host: &str,
    containers_scan: Option<&ContainerScan>,
    app: Entity<CrowApp>,
) -> impl IntoElement {
    let is_selected = st.selected == Some(DokkuSelection::App(a.name.clone()));
    let compact = st.selected.is_some();
    let color = app_state_color(a);
    let scale_summary = if a.processes.is_empty() {
        "—".to_string()
    } else {
        a.processes
            .iter()
            .map(|p| format!("{}:{}/{}", p.proc_type, p.running, p.desired))
            .collect::<Vec<_>>()
            .join(" ")
    };
    let usage_text = containers_scan
        .and_then(|cs| cs.dokku_app_usage(&a.name))
        .map(|(cpu, mem)| format!("{cpu:.1}% · {}", human_size(mem)))
        .unwrap_or_else(|| "—".to_string());
    let domain_summary = a.domains.first().cloned().unwrap_or_else(|| "—".to_string());
    let svc_summary = if a.linked_services.is_empty() {
        "—".to_string()
    } else {
        a.linked_services
            .iter()
            .map(|(k, n)| format!("{}:{n}", k.id()))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let deploy_summary = if !a.git_sha.is_empty() {
        a.git_sha.clone()
    } else if !a.deploy_source_image.is_empty() {
        a.deploy_source_image.clone()
    } else {
        "—".to_string()
    };
    let name = a.name.clone();

    div()
        .id(ElementId::Name(format!("dokku-app-row-{}", a.name).into()))
        .flex()
        .items_center()
        .min_h(px(32.0))
        .px(px(10.0))
        .gap(px(10.0))
        .border_b_1()
        .border_l_1()
        .border_r_1()
        .border_color(BORDER_PANEL)
        .bg(if is_selected { BG_ROW_SELECTED } else { BG_PANEL })
        .cursor_pointer()
        .hover(|s| s.bg(BG_ROW_HOVER))
        .on_click(move |_ev, _w, cx| app.update(cx, |this, cx| this.select_dokku_app(&name, cx)))
        .child(
            div()
                .flex_1()
                .min_w(px(W_APP_MIN))
                .flex()
                .items_center()
                .gap(px(8.0))
                .child(div().flex_none().size(px(7.0)).rounded_full().bg(color))
                .child(
                    mono(11.0, TEXT_PRIMARY)
                        .font_weight(FontWeight::BOLD)
                        .whitespace_nowrap()
                        .overflow_hidden()
                        .text_ellipsis()
                        .child(a.name.clone()),
                )
                .when(a.ssl_enabled, |d| {
                    d.child(
                        crate::components::button::badge(OK)
                            .text_size(px(8.5))
                            .child(t!("dokku-badge-tls")),
                    )
                }),
        )
        .child(mono(10.5, color).w(px(W_STATE)).flex_none().child(app_state_label(a)))
        .child(
            mono(10.0, TEXT_SECONDARY)
                .w(px(W_SCALE))
                .flex_none()
                .whitespace_nowrap()
                .overflow_hidden()
                .text_ellipsis()
                .child(scale_summary),
        )
        .child(
            mono(10.0, TEXT_TERTIARY)
                .w(px(W_USAGE))
                .flex_none()
                .whitespace_nowrap()
                .child(usage_text),
        )
        .when(!compact, |d| {
            d.child(
                mono(10.0, TEXT_SECONDARY)
                    .w(px(W_DOMAINS))
                    .flex_none()
                    .whitespace_nowrap()
                    .overflow_hidden()
                    .text_ellipsis()
                    .child(domain_summary),
            )
            .child(
                mono(10.0, TEXT_TERTIARY)
                    .w(px(W_SERVICES))
                    .flex_none()
                    .whitespace_nowrap()
                    .overflow_hidden()
                    .text_ellipsis()
                    .child(svc_summary),
            )
            .child(
                mono(10.0, TEXT_DIM)
                    .w(px(W_DEPLOY))
                    .flex_none()
                    .whitespace_nowrap()
                    .overflow_hidden()
                    .text_ellipsis()
                    .child(deploy_summary),
            )
        })
}

fn services_table_head(compact: bool) -> impl IntoElement {
    let cell = |w: Option<f32>, text: &'static str| {
        let d = mono(9.0, TEXT_FAINT).font_weight(FontWeight::SEMIBOLD).child(text);
        match w {
            Some(w) => d.w(px(w)).flex_none(),
            None => d.flex_1().min_w(px(W_APP_MIN)),
        }
    };
    div()
        .flex()
        .items_center()
        .h(px(24.0))
        .px(px(10.0))
        .gap(px(10.0))
        .bg(BG_SUBHEAD)
        .border_1()
        .border_color(BORDER_PANEL)
        .child(cell(None, t!("dokku-col-service")))
        .child(cell(Some(W_STATE), t!("dokku-col-state")))
        .child(cell(Some(W_USAGE), t!("dokku-col-cpu-mem")))
        .when(!compact, |d| {
            d.child(cell(Some(W_DOMAINS), t!("dokku-col-engine")))
                .child(cell(Some(W_SERVICES), t!("dokku-col-linked-apps")))
        })
}

fn service_row(
    s: &BackingService,
    st: &DokkuState,
    containers_scan: Option<&ContainerScan>,
    app: Entity<CrowApp>,
) -> impl IntoElement {
    let is_selected = st.selected == Some(DokkuSelection::Service(s.kind, s.name.clone()));
    let compact = st.selected.is_some();
    let running = s.status == "running";
    let color = if running { OK } else { TEXT_DIM };
    let usage_text = containers_scan
        .and_then(|cs| cs.dokku_service_usage(s.kind.id(), &s.name))
        .map(|(cpu, mem)| format!("{cpu:.1}% · {}", human_size(mem)))
        .unwrap_or_else(|| "—".to_string());
    let linked = if s.linked_apps.is_empty() {
        "—".to_string()
    } else {
        s.linked_apps.join(", ")
    };
    let (kind, name) = (s.kind, s.name.clone());

    div()
        .id(ElementId::Name(format!("dokku-svc-row-{}-{}", s.kind.id(), s.name).into()))
        .flex()
        .items_center()
        .min_h(px(30.0))
        .px(px(10.0))
        .gap(px(10.0))
        .border_b_1()
        .border_l_1()
        .border_r_1()
        .border_color(BORDER_PANEL)
        .bg(if is_selected { BG_ROW_SELECTED } else { BG_PANEL })
        .cursor_pointer()
        .hover(|el| el.bg(BG_ROW_HOVER))
        .on_click(move |_ev, _w, cx| {
            app.update(cx, |this, cx| this.select_dokku_service(kind, &name, cx))
        })
        .child(
            div()
                .flex_1()
                .min_w(px(W_APP_MIN))
                .flex()
                .items_center()
                .gap(px(8.0))
                .child(div().flex_none().size(px(7.0)).rounded_full().bg(color))
                .child(
                    crate::components::button::badge(TEXT_SECONDARY)
                        .text_size(px(8.5))
                        .child(s.kind.label()),
                )
                .child(
                    mono(11.0, TEXT_PRIMARY)
                        .font_weight(FontWeight::BOLD)
                        .child(s.name.clone()),
                ),
        )
        .child(mono(10.5, color).w(px(W_STATE)).flex_none().child(s.status.clone()))
        .child(mono(10.0, TEXT_TERTIARY).w(px(W_USAGE)).flex_none().child(usage_text))
        .when(!compact, |d| {
            d.child(
                mono(10.0, TEXT_SECONDARY)
                    .w(px(W_DOMAINS))
                    .flex_none()
                    .whitespace_nowrap()
                    .overflow_hidden()
                    .text_ellipsis()
                    .child(if s.version.is_empty() {
                        s.kind.label().to_string()
                    } else {
                        s.version.clone()
                    }),
            )
            .child(
                mono(10.0, TEXT_TERTIARY)
                    .w(px(W_SERVICES))
                    .flex_none()
                    .whitespace_nowrap()
                    .overflow_hidden()
                    .text_ellipsis()
                    .child(linked),
            )
        })
}

// =============================================================================
// App Inspector Drawer (`ERR-206`, `ERR-207`, `ERR-208`, `ERR-209`, `ERR-210`)
// =============================================================================

fn app_drawer(
    a: &DokkuApp,
    scan: &DokkuScan,
    st: &DokkuState,
    inputs: Option<&DokkuInputs>,
    server_host: &str,
    containers_scan: Option<&ContainerScan>,
    app: Entity<CrowApp>,
) -> impl IntoElement {
    let color = app_state_color(a);
    let eli5_key = format!("dokku:{}", a.name);
    let (app_close, app_wand, app_console) = (app.clone(), app.clone(), app.clone());
    let app_name_console = a.name.clone();

    let lifecycle_btn = |action: AppLifecycleAction, label: &'static str, c: Rgba| {
        let (app_ent, name) = (app.clone(), a.name.clone());
        button(
            format!("btn-dokku-app-{}-{}", a.name, action.subcommand()),
            label,
            c,
        )
        .on_click(move |_ev, _w, cx| {
            app_ent.update(cx, |this, cx| {
                this.request_dokku_action(
                    PendingDokkuAction::App {
                        app: name.clone(),
                        action,
                    },
                    cx,
                )
            })
        })
    };

    let pending_banner = match &st.pending {
        Some(PendingDokkuAction::App { app: target, action }) if target == &a.name => {
            let cmd = if *action == AppLifecycleAction::Destroy {
                format!("dokku --force apps:destroy {}", a.name)
            } else {
                format!("dokku {} {}", action.subcommand(), a.name)
            };
            let q = match action {
                AppLifecycleAction::Destroy => t!("dokku-confirm-destroy-app"),
                AppLifecycleAction::MaintenanceToggle(true) => t!("dokku-confirm-maint-on"),
                AppLifecycleAction::MaintenanceToggle(false) => t!("dokku-confirm-maint-off"),
                _ => t!("dokku-confirm-app-action"),
            };
            Some(confirm_banner(q, cmd, app.clone()))
        }
        Some(PendingDokkuAction::Link { kind, service, app: target, link }) if target == &a.name => {
            let verb = if *link { "link" } else { "unlink" };
            let flag = if st.restart_on_link { "" } else { " --no-restart" };
            let cmd = format!("dokku {}:{verb}{flag} {service} {}", kind.id(), a.name);
            let q = if *link {
                t!("dokku-confirm-link-service", env_var = kind.env_var(), app = a.name)
            } else {
                t!("dokku-confirm-unlink-service", env_var = kind.env_var(), app = a.name)
            };
            Some(confirm_banner(q, cmd, app.clone()))
        }
        _ => None,
    };

    let tab_btn = |id: &'static str, label: &'static str, tab: DokkuDrawerTab| {
        let active = st.tab == tab;
        let app_t = app.clone();
        div()
            .id(id)
            .px(px(9.0))
            .py(px(6.0))
            .border_b_2()
            .border_color(if active { TEXT_PRIMARY } else { hex_rgba(0, 0.0) })
            .font_family(FONT_MONO)
            .text_size(px(10.0))
            .font_weight(FontWeight::BOLD)
            .text_color(if active { TEXT_PRIMARY } else { TEXT_DIM })
            .cursor_pointer()
            .hover(|s| s.text_color(TEXT_PRIMARY))
            .on_click(move |_ev, _w, cx| app_t.update(cx, |this, cx| this.set_dokku_tab(tab, cx)))
            .child(label)
    };

    let tab_content: AnyElement = match st.tab {
        DokkuDrawerTab::Overview => {
            drawer_overview_tab(a, st, inputs, containers_scan, app.clone()).into_any_element()
        }
        DokkuDrawerTab::Env => drawer_env_tab(a, st, inputs, app.clone()).into_any_element(),
        DokkuDrawerTab::Domains => drawer_domains_tab(a, inputs, app.clone()).into_any_element(),
        DokkuDrawerTab::Services => {
            drawer_services_tab(a, scan, st, app.clone()).into_any_element()
        }
        DokkuDrawerTab::Deploy => {
            drawer_deploy_tab(a, inputs, server_host, app.clone()).into_any_element()
        }
        DokkuDrawerTab::Logs => drawer_logs_tab(a, st, app.clone()).into_any_element(),
    };

    div()
        .w(px(550.0))
        .flex_none()
        .h_full()
        .flex()
        .flex_col()
        .bg(BG_PANEL)
        .border_l_1()
        .border_color(BORDER_PANEL)
        .child(
            div()
                .flex_none()
                .flex()
                .flex_col()
                .gap(px(8.0))
                .p(px(14.0))
                .border_b_1()
                .border_color(BORDER_PANEL)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .child(div().flex_none().size(px(8.0)).rounded_full().bg(color))
                        .child(
                            mono(13.0, TEXT_PRIMARY)
                                .font_weight(FontWeight::BOLD)
                                .flex_1()
                                .min_w(px(0.0))
                                .overflow_hidden()
                                .text_ellipsis()
                                .child(a.name.clone()),
                        )
                        .child(
                            button("btn-dokku-open-console", t!("dokku-btn-console"), TEXT_SECONDARY)
                                .on_click(move |_ev, _w, cx| {
                                    app_console.update(cx, |this, cx| {
                                        this.open_dokku_console(&app_name_console, "web", cx)
                                    })
                                }),
                        )
                        .child(
                            div()
                                .id("btn-dokku-eli5")
                                .size(px(22.0))
                                .flex()
                                .items_center()
                                .justify_center()
                                .text_color(if st.eli5_open { WAND } else { TEXT_DIM })
                                .bg(if st.eli5_open { WAND.opacity(0.14) } else { hex_rgba(0, 0.0) })
                                .cursor_pointer()
                                .hover(|s| s.bg(WAND.opacity(0.14)).text_color(WAND))
                                .on_click(move |_ev, _w, cx| {
                                    app_wand.update(cx, |this, cx| this.explain_dokku_selection(cx))
                                })
                                .child(inherited_icon(TablerIcon::Wand, px(14.0))),
                        )
                        .child(
                            div()
                                .id("btn-dokku-drawer-close")
                                .size(px(22.0))
                                .flex()
                                .items_center()
                                .justify_center()
                                .text_color(TEXT_DIM)
                                .cursor_pointer()
                                .hover(|s| s.bg(BG_ROW_HOVER).text_color(TEXT_PRIMARY))
                                .on_click(move |_ev, _w, cx| {
                                    app_close.update(cx, |this, cx| this.close_dokku_drawer(cx))
                                })
                                .child(inherited_icon(TablerIcon::X, px(14.0))),
                        ),
                )
                .when(st.eli5_open, |d| {
                    d.child(render_eli5(
                        st.eli5_loading.contains(&eli5_key),
                        st.eli5.get(&eli5_key),
                    ))
                })
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .gap(px(6.0))
                        .when(!a.running, |d| {
                            d.child(lifecycle_btn(AppLifecycleAction::Start, t!("dokku-btn-start"), OK))
                        })
                        .when(a.running, |d| {
                            d.child(lifecycle_btn(
                                AppLifecycleAction::Restart,
                                t!("dokku-btn-restart"),
                                WARN,
                            ))
                            .child(lifecycle_btn(AppLifecycleAction::Stop, t!("dokku-btn-stop"), CRIT))
                        })
                        .child(lifecycle_btn(
                            AppLifecycleAction::Rebuild,
                            t!("dokku-btn-rebuild"),
                            TEXT_SECONDARY,
                        ))
                        .child(lifecycle_btn(
                            AppLifecycleAction::MaintenanceToggle(!a.maintenance),
                            if a.maintenance {
                                t!("dokku-btn-maint-disable")
                            } else {
                                t!("dokku-btn-maint-enable")
                            },
                            WARN,
                        ))
                        .child(lifecycle_btn(
                            AppLifecycleAction::Destroy,
                            t!("dokku-btn-destroy"),
                            CRIT,
                        )),
                )
                .children(pending_banner),
        )
        .child(
            div()
                .flex_none()
                .flex()
                .items_center()
                .px(px(8.0))
                .border_b_1()
                .border_color(BORDER_PANEL)
                .child(tab_btn("dokku-tab-overview", t!("dokku-tab-overview"), DokkuDrawerTab::Overview))
                .child(tab_btn("dokku-tab-env", t!("dokku-tab-env"), DokkuDrawerTab::Env))
                .child(tab_btn("dokku-tab-domains", t!("dokku-tab-domains"), DokkuDrawerTab::Domains))
                .child(tab_btn("dokku-tab-services", t!("dokku-tab-services"), DokkuDrawerTab::Services))
                .child(tab_btn("dokku-tab-deploy", t!("dokku-tab-deploy"), DokkuDrawerTab::Deploy))
                .child(tab_btn("dokku-tab-logs", t!("dokku-tab-logs"), DokkuDrawerTab::Logs)),
        )
        .child(
            div()
                .id("dokku-drawer-scroll")
                .flex_1()
                .min_h(px(0.0))
                .overflow_y_scrollbar()
                .p(px(14.0))
                .child(tab_content),
        )
}

fn drawer_overview_tab(
    a: &DokkuApp,
    st: &DokkuState,
    inputs: Option<&DokkuInputs>,
    containers_scan: Option<&ContainerScan>,
    app: Entity<CrowApp>,
) -> impl IntoElement {
    let staged_for_app = st.staged_scale.get(&a.name);
    let mut proc_types: BTreeMap<String, (u32, u32)> = BTreeMap::new();
    for p in &a.processes {
        proc_types.insert(p.proc_type.clone(), (p.running, p.desired));
    }
    if proc_types.is_empty() {
        proc_types.insert("web".to_string(), (0, 0));
        proc_types.insert("worker".to_string(), (0, 0));
    }

    let (app_apply_scale, app_clear_scale, app_mount) = (app.clone(), app.clone(), app.clone());
    let (name_apply, name_clear, name_mount) = (a.name.clone(), a.name.clone(), a.name.clone());

    div()
        .flex()
        .flex_col()
        .gap(px(14.0))
        .children(
            containers_scan
                .and_then(|cs| cs.dokku_app_usage(&a.name))
                .map(|(cpu, mem)| {
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .p(px(10.0))
                        .bg(BG_APP)
                        .border_1()
                        .border_color(BORDER_PANEL)
                        .child(mono(10.0, TEXT_SECONDARY).child(t!("dokku-live-container-usage")))
                        .child(
                            mono(10.5, OK)
                                .font_weight(FontWeight::BOLD)
                                .child(t!(
                                    "dokku-usage-cpu-mem",
                                    cpu = format!("{cpu:.1}"),
                                    mem = human_size(mem)
                                )),
                        )
                }),
        )
        // Process Scaling section
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(8.0))
                .child(
                    mono(10.5, TEXT_PRIMARY)
                        .font_weight(FontWeight::BOLD)
                        .child(t!("dokku-procfile-scaling")),
                )
                .children(proc_types.into_iter().map(|(proc_type, (running, desired))| {
                    let target = staged_for_app
                        .and_then(|m| m.get(&proc_type).copied())
                        .unwrap_or(desired);
                    let modified = target != desired;
                    let (a_dec, a_inc) = (app.clone(), app.clone());
                    let (app1, pt1, app2, pt2) = (
                        a.name.clone(),
                        proc_type.clone(),
                        a.name.clone(),
                        proc_type.clone(),
                    );
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .px(px(10.0))
                        .py(px(7.0))
                        .bg(BG_APP)
                        .border_1()
                        .border_color(if modified { WARN.opacity(0.6) } else { BORDER_PANEL })
                        .child(
                            mono(11.0, TEXT_PRIMARY)
                                .font_weight(FontWeight::BOLD)
                                .w(px(90.0))
                                .child(proc_type.clone()),
                        )
                        .child(
                            mono(10.0, if running == desired { TEXT_SECONDARY } else { WARN })
                                .flex_1()
                                .child(t!(
                                    "dokku-proc-running-desired",
                                    running = running,
                                    desired = desired
                                )),
                        )
                        .child(
                            button(format!("btn-scale-dec-{proc_type}"), "-", TEXT_SECONDARY).on_click(
                                move |_ev, _w, cx| {
                                    a_dec.update(cx, |this, cx| {
                                        this.adjust_dokku_scale(&app1, &pt1, desired, -1, cx)
                                    })
                                },
                            ),
                        )
                        .child(
                            mono(11.0, if modified { WARN } else { TEXT_PRIMARY })
                                .font_weight(FontWeight::BOLD)
                                .w(px(28.0))
                                .child(format!("{target}")),
                        )
                        .child(
                            button(format!("btn-scale-inc-{proc_type}"), "+", TEXT_SECONDARY).on_click(
                                move |_ev, _w, cx| {
                                    a_inc.update(cx, |this, cx| {
                                        this.adjust_dokku_scale(&app2, &pt2, desired, 1, cx)
                                    })
                                },
                            ),
                        )
                }))
                .children(staged_for_app.filter(|m| !m.is_empty()).map(|staged| {
                    let args = staged
                        .iter()
                        .map(|(k, v)| format!("{k}={v}"))
                        .collect::<Vec<_>>()
                        .join(" ");
                    let scale_cmd = format!("dokku ps:scale {} {args}", a.name);
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .p(px(8.0))
                        .bg(WARN.opacity(0.08))
                        .border_1()
                        .border_color(WARN.opacity(0.4))
                        .child(
                            mono(10.0, TEXT_PRIMARY)
                                .flex_1()
                                .child(scale_cmd),
                        )
                        .child(
                            button("btn-dokku-apply-scale", t!("dokku-btn-apply-scale"), OK).on_click(
                                move |_ev, _w, cx| {
                                    app_apply_scale
                                        .update(cx, |this, cx| this.apply_dokku_scale(&name_apply, cx))
                                },
                            ),
                        )
                        .child(
                            button("btn-dokku-reset-scale", t!("dokku-cancel"), TEXT_DIM).on_click(
                                move |_ev, _w, cx| {
                                    app_clear_scale.update(cx, |this, cx| {
                                        this.clear_dokku_staged_scale(&name_clear, cx)
                                    })
                                },
                            ),
                        )
                })),
        )
        // Persistent Host Storage Mounts section (`ERR-207`)
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(8.0))
                .child(
                    mono(10.5, TEXT_PRIMARY)
                        .font_weight(FontWeight::BOLD)
                        .child(t!("dokku-storage-mounts")),
                )
                .when(a.mounts.is_empty(), |d| {
                    d.child(mono(10.0, TEXT_DIM).child(t!("dokku-no-storage-mounts")))
                })
                .children(a.mounts.iter().enumerate().map(|(idx, m)| {
                    let app_rm = app.clone();
                    let (hp_str, cp_str) = m.split_once(':').unwrap_or((m.as_str(), ""));
                    let (aname, hp, cp) = (
                        a.name.clone(),
                        hp_str.to_string(),
                        cp_str.to_string(),
                    );
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .px(px(8.0))
                        .py(px(6.0))
                        .bg(BG_APP)
                        .border_1()
                        .border_color(BORDER_PANEL)
                        .child(
                            mono(10.0, TEXT_SECONDARY)
                                .flex_1()
                                .min_w(px(0.0))
                                .overflow_hidden()
                                .text_ellipsis()
                                .child(format!("{hp_str} → {cp_str}")),
                        )
                        .child(
                            button(
                                format!("btn-dokku-unmount-{idx}"),
                                t!("dokku-btn-unmount"),
                                CRIT,
                            )
                            .on_click(move |_ev, _w, cx| {
                                app_rm.update(cx, |this, cx| {
                                    this.remove_dokku_storage_mount(&aname, &hp, &cp, cx)
                                })
                            }),
                        )
                }))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        .child(text_input(inputs.map(|i| &i.mount_host)))
                        .child(mono(10.0, TEXT_DIM).child("→"))
                        .child(text_input(inputs.map(|i| &i.mount_container)))
                        .child(
                            button("btn-dokku-add-mount", t!("dokku-btn-mount"), OK).on_click(
                                move |_ev, _w, cx| {
                                    app_mount.update(cx, |this, cx| {
                                        this.add_dokku_storage_mount(&name_mount, cx)
                                    })
                                },
                            ),
                        ),
                ),
        )
}

fn drawer_env_tab(
    a: &DokkuApp,
    st: &DokkuState,
    inputs: Option<&DokkuInputs>,
    app: Entity<CrowApp>,
) -> impl IntoElement {
    let staged_set = st.staged_env_set.get(&a.name);
    let staged_unset = st.staged_env_unset.get(&a.name);
    let has_staged = staged_set.is_some_and(|m| !m.is_empty())
        || staged_unset.is_some_and(|s| !s.is_empty());

    let (a_ref, a_paste, a_restart, a_add, a_blob, a_apply, a_discard) = (
        app.clone(),
        app.clone(),
        app.clone(),
        app.clone(),
        app.clone(),
        app.clone(),
        app.clone(),
    );
    let (n_ref, n_add, n_blob, n_apply, n_discard) = (
        a.name.clone(),
        a.name.clone(),
        a.name.clone(),
        a.name.clone(),
        a.name.clone(),
    );

    let live_env = st.env_cache.get(&a.name).and_then(|r| r.as_ref().ok());
    let mut all_keys: BTreeMap<String, String> = live_env.cloned().unwrap_or_default();
    if let Some(s) = staged_set {
        for (k, v) in s {
            all_keys.insert(k.clone(), v.clone());
        }
    }

    let linked_vars: HashSet<&str> = ServiceKind::ALL.iter().map(|k| k.env_var()).collect();

    div()
        .flex()
        .flex_col()
        .gap(px(10.0))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(6.0))
                .child(
                    chip(
                        "btn-dokku-env-paste-toggle",
                        t!("dokku-btn-dotenv-paste"),
                        st.dotenv_paste_open,
                    )
                    .on_click(move |_ev, _w, cx| {
                        a_paste.update(cx, |this, cx| this.toggle_dokku_dotenv_paste(cx))
                    }),
                )
                .child(
                    chip(
                        "btn-dokku-env-restart-toggle",
                        if st.restart_on_env_apply {
                            t!("dokku-env-restart-yes")
                        } else {
                            t!("dokku-env-restart-no")
                        },
                        st.restart_on_env_apply,
                    )
                    .on_click(move |_ev, _w, cx| {
                        a_restart.update(cx, |this, cx| this.toggle_dokku_restart_on_env(cx))
                    }),
                )
                .child(div().flex_1())
                .child(
                    button("btn-dokku-env-refresh", t!("dokku-refresh"), TEXT_SECONDARY).on_click(
                        move |_ev, _w, cx| {
                            a_ref.update(cx, |this, cx| this.load_dokku_app_env(&n_ref, true, cx))
                        },
                    ),
                ),
        )
        // Staged diff apply banner
        .when(has_staged, |d| {
            let set_count = staged_set.map(BTreeMap::len).unwrap_or(0);
            let unset_count = staged_unset.map(HashSet::len).unwrap_or(0);
            d.child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .p(px(8.0))
                    .bg(WARN.opacity(0.08))
                    .border_1()
                    .border_color(WARN.opacity(0.4))
                    .child(
                        mono(10.0, TEXT_PRIMARY).flex_1().child(t!(
                            "dokku-env-staged-summary",
                            set = set_count,
                            unset = unset_count
                        )),
                    )
                    .child(
                        button("btn-dokku-env-apply", t!("dokku-btn-apply-env"), OK).on_click(
                            move |_ev, _w, cx| {
                                a_apply.update(cx, |this, cx| this.apply_dokku_staged_env(&n_apply, cx))
                            },
                        ),
                    )
                    .child(
                        button("btn-dokku-env-discard", t!("dokku-cancel"), TEXT_DIM).on_click(
                            move |_ev, _w, cx| {
                                a_discard
                                    .update(cx, |this, cx| this.clear_dokku_staged_env(&n_discard, cx))
                            },
                        ),
                    ),
            )
        })
        // Add KEY=VALUE row
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(6.0))
                .child(div().w(px(150.0)).child(text_input(inputs.map(|i| &i.env_key))))
                .child(text_input(inputs.map(|i| &i.env_val)))
                .child(
                    button("btn-dokku-env-stage", t!("dokku-btn-stage"), OK).on_click(
                        move |_ev, w, cx| {
                            a_add.update(cx, |this, cx| this.stage_dokku_env_pair(&n_add, w, cx))
                        },
                    ),
                ),
        )
        // Optional .env bulk import row
        .when(st.dotenv_paste_open, |d| {
            d.child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.0))
                    .child(text_input(inputs.map(|i| &i.dotenv_blob)))
                    .child(
                        button("btn-dokku-env-stage-blob", t!("dokku-btn-import-dotenv"), OK)
                            .on_click(move |_ev, w, cx| {
                                a_blob.update(cx, |this, cx| this.stage_dokku_dotenv_blob(&n_blob, w, cx))
                            }),
                    ),
            )
        })
        // Env list
        .child(if st.env_loading.contains(&a.name) {
            mono(10.5, TEXT_DIM)
                .child(t!("dokku-reading-env"))
                .into_any_element()
        } else if let Some(Err(e)) = st.env_cache.get(&a.name) {
            mono(10.5, CRIT).child(e.clone()).into_any_element()
        } else if all_keys.is_empty() {
            mono(10.5, TEXT_DIM)
                .child(t!("dokku-no-env-vars"))
                .into_any_element()
        } else {
            div()
                .flex()
                .flex_col()
                .border_t_1()
                .border_color(BORDER_PANEL)
                .children(all_keys.into_iter().map(|(k, v)| {
                    let is_staged_set = staged_set.is_some_and(|m| m.contains_key(&k));
                    let is_staged_unset = staged_unset.is_some_and(|s| s.contains(&k));
                    let is_revealed = st.env_revealed.contains(&format!("{}:{k}", a.name));
                    let is_service_var = linked_vars.contains(k.as_str());
                    let display_val = if is_revealed {
                        v.clone()
                    } else {
                        "••••••••".to_string()
                    };
                    let (a_rev, a_un) = (app.clone(), app.clone());
                    let (an1, k1, an2, k2) = (a.name.clone(), k.clone(), a.name.clone(), k.clone());
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .px(px(8.0))
                        .py(px(6.0))
                        .border_b_1()
                        .border_l_1()
                        .border_r_1()
                        .border_color(BORDER_PANEL)
                        .bg(if is_staged_unset {
                            CRIT.opacity(0.06)
                        } else if is_staged_set {
                            OK.opacity(0.06)
                        } else {
                            BG_APP
                        })
                        .child(
                            mono(
                                10.0,
                                if is_staged_unset {
                                    CRIT
                                } else if is_staged_set {
                                    OK
                                } else {
                                    TEXT_PRIMARY
                                },
                            )
                            .font_weight(FontWeight::BOLD)
                            .w(px(160.0))
                            .overflow_hidden()
                            .text_ellipsis()
                            .child(k.clone()),
                        )
                        .when(is_service_var, |d| {
                            d.child(
                                crate::components::button::badge(WARN)
                                    .text_size(px(8.0))
                                    .child(t!("dokku-badge-linked")),
                            )
                        })
                        .child(
                            mono(10.0, TEXT_SECONDARY)
                                .flex_1()
                                .min_w(px(0.0))
                                .overflow_hidden()
                                .text_ellipsis()
                                .child(display_val),
                        )
                        .child(
                            button(
                                format!("btn-dokku-env-rev-{k}"),
                                if is_revealed {
                                    t!("dokku-btn-hide")
                                } else {
                                    t!("dokku-btn-show")
                                },
                                TEXT_DIM,
                            )
                            .on_click(move |_ev, _w, cx| {
                                a_rev.update(cx, |this, cx| this.toggle_dokku_env_reveal(&an1, &k1, cx))
                            }),
                        )
                        .child(
                            button(
                                format!("btn-dokku-env-unset-{k}"),
                                t!("dokku-btn-unset"),
                                CRIT,
                            )
                            .on_click(move |_ev, _w, cx| {
                                a_un.update(cx, |this, cx| this.stage_dokku_env_unset(&an2, &k2, cx))
                            }),
                        )
                }))
                .into_any_element()
        })
}

fn drawer_domains_tab(
    a: &DokkuApp,
    inputs: Option<&DokkuInputs>,
    app: Entity<CrowApp>,
) -> impl IntoElement {
    let (a_le, a_dom, a_port) = (app.clone(), app.clone(), app.clone());
    let (n_le, n_dom, n_port) = (a.name.clone(), a.name.clone(), a.name.clone());

    div()
        .flex()
        .flex_col()
        .gap(px(14.0))
        // Let's Encrypt TLS section
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .p(px(10.0))
                .bg(BG_APP)
                .border_1()
                .border_color(BORDER_PANEL)
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(2.0))
                        .child(
                            mono(10.5, if a.ssl_enabled { OK } else { WARN })
                                .font_weight(FontWeight::BOLD)
                                .child(if a.ssl_enabled {
                                    t!("dokku-tls-enabled")
                                } else {
                                    t!("dokku-tls-disabled")
                                }),
                        )
                        .children(
                            a.letsencrypt_expiry_days
                                .map(|days| mono(9.5, TEXT_DIM).child(format!("{days}d"))),
                        ),
                )
                .child(
                    button(
                        "btn-dokku-enable-le",
                        if a.ssl_enabled {
                            t!("dokku-btn-renew-le")
                        } else {
                            t!("dokku-btn-enable-le")
                        },
                        OK,
                    )
                    .on_click(move |_ev, _w, cx| {
                        a_le.update(cx, |this, cx| this.enable_dokku_letsencrypt(&n_le, cx))
                    }),
                ),
        )
        // Domains list
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(6.0))
                .child(
                    mono(10.5, TEXT_PRIMARY)
                        .font_weight(FontWeight::BOLD)
                        .child(t!("dokku-domains-heading")),
                )
                .when(a.domains.is_empty(), |d| {
                    d.child(mono(10.0, TEXT_DIM).child(t!("dokku-no-domains")))
                })
                .children(a.domains.iter().map(|dom| {
                    let (a_chk, a_rm) = (app.clone(), app.clone());
                    let (an1, d1, an2, d2) = (
                        a.name.clone(),
                        dom.clone(),
                        a.name.clone(),
                        dom.clone(),
                    );
                    let https = a.ssl_enabled;
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .px(px(8.0))
                        .py(px(6.0))
                        .bg(BG_APP)
                        .border_1()
                        .border_color(BORDER_PANEL)
                        .child(mono(10.5, TEXT_PRIMARY).flex_1().child(dom.clone()))
                        .child(
                            button(
                                format!("btn-dokku-watch-dom-{dom}"),
                                t!("dokku-btn-watch-checks"),
                                TEXT_SECONDARY,
                            )
                            .on_click(move |_ev, _w, cx| {
                                a_chk.update(cx, |this, cx| {
                                    this.watch_dokku_domain_in_checks(&an1, &d1, https, cx)
                                })
                            }),
                        )
                        .child(
                            button(
                                format!("btn-dokku-rm-dom-{dom}"),
                                t!("dokku-btn-remove"),
                                CRIT,
                            )
                            .on_click(move |_ev, _w, cx| {
                                a_rm.update(cx, |this, cx| this.remove_dokku_domain(&an2, &d2, cx))
                            }),
                        )
                }))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        .child(text_input(inputs.map(|i| &i.domain_add)))
                        .child(
                            button("btn-dokku-add-dom", t!("dokku-btn-add-domain"), OK).on_click(
                                move |_ev, _w, cx| {
                                    a_dom.update(cx, |this, cx| this.add_dokku_domain(&n_dom, cx))
                                },
                            ),
                        ),
                ),
        )
        // Proxy Port Mappings
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(6.0))
                .child(
                    mono(10.5, TEXT_PRIMARY)
                        .font_weight(FontWeight::BOLD)
                        .child(t!("dokku-ports-heading")),
                )
                .children(a.ports.iter().map(|p| {
                    let spec = p.spec();
                    let a_rm = app.clone();
                    let (an, sp) = (a.name.clone(), spec.clone());
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .px(px(8.0))
                        .py(px(6.0))
                        .bg(BG_APP)
                        .border_1()
                        .border_color(BORDER_PANEL)
                        .child(
                            mono(10.0, TEXT_PRIMARY)
                                .flex_1()
                                .child(format!("{} : {} → {}", p.scheme, p.host_port, p.container_port)),
                        )
                        .child(
                            button(
                                format!("btn-dokku-rm-port-{spec}"),
                                t!("dokku-btn-remove"),
                                CRIT,
                            )
                            .on_click(move |_ev, _w, cx| {
                                a_rm.update(cx, |this, cx| this.remove_dokku_port_map(&an, &sp, cx))
                            }),
                        )
                }))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        .child(text_input(inputs.map(|i| &i.port_add)))
                        .child(
                            button("btn-dokku-add-port", t!("dokku-btn-add-port"), OK).on_click(
                                move |_ev, _w, cx| {
                                    a_port.update(cx, |this, cx| this.add_dokku_port_map(&n_port, cx))
                                },
                            ),
                        ),
                ),
        )
}

fn drawer_services_tab(
    a: &DokkuApp,
    scan: &DokkuScan,
    st: &DokkuState,
    app: Entity<CrowApp>,
) -> impl IntoElement {
    let app_toggle = app.clone();
    div()
        .flex()
        .flex_col()
        .gap(px(10.0))
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    mono(10.5, TEXT_PRIMARY)
                        .font_weight(FontWeight::BOLD)
                        .child(t!("dokku-backing-services-for-app")),
                )
                .child(
                    chip(
                        "btn-dokku-link-restart-toggle",
                        if st.restart_on_link {
                            t!("dokku-link-restart-yes")
                        } else {
                            t!("dokku-link-restart-no")
                        },
                        st.restart_on_link,
                    )
                    .on_click(move |_ev, _w, cx| {
                        app_toggle.update(cx, |this, cx| this.toggle_dokku_restart_on_link(cx))
                    }),
                ),
        )
        .when(scan.services.is_empty(), |d| {
            d.child(mono(10.0, TEXT_DIM).child(t!("dokku-no-services-yet")))
        })
        .children(scan.services.iter().map(|svc| {
            let is_linked = a.linked_services.contains(&(svc.kind, svc.name.clone()));
            let app_act = app.clone();
            let (kind, sname, aname) = (svc.kind, svc.name.clone(), a.name.clone());
            div()
                .flex()
                .items_center()
                .gap(px(8.0))
                .px(px(10.0))
                .py(px(8.0))
                .bg(BG_APP)
                .border_1()
                .border_color(if is_linked { OK.opacity(0.5) } else { BORDER_PANEL })
                .child(
                    crate::components::button::badge(TEXT_SECONDARY)
                        .text_size(px(8.5))
                        .child(svc.kind.label()),
                )
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .flex_col()
                        .gap(px(2.0))
                        .child(
                            mono(10.5, TEXT_PRIMARY)
                                .font_weight(FontWeight::BOLD)
                                .child(svc.name.clone()),
                        )
                        .child(
                            mono(9.5, TEXT_FAINT)
                                .child(format!("{} · {}", svc.kind.env_var(), svc.status)),
                        ),
                )
                .child(
                    button(
                        format!("btn-dokku-link-{}-{}", svc.kind.id(), svc.name),
                        if is_linked {
                            t!("dokku-btn-unlink")
                        } else {
                            t!("dokku-btn-link")
                        },
                        if is_linked { CRIT } else { OK },
                    )
                    .on_click(move |_ev, _w, cx| {
                        app_act.update(cx, |this, cx| {
                            this.request_dokku_action(
                                PendingDokkuAction::Link {
                                    kind,
                                    service: sname.clone(),
                                    app: aname.clone(),
                                    link: !is_linked,
                                },
                                cx,
                            )
                        })
                    }),
                )
        }))
}

fn drawer_deploy_tab(
    a: &DokkuApp,
    inputs: Option<&DokkuInputs>,
    server_host: &str,
    app: Entity<CrowApp>,
) -> impl IntoElement {
    let git_remote = format!("dokku@{server_host}:{}", a.name);
    let git_cmd = format!("git remote add dokku {git_remote} && git push dokku main");
    let git_cmd_copy = git_cmd.clone();
    let (a_img, a_keys) = (app.clone(), app.clone());
    let n_img = a.name.clone();

    div()
        .flex()
        .flex_col()
        .gap(px(14.0))
        // Git Push deploy box
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(6.0))
                .p(px(10.0))
                .bg(BG_APP)
                .border_1()
                .border_color(BORDER_PANEL)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            mono(10.5, TEXT_PRIMARY)
                                .font_weight(FontWeight::BOLD)
                                .child(t!("dokku-deploy-git-push")),
                        )
                        .child(
                            button("btn-dokku-copy-git", t!("dokku-btn-copy-git"), OK).on_click(
                                move |_ev, _w, cx| {
                                    cx.write_to_clipboard(ClipboardItem::new_string(
                                        git_cmd_copy.clone(),
                                    ));
                                },
                            ),
                        ),
                )
                .child(mono(10.0, TEXT_SECONDARY).child(git_cmd))
                .children((!a.git_sha.is_empty()).then(|| {
                    mono(9.5, TEXT_FAINT).child(format!(
                        "{} · {}",
                        a.git_sha,
                        a.last_updated
                    ))
                }))
                .child(
                    div().pt(px(4.0)).child(
                        button(
                            "btn-dokku-deploy-manage-keys",
                            t!("dokku-btn-manage-deploy-keys"),
                            TEXT_SECONDARY,
                        )
                        .on_click(move |_ev, w, cx| {
                            a_keys.update(cx, |this, cx| {
                                this.toggle_dokku_top_panel(DokkuTopPanel::DeployKeys, w, cx)
                            })
                        }),
                    ),
                ),
        )
        // Deploy from Docker/OCI Image (`dokku git:from-image`)
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(6.0))
                .p(px(10.0))
                .bg(BG_APP)
                .border_1()
                .border_color(BORDER_PANEL)
                .child(
                    mono(10.5, TEXT_PRIMARY)
                        .font_weight(FontWeight::BOLD)
                        .child(t!("dokku-deploy-from-image")),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        .child(text_input(inputs.map(|i| &i.deploy_image)))
                        .child(
                            button("btn-dokku-deploy-image-go", t!("dokku-btn-deploy-image"), OK)
                                .on_click(move |_ev, _w, cx| {
                                    a_img.update(cx, |this, cx| {
                                        this.deploy_dokku_image(&n_img, cx)
                                    })
                                }),
                        ),
                ),
        )
}

fn drawer_logs_tab(a: &DokkuApp, st: &DokkuState, app: Entity<CrowApp>) -> impl IntoElement {
    let sel_key = format!("app:{}", a.name);
    let app_follow = app.clone();
    let app_all = app.clone();

    div()
        .flex()
        .flex_col()
        .gap(px(8.0))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(6.0))
                .child(
                    chip(
                        "btn-dokku-log-proc-all",
                        t!("dokku-logs-all"),
                        st.log_process_filter.is_none(),
                    )
                    .on_click(move |_ev, _w, cx| {
                        app_all.update(cx, |this, cx| this.set_dokku_log_filter(None, cx))
                    }),
                )
                .children(a.processes.iter().map(|p| {
                    let pt = p.proc_type.clone();
                    let active = st.log_process_filter.as_deref() == Some(pt.as_str());
                    let app_p = app.clone();
                    chip(format!("btn-dokku-log-proc-{pt}"), pt.clone(), active).on_click(
                        move |_ev, _w, cx| {
                            app_p.update(cx, |this, cx| {
                                this.set_dokku_log_filter(Some(pt.clone()), cx)
                            })
                        },
                    )
                }))
                .child(div().flex_1())
                .child(
                    div()
                        .id("btn-dokku-log-follow")
                        .flex()
                        .items_center()
                        .gap(px(5.0))
                        .cursor_pointer()
                        .on_click(move |_ev, _w, cx| {
                            app_follow.update(cx, |this, cx| this.toggle_dokku_follow(cx))
                        })
                        .child(
                            div()
                                .size(px(6.0))
                                .rounded_full()
                                .bg(if st.follow { OK } else { TEXT_GHOST }),
                        )
                        .child(
                            mono(9.5, if st.follow { OK } else { TEXT_DIM })
                                .font_weight(FontWeight::BOLD)
                                .child(if st.follow {
                                    t!("containers-following")
                                } else {
                                    t!("containers-follow")
                                }),
                        ),
                ),
        )
        .child(render_log_lines(&st.logs, &sel_key))
}

fn render_log_lines(
    logs: &Option<(String, Result<String, String>)>,
    expected_key: &str,
) -> AnyElement {
    match logs {
        Some((k, Ok(text))) if k == expected_key => {
            let lines: Vec<&str> = text.lines().collect();
            if lines.is_empty() {
                mono(10.5, TEXT_DIM)
                    .child(t!("containers-no-output-yet"))
                    .into_any_element()
            } else {
                div()
                    .flex()
                    .flex_col()
                    .children(lines.into_iter().rev().map(|l| {
                        mono(10.0, TEXT_SECONDARY)
                            .py(px(1.0))
                            .child(l.to_string())
                    }))
                    .into_any_element()
            }
        }
        Some((k, Err(e))) if k == expected_key => {
            mono(10.5, CRIT).child(e.clone()).into_any_element()
        }
        _ => mono(10.5, TEXT_DIM)
            .child(t!("containers-reading-logs"))
            .into_any_element(),
    }
}

// =============================================================================
// Backing Service Inspector Drawer (`ERR-207`)
// =============================================================================

fn service_drawer(
    svc: &BackingService,
    scan: &DokkuScan,
    st: &DokkuState,
    containers_scan: Option<&ContainerScan>,
    app: Entity<CrowApp>,
) -> impl IntoElement {
    let running = svc.status == "running";
    let color = if running { OK } else { TEXT_DIM };
    let sel_key = format!("svc:{}:{}", svc.kind.id(), svc.name);
    let (app_close, app_follow) = (app.clone(), app.clone());

    let svc_btn = |action: ServiceLifecycleAction, label: &'static str, c: Rgba| {
        let (app_ent, kind, name) = (app.clone(), svc.kind, svc.name.clone());
        button(
            format!("btn-dokku-svc-{}-{}-{:?}", svc.kind.id(), svc.name, action),
            label,
            c,
        )
        .on_click(move |_ev, _w, cx| {
            app_ent.update(cx, |this, cx| {
                this.request_dokku_action(
                    PendingDokkuAction::Service {
                        kind,
                        name: name.clone(),
                        action,
                    },
                    cx,
                )
            })
        })
    };

    let pending_banner = match &st.pending {
        Some(PendingDokkuAction::Service { kind, name, action })
            if *kind == svc.kind && name == &svc.name =>
        {
            let cmd = match action {
                ServiceLifecycleAction::Export => {
                    format!("dokku {}:export {} > /var/backups/...", kind.id(), svc.name)
                }
                ServiceLifecycleAction::Destroy => {
                    format!("dokku --force {}:destroy {}", kind.id(), svc.name)
                }
                ServiceLifecycleAction::Start => format!("dokku {}:start {}", kind.id(), svc.name),
                ServiceLifecycleAction::Stop => format!("dokku {}:stop {}", kind.id(), svc.name),
                ServiceLifecycleAction::Restart => {
                    format!("dokku {}:restart {}", kind.id(), svc.name)
                }
            };
            let q = match action {
                ServiceLifecycleAction::Export => t!("dokku-confirm-export-service"),
                ServiceLifecycleAction::Destroy => t!("dokku-confirm-destroy-service"),
                _ => t!("dokku-confirm-service-action"),
            };
            Some(confirm_banner(q, cmd, app.clone()))
        }
        Some(PendingDokkuAction::Link { kind, service, app: target_app, link })
            if *kind == svc.kind && service == &svc.name =>
        {
            let verb = if *link { "link" } else { "unlink" };
            let cmd = format!("dokku {}:{verb} {} {target_app}", kind.id(), svc.name);
            let q = if *link {
                t!("dokku-confirm-link-service", env_var = kind.env_var(), app = target_app)
            } else {
                t!("dokku-confirm-unlink-service", env_var = kind.env_var(), app = target_app)
            };
            Some(confirm_banner(q, cmd, app.clone()))
        }
        _ => None,
    };

    div()
        .w(px(520.0))
        .flex_none()
        .h_full()
        .flex()
        .flex_col()
        .bg(BG_PANEL)
        .border_l_1()
        .border_color(BORDER_PANEL)
        .child(
            div()
                .flex_none()
                .flex()
                .flex_col()
                .gap(px(8.0))
                .p(px(14.0))
                .border_b_1()
                .border_color(BORDER_PANEL)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .child(div().flex_none().size(px(8.0)).rounded_full().bg(color))
                        .child(
                            crate::components::button::badge(TEXT_SECONDARY)
                                .text_size(px(9.0))
                                .child(svc.kind.label()),
                        )
                        .child(
                            mono(13.0, TEXT_PRIMARY)
                                .font_weight(FontWeight::BOLD)
                                .flex_1()
                                .child(svc.name.clone()),
                        )
                        .child(
                            div()
                                .id("btn-dokku-svc-drawer-close")
                                .size(px(22.0))
                                .flex()
                                .items_center()
                                .justify_center()
                                .text_color(TEXT_DIM)
                                .cursor_pointer()
                                .hover(|s| s.bg(BG_ROW_HOVER).text_color(TEXT_PRIMARY))
                                .on_click(move |_ev, _w, cx| {
                                    app_close.update(cx, |this, cx| this.close_dokku_drawer(cx))
                                })
                                .child(inherited_icon(TablerIcon::X, px(14.0))),
                        ),
                )
                .child(
                    mono(10.0, TEXT_TERTIARY).child(format!(
                        "{} · {} · {}",
                        svc.kind.env_var(),
                        svc.status,
                        if svc.version.is_empty() {
                            svc.kind.label()
                        } else {
                            &svc.version
                        }
                    )),
                )
                .children(
                    containers_scan
                        .and_then(|cs| cs.dokku_service_usage(svc.kind.id(), &svc.name))
                        .map(|(cpu, mem)| {
                            mono(10.0, OK).child(t!(
                                "dokku-usage-cpu-mem",
                                cpu = format!("{cpu:.1}"),
                                mem = human_size(mem)
                            ))
                        }),
                )
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .gap(px(6.0))
                        .when(!running, |d| {
                            d.child(svc_btn(ServiceLifecycleAction::Start, t!("dokku-btn-start"), OK))
                        })
                        .when(running, |d| {
                            d.child(svc_btn(
                                ServiceLifecycleAction::Restart,
                                t!("dokku-btn-restart"),
                                WARN,
                            ))
                            .child(svc_btn(ServiceLifecycleAction::Stop, t!("dokku-btn-stop"), CRIT))
                        })
                        .child(svc_btn(
                            ServiceLifecycleAction::Export,
                            t!("dokku-btn-export-dump"),
                            TEXT_SECONDARY,
                        ))
                        .child(svc_btn(
                            ServiceLifecycleAction::Destroy,
                            t!("dokku-btn-destroy"),
                            CRIT,
                        )),
                )
                .children(pending_banner),
        )
        .child(
            div()
                .id("dokku-svc-drawer-scroll")
                .flex_1()
                .min_h(px(0.0))
                .overflow_y_scrollbar()
                .p(px(14.0))
                .flex()
                .flex_col()
                .gap(px(14.0))
                // Linked apps section
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(6.0))
                        .child(
                            mono(10.5, TEXT_PRIMARY)
                                .font_weight(FontWeight::BOLD)
                                .child(t!("dokku-col-linked-apps")),
                        )
                        .children(scan.apps.iter().map(|a| {
                            let is_linked = svc.linked_apps.contains(&a.name);
                            let app_act = app.clone();
                            let (kind, sname, aname) = (svc.kind, svc.name.clone(), a.name.clone());
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .px(px(8.0))
                                .py(px(6.0))
                                .bg(BG_APP)
                                .border_1()
                                .border_color(BORDER_PANEL)
                                .child(mono(10.5, TEXT_PRIMARY).child(a.name.clone()))
                                .child(
                                    button(
                                        format!("btn-svc-link-app-{}", a.name),
                                        if is_linked {
                                            t!("dokku-btn-unlink")
                                        } else {
                                            t!("dokku-btn-link")
                                        },
                                        if is_linked { CRIT } else { OK },
                                    )
                                    .on_click(move |_ev, _w, cx| {
                                        app_act.update(cx, |this, cx| {
                                            this.request_dokku_action(
                                                PendingDokkuAction::Link {
                                                    kind,
                                                    service: sname.clone(),
                                                    app: aname.clone(),
                                                    link: !is_linked,
                                                },
                                                cx,
                                            )
                                        })
                                    }),
                                )
                        })),
                )
                // Service Logs section
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(6.0))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .child(
                                    mono(10.5, TEXT_PRIMARY)
                                        .font_weight(FontWeight::BOLD)
                                        .child(t!("dokku-tab-logs")),
                                )
                                .child(
                                    div()
                                        .id("btn-dokku-svc-log-follow")
                                        .flex()
                                        .items_center()
                                        .gap(px(5.0))
                                        .cursor_pointer()
                                        .on_click(move |_ev, _w, cx| {
                                            app_follow.update(cx, |this, cx| this.toggle_dokku_follow(cx))
                                        })
                                        .child(
                                            div()
                                                .size(px(6.0))
                                                .rounded_full()
                                                .bg(if st.follow { OK } else { TEXT_GHOST }),
                                        )
                                        .child(
                                            mono(9.5, if st.follow { OK } else { TEXT_DIM })
                                                .font_weight(FontWeight::BOLD)
                                                .child(if st.follow {
                                                    t!("containers-following")
                                                } else {
                                                    t!("containers-follow")
                                                }),
                                        ),
                                ),
                        )
                        .child(render_log_lines(&st.logs, &sel_key)),
                ),
        )
}
