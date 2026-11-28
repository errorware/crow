use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::*;
use crate::theme::*;
use crate::app::CrowApp;
use crate::components::icons::{TablerIcon, inherited_icon};
use super::models::SystemUserRecord;

pub fn passwd_inspector(
    user: &SystemUserRecord,
    all_users: &[SystemUserRecord],
    app: Entity<CrowApp>,
) -> impl IntoElement {
    let app_back = app.clone();
    let app_edit_cfg = app.clone();
    let uname = user.username.clone();

    div()
        .id("passwd-inspector-view")
        .size_full()
        .flex()
        .flex_col()
        .bg(bg_app())
        // 1. Top Control Bar
        .child(
            div()
                .h(px(42.0))
                .flex_none()
                .flex()
                .items_center()
                .justify_between()
                .px(px(16.0))
                .bg(bg_panel())
                .border_b_1()
                .border_color(border_panel())
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(10.0))
                        .child(
                            div()
                                .id("btn-back-to-users")
                                .flex()
                                .items_center()
                                .gap(px(6.0))
                                .px(px(10.0))
                                .py(px(4.0))
                                .bg(bg_control())
                                .border_1()
                                .border_color(border_default())
                                .cursor_pointer()
                                .hover(|s| s.bg(bg_row_hover()).text_color(text_primary()))
                                .font_family(FONT_MONO)
                                .text_size(font_px(10.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(text_secondary())
                                .on_click(move |_ev, _window, cx| {
                                    app_back.update(cx, |this, cx| {
                                        this.users.selected_for_passwd = None; cx.notify();
                                    });
                                })
                                .child(t!("users-passwd-inspector-return-to-users")),
                        )
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(font_px(13.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(text_max())
                                .child("/etc/passwd"),
                        )
                        .child(
                            crate::components::button::badge(accent())
                                .text_size(font_px(9.5))
                                .font_weight(FontWeight::BOLD)
                                .child(t!("users-passwd-inspector-account", uname = uname)),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .child(
                            div()
                                .id("btn-open-in-config")
                                .flex()
                                .items_center()
                                .gap(px(6.0))
                                .px(px(10.0))
                                .py(px(4.0))
                                .bg(theme().ok_bg)
                                .border_1()
                                .border_color(hex_rgba(0x10b981, 0.4))
                                .cursor_pointer()
                                .hover(|s| s.bg(hex_rgba(0x10b981, 0.25)))
                                .font_family(FONT_MONO)
                                .text_size(font_px(10.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme().ok)
                                .on_click(move |_ev, _window, cx| {
                                    app_edit_cfg.update(cx, |this, cx| {
                                        this.select_managed_file("passwd", cx);
                                        this.set_view("config", cx);
                                    });
                                })
                                .child(inherited_icon(TablerIcon::FileText, font_px(12.0)))
                                .child(t!("users-passwd-inspector-open-in-config-editor")),
                        ),
                ),
        )
        // 2. Main Scrollable Content
        .child(
            div()
                .id("passwd-inspector-scroll")
                .flex_1()
                .overflow_y_scrollbar()
                .p(px(20.0))
                .flex()
                .flex_col()
                .gap(px(18.0))
                // Hero Raw String Presentation
                .child(
                    div()
                        .p(px(16.0))
                        .bg(bg_panel())
                        .border_1()
                        .border_color(border_panel())
                        .rounded_md()
                        .flex()
                        .flex_col()
                        .gap(px(10.0))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(font_px(10.5))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(text_muted())
                                        .child(t!("users-passwd-inspector-raw-7-field-posix-passwd")),
                                )
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(font_px(9.0))
                                        .text_color(text_faint())
                                        .child(t!("users-passwd-inspector-delimiter-colon")),
                                ),
                        )
                        .child(
                            div()
                                .p(px(12.0))
                                .bg(hex_rgba(0x000000, 0.5))
                                .border_1()
                                .border_color(border_default())
                                .rounded_sm()
                                .font_family(FONT_MONO)
                                .text_size(font_px(14.0))
                                .child(render_raw_passwd_line(user)),
                        ),
                )
                // Exploded 7-Field Anatomy Cards
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(10.0))
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(font_px(11.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(text_primary())
                                .child(t!("users-passwd-inspector-exploded-field-anatomy-system-semantics")),
                        )
                        .child(render_exploded_fields(user)),
                )
                // Quick Switch User Record selector
                .child(
                    div()
                        .mt(px(10.0))
                        .flex()
                        .flex_col()
                        .gap(px(8.0))
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(font_px(10.5))
                                .font_weight(FontWeight::BOLD)
                                .text_color(text_muted())
                                .child(t!("users-passwd-inspector-switch-passwd-record-to-inspect")),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_wrap()
                                .gap(px(6.0))
                                .children(all_users.iter().map(|u| {
                                    let app_sw = app.clone();
                                    let un = u.username.clone();
                                    let is_curr = u.username == user.username;
                                    div()
                                        .id(ElementId::NamedInteger(format!("sw-user-{}", un).into(), 0))
                                        .px(px(8.0))
                                        .py(px(4.0))
                                        .border_1()
                                        .border_color(if is_curr { accent_border() } else { border_default() })
                                        .bg(if is_curr { bg_nav_active() } else { bg_control() })
                                        .cursor_pointer()
                                        .hover(|s| s.bg(bg_row_hover()))
                                        .font_family(FONT_MONO)
                                        .text_size(font_px(10.0))
                                        .font_weight(if is_curr { FontWeight::BOLD } else { FontWeight::NORMAL })
                                        .text_color(if is_curr { accent() } else { text_secondary() })
                                        .on_click(move |_ev, _window, cx| {
                                            let target = un.clone();
                                            app_sw.update(cx, |this, cx| {
                                                this.users.selected_for_passwd = Some(target.to_string()); cx.notify();
                                            });
                                        })
                                        .child(format!("{}:{}", u.username, u.uid))
                                })),
                        ),
                ),
        )
}

fn render_raw_passwd_line(user: &SystemUserRecord) -> impl IntoElement {
    let pwd_flag = if user.is_locked { "!" } else { "x" };

    div()
        .flex()
        .flex_wrap()
        .items_center()
        .child(span_token(&user.username, accent())) // username
        .child(span_delim())
        .child(span_token(pwd_flag, hex_rgb(0xf43f5e))) // password (rose)
        .child(span_delim())
        .child(span_token(&user.uid.to_string(), hex_rgb(0x10b981))) // uid (green)
        .child(span_delim())
        .child(span_token(&user.gid.to_string(), hex_rgb(0x60a5fa))) // gid (blue)
        .child(span_delim())
        .child(span_token(&user.gecos, hex_rgb(0xc084fc))) // gecos (purple)
        .child(span_delim())
        .child(span_token(&user.home_dir, hex_rgb(0xfbbf24))) // home (amber)
        .child(span_delim())
        .child(span_token(&user.shell, hex_rgb(0x34d399))) // shell (mint)
}

fn span_token(text: &str, color: Rgba) -> Div {
    div()
        .font_weight(FontWeight::BOLD)
        .text_color(color)
        .child(text.to_string())
}

fn span_delim() -> Div {
    div()
        .px(px(2.0))
        .font_weight(FontWeight::BOLD)
        .text_color(hex_rgb(0xec4899))
        .child(":")
}

fn render_exploded_fields(user: &SystemUserRecord) -> impl IntoElement {
    let pwd_flag = if user.is_locked { "!" } else { "x" };

    let fields = [
        (
            1,
            t!("users-passwd-inspector-login-username"),
            user.username.clone(),
            accent(),
            "Identifier used to authenticate during SSH, login, PAM, or sudo sessions. Restricted to alphanumeric characters.",
        ),
        (
            2,
            t!("users-passwd-inspector-password-placeholder"),
            pwd_flag.to_string(),
            hex_rgb(0xf43f5e),
            if user.is_locked {
                t!("users-passwd-inspector-indicates-account-password-is-locked")
            } else {
                t!("users-passwd-inspector-x-indicates-hashed-passwords-and")
            },
        ),
        (
            3,
            t!("users-passwd-inspector-user-identifier-uid"),
            user.uid.to_string(),
            hex_rgb(0x10b981),
            if user.uid == 0 {
                t!("users-passwd-inspector-uid-0-superuser-root-with")
            } else if user.uid < 1000 {
                t!("users-passwd-inspector-system-service-uid-reserved-for")
            } else {
                t!("users-passwd-inspector-interactive-login-uid-standard-regular")
            },
        ),
        (
            4,
            t!("users-passwd-inspector-primary-group-gid"),
            user.gid.to_string(),
            hex_rgb(0x60a5fa),
            t!("users-passwd-inspector-primary-numeric-group-id-applied"),
        ),
        (
            5,
            t!("users-passwd-inspector-gecos-metadata"),
            if user.gecos.is_empty() { t!("users-passwd-inspector-empty").to_string() } else { user.gecos.clone() },
            hex_rgb(0xc084fc),
            t!("users-passwd-inspector-general-electric-comprehensive-operating-supervisor"),
        ),
        (
            6,
            t!("users-passwd-inspector-home-directory"),
            user.home_dir.clone(),
            hex_rgb(0xfbbf24),
            t!("users-passwd-inspector-initial-working-directory-set-upon"),
        ),
        (
            7,
            t!("users-passwd-inspector-login-shell"),
            user.shell.clone(),
            hex_rgb(0x34d399),
            if user.shell.contains("nologin") || user.shell.contains("false") {
                t!("users-passwd-inspector-prohibits-interactive-terminal-access-daemons")
            } else {
                t!("users-passwd-inspector-command-interpreter-spawned-upon-login")
            },
        ),
    ];

    div()
        .flex()
        .flex_col()
        .gap(px(8.0))
        .children(fields.iter().map(|(num, label, val, color, desc)| {
            div()
                .p(px(12.0))
                .bg(bg_panel())
                .border_1()
                .border_color(border_default())
                .rounded_sm()
                .flex()
                .flex_col()
                .gap(px(4.0))
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
                                    crate::components::button::badge(*color)
                                        .text_size(font_px(9.0))
                                        .font_weight(FontWeight::BOLD)
                                        .child(t!("users-passwd-inspector-portion", num = num)),
                                )
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(font_px(11.0))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(text_primary())
                                        .child(*label),
                                ),
                        )
                        .child(
                            div()
                                .px(px(8.0))
                                .py(px(2.0))
                                .bg(hex_rgba(0x000000, 0.4))
                                .border_1()
                                .border_color(border_default())
                                .rounded_sm()
                                .font_family(FONT_MONO)
                                .text_size(font_px(12.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(*color)
                                .child(val.clone()),
                        ),
                )
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(font_px(10.0))
                        .text_color(text_muted())
                        .child(*desc),
                )
        }))
}
