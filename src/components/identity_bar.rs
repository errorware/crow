use gpui_kit::*;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::component::scroll::ScrollableElement;
use crate::theme::*;
use crate::app::CrowApp;
use crate::host::{connection_state, transport_kind, ConnectionState, TransportKind};
use crate::vault::ServerRecord;
use crate::components::icons::{TablerIcon, tabler_icon};
use crate::os_detect::{classify_distro_family, DistroFamily};

/// The jump host a server is reached through: its name, and whether it's
/// down right now (ERR-91).
pub type JumpInfo = Option<(String, bool)>;

pub fn identity_bar(server: Option<&ServerRecord>, jump: JumpInfo, region_picker_open: bool, snapshots: bool, app: Entity<CrowApp>) -> impl IntoElement {
    let app_clone = app.clone();

    let server_name = server.map(|s| s.name.as_str()).unwrap_or("localhost");
    let endpoint_str = server
        .map(|s| format!("{}@{}:{}", s.login_user, s.host, s.port))
        .unwrap_or_else(|| "operator@127.0.0.1:22".to_string());

    // Live transport state: this machine needs no connection; SSH servers
    // report what their last command saw.
    let (status_color, status_text, status_detail): (Rgba, &str, Option<String>) = match server {
        None => (OK, "LOCAL", None),
        Some(s) => match transport_kind(s) {
            TransportKind::Local => (OK, "LOCAL", None),
            TransportKind::Container => (OK, "LAB CONTAINER", None),
            TransportKind::Ssh => match connection_state(&s.id) {
                None => (TEXT_FAINT, "CONNECTING…", None),
                Some(ConnectionState::Connected) => (OK, "CONNECTED", None),
                Some(state) => (CRIT, state.label(), state.detail().map(str::to_string)),
            },
        },
    };

    // Facts Crow has; placeholders ("-", "—") count as not known (enrolled before they were read, or imported).
    use crate::app::server_facts::unknown_fact;
    let known = |v: &str| (!unknown_fact(v)).then(|| v.trim().to_string());
    let distro = server.and_then(|s| known(&s.os_distro));
    let distro_family = distro.as_deref().map(classify_distro_family).unwrap_or(DistroFamily::Unknown);
    let kernel = server.and_then(|s| known(&s.os_kernel));
    let role = server.and_then(|s| known(&s.role));
    let env_str = server.and_then(|s| known(&s.env)).unwrap_or_else(|| "local".to_string());

    div()
        .h(px(52.0))
        .flex_none()
        .flex()
        .items_center()
        .bg(BG_PANEL)
        .border_b_1()
        .border_color(BORDER_PANEL)
        // 1. Server identity, with where it lives (ERR-36)
        .child(
            div()
                .relative()
                .flex()
                .flex_col()
                .gap(px(2.0))
                .px(px(16.0))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .child(if let Some(s) = server.filter(|s| !s.region_country.is_empty()) {
                            crate::components::flag::flag(&s.region_country, 12.0)
                        } else {
                            tabler_icon(TablerIcon::Server)
                                .size(px(14.0))
                                .text_color(TEXT_DIM)
                                .into_any_element()
                        })
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(15.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(TEXT_PRIMARY)
                                .child(server_name.to_string()),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .font_family("JetBrains Mono")
                        .text_size(px(9.5))
                        .child(div().text_color(TEXT_DIM).child(endpoint_str))
                        .children(jump.map(|(name, down)| {
                            div()
                                .text_color(if down { CRIT } else { TEXT_DIM })
                                .child(if down { format!("via {name} (unreachable)") } else { format!("via {name}") })
                        }))
                        .children(server.map(|s| {
                            let app = app.clone();
                            div()
                                .id("identity-region")
                                .text_color(TEXT_DIM)
                                .cursor_pointer()
                                .hover(|h| h.text_color(TEXT_PRIMARY))
                                .on_click(move |_ev, _window, cx| app.update(cx, |this, cx| this.toggle_region_picker(cx)))
                                .child(format!("· {}", region_label(s)))
                        })),
                )
                .children(server.filter(|_| region_picker_open).map(|s| deferred(region_picker(s, app.clone())).with_priority(1))),
        )
        // 2. Connection state: the dot carries the color.
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(7.0))
                .pr(px(16.0))
                .h_full()
                .border_r_1()
                .border_color(BORDER_PANEL)
                .child(div().size(px(6.0)).rounded_full().bg(status_color).flex_none())
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(10.5))
                        .text_color(if status_color == CRIT { CRIT } else { TEXT_SECONDARY })
                        .child(status_text.to_lowercase()),
                )
                .children(status_detail.map(|d| div().max_w(px(320.0)).overflow_hidden().font_family(FONT_MONO).text_size(px(9.5)).text_color(TEXT_FAINT).child(d))),
        )
        // 3. What the server is, in one line. Only the environment has color
        //    (outlined, for prod and staging); facts Crow doesn't have yet
        //    are left out rather than shown as "-".
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(12.0))
                .px(px(16.0))
                .h_full()
                .min_w(px(0.0))
                .overflow_hidden()
                .border_r_1()
                .border_color(BORDER_PANEL)
                .font_family(FONT_MONO)
                .text_size(px(10.5))
                .child({
                    let (fg, outlined) = match env_str.to_uppercase().as_str() {
                        "PROD" => (CRIT, true),
                        "STAGE" | "STAGING" => (WARN, true),
                        _ => (TEXT_DIM, false),
                    };
                    div()
                        .flex_none()
                        .px(px(5.0))
                        .py(px(1.0))
                        .when(outlined, |d| d.border_1().border_color(fg))
                        .text_size(px(9.5))
                        .font_weight(FontWeight::BOLD)
                        .text_color(fg)
                        .child(env_str.to_uppercase())
                })
                .children(distro.map(|d| {
                    div()
                        .flex()
                        .flex_none()
                        .items_center()
                        .gap(px(6.0))
                        .child(div().text_color(TEXT_SECONDARY).child(d))
                        .when(!distro_family.is_supported(), |el| el.child(div().text_size(px(9.5)).text_color(WARN).child("unverified")))
                }))
                .children(kernel.map(|k| fact("kernel", k)))
                .children(role.map(|r| fact("role", r)))
                .children(server.map(|s| {
                    let app = app.clone();
                    let s_id = s.id.clone();
                    let none = crate::app::groups::is_ungrouped(&s.group_name);
                    div()
                        .id("identity-group-chip")
                        .flex()
                        .flex_none()
                        .items_center()
                        .gap(px(5.0))
                        .cursor_pointer()
                        .hover(|h| h.text_color(TEXT_PRIMARY))
                        .text_color(if none { TEXT_DIM } else { TEXT_SECONDARY })
                        .on_click(move |_ev, _window, cx| app.update(cx, |this, cx| this.set_group_assign_target(Some(s_id.clone()), cx)))
                        .child(div().text_color(TEXT_FAINT).child("group"))
                        .child(if none { "none ▾".to_string() } else { format!("{} ▾", s.group_name.trim()) })
                })),
        )
        // 4. Spacer
        .child(div().flex_1())
        // 5. Command trigger
        .child(
            div()
                .id("cmd-palette-trigger")
                .flex()
                .items_center()
                .gap(px(7.0))
                .h(px(26.0))
                .px(px(8.0))
                .mr(px(8.0))
                .border_1()
                .border_color(BORDER_DEFAULT)
                .bg(BG_APP)
                .cursor_pointer()
                .hover(|s| s.border_color(BORDER_STRONG))
                .on_click(move |_ev, _window, cx| app_clone.update(cx, |this, cx| this.toggle_palette(cx)))
                .child(tabler_icon(TablerIcon::Search).size(px(13.0)).text_color(TEXT_FAINT))
                .child(div().font_family(FONT_MONO).text_size(px(11.0)).text_color(TEXT_FAINT).child("Search"))
                .child(div().font_family(FONT_MONO).text_size(px(10.0)).text_color(TEXT_MUTED).bg(BG_KEY).border_1().border_color(BORDER_KEY).px(px(5.0)).py(px(1.0)).child("⌘K")),
        )
        // 6. Actions that do something: a provider snapshot, a fresh SSH
        //    connection. Left out where they don't apply.
        .children(snapshots.then(|| {
            let app = app.clone();
            header_button("btn-identity-snapshot", "SNAPSHOT").on_click(move |_ev, window, cx| {
                app.update(cx, |this, cx| {
                    this.set_view("danger", cx);
                    this.arm_danger_zone_action("snapshot", window, cx);
                })
            })
        }))
        .children(server.filter(|s| transport_kind(s) == TransportKind::Ssh).map(|_| {
            let app = app.clone();
            header_button("btn-identity-reconnect", "RECONNECT").on_click(move |_ev, _window, cx| app.update(cx, |this, cx| this.reconnect_active_server(cx)))
        }))
        .child(div().w(px(4.0)))
}

/// "kernel 6.8.0", label dimmer than the value.
fn fact(label: &'static str, value: String) -> impl IntoElement {
    div().flex().flex_none().items_center().gap(px(5.0)).child(div().text_color(TEXT_FAINT).child(label)).child(div().text_color(TEXT_SECONDARY).child(value))
}

fn header_button(id: &'static str, label: &'static str) -> Stateful<Div> {
    div()
        .id(id)
        .mr(px(8.0))
        .px(px(9.0))
        .py(px(4.0))
        .border_1()
        .border_color(BORDER_DEFAULT)
        .cursor_pointer()
        .hover(|s| s.bg(BG_ROW_HOVER).text_color(TEXT_PRIMARY))
        .font_family(FONT_MONO)
        .text_size(px(10.5))
        .text_color(TEXT_SECONDARY)
        .child(label)
}

/// A strip under the identity bar while an SSH server can't be reached, so
/// empty pages read as "Crow couldn't read this", not "the server has nothing".
pub fn connection_banner(server: Option<&ServerRecord>, jump: JumpInfo, app: Entity<CrowApp>) -> Option<Div> {
    let s = server?;
    if transport_kind(s) != TransportKind::Ssh {
        return None;
    }
    let state = connection_state(&s.id)?;
    if state == ConnectionState::Connected {
        return None;
    }
    let detail = state.detail().unwrap_or_default().to_string();
    // A warning strip, not an alarm: the Danger Zone bar's palette (background
    // and hairlines) with its buttons' muted red text, like the surge strip.
    Some(
        div()
            .w_full()
            .h(px(crate::components::stat_strip::STATUS_STRIP_HEIGHT))
            .flex_none()
            .px(px(16.0))
            .bg(CRIT_STRIP_BG)
            .border_t_1()
            .border_b_1()
            .border_color(BORDER_DANGER)
            .flex()
            .items_center()
            .gap(px(8.0))
            .font_family(FONT_MONO)
            .text_size(px(10.5))
            .text_color(CRIT_INK_DIM)
            .child(div().flex_none().font_weight(FontWeight::BOLD).child(format!("✕ {}", state.label())))
            .child(div().flex_1().min_w(px(0.0)).overflow_hidden().whitespace_nowrap().text_ellipsis().child(match &jump {
                // The likeliest cause, said first.
                Some((name, true)) => format!("Crow reaches {} through {name}, which is unreachable itself: fix {name} first. {}", s.name, detail),
                _ => format!("Crow can't read {} right now — pages below stay empty until the connection works. {}", s.name, detail),
            }))
            // The diagnosis and the fix (ERR-89).
            .child({
                let (app, id) = (app.clone(), s.id.clone());
                div()
                    .id("link-recovery")
                    .flex_none()
                    .text_color(hex_rgb(0x8ab4ff))
                    .cursor_pointer()
                    .hover(|h| h.text_color(TEXT_PRIMARY).underline())
                    .on_click(move |_ev, window, cx| {
                        let id = id.clone();
                        app.update(cx, |this, cx| this.open_recovery(&id, window, cx));
                    })
                    .child("what happened →")
            })
            // Password servers: an inline link to log in once with the password
            // and install Crow's key.
            .children((s.auth_method == "password").then(|| {
                let srv = s.clone();
                div()
                    .id("link-setup-key-login")
                    .flex_none()
                    .text_color(hex_rgb(0x8ab4ff))
                    .cursor_pointer()
                    .hover(|h| h.text_color(TEXT_PRIMARY).underline())
                    .on_click(move |_ev, _window, cx| {
                        let srv = srv.clone();
                        app.update(cx, |this, cx| this.start_onboarding_for(&srv, cx));
                    })
                    .child("set up key login →")
            })),
    )
}

/// "Frankfurt, Germany · Linode de-fra-2", "Germany (set by hand)",
/// "Germany (by IP)", or an
/// invitation to set it.
fn region_label(s: &ServerRecord) -> String {
    let place = match (crate::region::country_name(&s.region_country), s.region_city.as_str()) {
        (Some(c), "") => c.to_string(),
        (Some(c), city) => format!("{city}, {c}"),
        (None, _) if !s.region_country.is_empty() => s.region_country.clone(),
        (None, _) => return if s.region_provider.is_empty() { "region: unknown ▾".into() } else { format!("{} · region unknown ▾", s.region_provider) },
    };
    match s.region_source.as_str() {
        "manual" => format!("{place} (set by hand) ▾"),
        "geoip" => format!("{place} (by IP) ▾"),
        _ if !s.region_provider.is_empty() => format!("{place} · {} {} ▾", s.region_provider, s.region_code).replace("  ", " "),
        _ => format!("{place} ▾"),
    }
}

/// Pick the server's country by hand, detect it again, or clear it.
fn region_picker(s: &ServerRecord, app: Entity<CrowApp>) -> impl IntoElement {
    let id = s.id.clone();
    let (app_detect, app_clear, id_detect, id_clear) = (app.clone(), app.clone(), id.clone(), id.clone());
    let action = |el_id: &'static str, label: &'static str, color: Rgba| {
        div().id(el_id).px(px(8.0)).py(px(3.0)).border_1().border_color(color).text_color(color).font_weight(FontWeight::BOLD).cursor_pointer().hover(|h| h.bg(BG_ROW_HOVER)).child(label)
    };
    div()
        .absolute()
        .top(px(46.0))
        .left(px(16.0))
        .w(px(300.0))
        .bg(BG_PANEL)
        .border_1()
        .border_color(BORDER_DEFAULT)
        .rounded_md()
        .shadow_lg()
        .occlude()
        .flex()
        .flex_col()
        .font_family(FONT_MONO)
        .text_size(px(10.5))
        .child(
            div()
                .flex()
                .gap(px(6.0))
                .p(px(8.0))
                .border_b_1()
                .border_color(BORDER_PANEL)
                .child(action("region-detect", "DETECT FROM CLOUD", OK).on_click(move |_ev, _window, cx| {
                    let id = id_detect.clone();
                    app_detect.update(cx, |this, cx| {
                        this.region_picker_open = false;
                        this.detect_server_regions(Some(id), cx);
                    })
                }))
                .child(action("region-clear", "CLEAR", TEXT_SECONDARY).on_click(move |_ev, _window, cx| {
                    let id = id_clear.clone();
                    app_clear.update(cx, |this, cx| this.set_server_region(&id, None, cx))
                })),
        )
        .child(
            div().id("region-country-list").h(px(260.0)).overflow_y_scrollbar().py(px(4.0)).children(crate::region::COUNTRIES.iter().map(|(cc, name)| {
                let (app, id) = (app.clone(), id.clone());
                let selected = s.region_country == *cc;
                div()
                    .id(SharedString::from(format!("region-pick-{cc}")))
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .px(px(10.0))
                    .py(px(4.0))
                    .bg(if selected { BG_ROW_SELECTED } else { hex_rgba(0, 0.0) })
                    .text_color(if selected { TEXT_MAX } else { TEXT_SECONDARY })
                    .cursor_pointer()
                    .hover(|h| h.bg(BG_ROW_HOVER))
                    .on_click(move |_ev, _window, cx| {
                        let id = id.clone();
                        app.update(cx, |this, cx| this.set_server_region(&id, Some(cc), cx))
                    })
                    .child(crate::components::flag::flag(cc, 11.0))
                    .child(*name)
            })),
        )
}
