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
        .bg(BG_APP)
        // 1. Top Control Bar
        .child(
            div()
                .h(px(42.0))
                .flex_none()
                .flex()
                .items_center()
                .justify_between()
                .px(px(16.0))
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
                                .id("btn-back-to-users")
                                .flex()
                                .items_center()
                                .gap(px(6.0))
                                .px(px(10.0))
                                .py(px(4.0))
                                .bg(BG_CONTROL)
                                .border_1()
                                .border_color(BORDER_DEFAULT)
                                .rounded_sm()
                                .cursor_pointer()
                                .hover(|s| s.bg(BG_ROW_HOVER).text_color(TEXT_PRIMARY))
                                .font_family(FONT_MONO)
                                .text_size(px(10.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(TEXT_SECONDARY)
                                .on_click(move |_ev, _window, cx| {
                                    app_back.update(cx, |this, cx| {
                                        this.users.selected_for_passwd = None; cx.notify();
                                    });
                                })
                                .child("← RETURN TO USERS"),
                        )
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(13.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(TEXT_MAX)
                                .child("/etc/passwd"),
                        )
                        .child(
                            div()
                                .bg(hex_rgba(0x8ab4ff, 0.15))
                                .border_1()
                                .border_color(hex_rgba(0x8ab4ff, 0.4))
                                .px(px(6.0))
                                .py(px(1.5))
                                .rounded_sm()
                                .font_family(FONT_MONO)
                                .text_size(px(9.5))
                                .font_weight(FontWeight::BOLD)
                                .text_color(hex_rgb(0x8ab4ff))
                                .child(format!("ACCOUNT: {}", uname)),
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
                                .bg(OK_BG)
                                .border_1()
                                .border_color(hex_rgba(0x10b981, 0.4))
                                .rounded_sm()
                                .cursor_pointer()
                                .hover(|s| s.bg(hex_rgba(0x10b981, 0.25)))
                                .font_family(FONT_MONO)
                                .text_size(px(10.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(OK)
                                .on_click(move |_ev, _window, cx| {
                                    app_edit_cfg.update(cx, |this, cx| {
                                        this.select_managed_file("passwd", cx);
                                        this.set_view("config", cx);
                                    });
                                })
                                .child(inherited_icon(TablerIcon::FileText, px(12.0)))
                                .child("OPEN IN CONFIG EDITOR"),
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
                        .bg(BG_PANEL)
                        .border_1()
                        .border_color(BORDER_PANEL)
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
                                        .text_size(px(10.5))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(TEXT_MUTED)
                                        .child("RAW 7-FIELD POSIX PASSWD RECORD"),
                                )
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(9.0))
                                        .text_color(TEXT_FAINT)
                                        .child("DELIMITER: ':' (COLON)"),
                                ),
                        )
                        .child(
                            div()
                                .p(px(12.0))
                                .bg(hex_rgba(0x000000, 0.5))
                                .border_1()
                                .border_color(BORDER_DEFAULT)
                                .rounded_sm()
                                .font_family(FONT_MONO)
                                .text_size(px(14.0))
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
                                .text_size(px(11.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(TEXT_PRIMARY)
                                .child("EXPLODED FIELD ANATOMY & SYSTEM SEMANTICS"),
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
                                .text_size(px(10.5))
                                .font_weight(FontWeight::BOLD)
                                .text_color(TEXT_MUTED)
                                .child("SWITCH PASSWD RECORD TO INSPECT"),
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
                                        .rounded_sm()
                                        .border_1()
                                        .border_color(if is_curr { hex_rgba(0x8ab4ff, 0.5) } else { BORDER_DEFAULT })
                                        .bg(if is_curr { BG_NAV_ACTIVE } else { BG_CONTROL })
                                        .cursor_pointer()
                                        .hover(|s| s.bg(BG_ROW_HOVER))
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.0))
                                        .font_weight(if is_curr { FontWeight::BOLD } else { FontWeight::NORMAL })
                                        .text_color(if is_curr { hex_rgb(0x8ab4ff) } else { TEXT_SECONDARY })
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
        .child(span_token(&user.username, hex_rgb(0x38bdf8))) // username (cyan)
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
            "LOGIN USERNAME",
            user.username.clone(),
            hex_rgb(0x38bdf8),
            "Identifier used to authenticate during SSH, login, PAM, or sudo sessions. Restricted to alphanumeric characters.",
        ),
        (
            2,
            "PASSWORD PLACEHOLDER",
            pwd_flag.to_string(),
            hex_rgb(0xf43f5e),
            if user.is_locked {
                "'!' indicates account password is locked. Login through PAM/password is denied."
            } else {
                "'x' indicates hashed passwords and salt are shadowed in /etc/shadow, restricted to root (0640)."
            },
        ),
        (
            3,
            "USER IDENTIFIER (UID)",
            user.uid.to_string(),
            hex_rgb(0x10b981),
            if user.uid == 0 {
                "UID 0: Superuser (root) with unrestricted Linux kernel privileges."
            } else if user.uid < 1000 {
                "System Service UID: Reserved for isolated background daemons and system services."
            } else {
                "Interactive Login UID: Standard regular human user accounts start at UID 1000+."
            },
        ),
        (
            4,
            "PRIMARY GROUP (GID)",
            user.gid.to_string(),
            hex_rgb(0x60a5fa),
            "Primary numeric group ID applied to new files and processes created by this user. Defined in /etc/group.",
        ),
        (
            5,
            "GECOS METADATA",
            if user.gecos.is_empty() { "(empty)".to_string() } else { user.gecos.clone() },
            hex_rgb(0xc084fc),
            "General Electric Comprehensive Operating Supervisor legacy field. Holds Full Name, Office, Phone numbers.",
        ),
        (
            6,
            "HOME DIRECTORY",
            user.home_dir.clone(),
            hex_rgb(0xfbbf24),
            "Initial working directory set upon login session initialization. Houses ~/.ssh, ~/.config, and dotfiles.",
        ),
        (
            7,
            "LOGIN SHELL",
            user.shell.clone(),
            hex_rgb(0x34d399),
            if user.shell.contains("nologin") || user.shell.contains("false") {
                "Prohibits interactive terminal access. Daemons and service accounts use nologin for defense in depth."
            } else {
                "Command interpreter spawned upon login session (e.g. /bin/bash, /bin/zsh, /usr/bin/fish)."
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
                .bg(BG_PANEL)
                .border_1()
                .border_color(BORDER_DEFAULT)
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
                                    div()
                                        .px(px(6.0))
                                        .py(px(1.5))
                                        .bg(Rgba { a: 0.15, ..*color })
                                        .border_1()
                                        .border_color(Rgba { a: 0.4, ..*color })
                                        .rounded_sm()
                                        .font_family(FONT_MONO)
                                        .text_size(px(9.0))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(*color)
                                        .child(format!("PORTION #{}", num)),
                                )
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(11.0))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(TEXT_PRIMARY)
                                        .child(*label),
                                ),
                        )
                        .child(
                            div()
                                .px(px(8.0))
                                .py(px(2.0))
                                .bg(hex_rgba(0x000000, 0.4))
                                .border_1()
                                .border_color(BORDER_DEFAULT)
                                .rounded_sm()
                                .font_family(FONT_MONO)
                                .text_size(px(12.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(*color)
                                .child(val.clone()),
                        ),
                )
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(10.0))
                        .text_color(TEXT_MUTED)
                        .child(*desc),
                )
        }))
}
