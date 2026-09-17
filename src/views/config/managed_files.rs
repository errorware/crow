use gpui_kit::*;
use crate::theme::*;

pub struct ManagedFileDef {
    pub name: &'static str,
    pub path: &'static str,
    pub pill: &'static str,
    pub color: Rgba,
    pub is_selected: bool,
}

pub fn default_managed_files() -> &'static [ManagedFileDef] {
    &[
        ManagedFileDef { name: "pg_hba.conf", path: "/etc/postgresql/16/main", pill: "EDITED", color: WARN, is_selected: true },
        ManagedFileDef { name: "sshd_config", path: "/etc/ssh", pill: "OK", color: OK, is_selected: false },
        ManagedFileDef { name: "postgresql.conf", path: "/etc/postgresql/16/main", pill: "OK", color: OK, is_selected: false },
        ManagedFileDef { name: "ufw/user.rules", path: "/etc/ufw", pill: "EDITED", color: WARN, is_selected: false },
        ManagedFileDef { name: "nginx.conf", path: "/etc/nginx", pill: "OK", color: OK, is_selected: false },
        ManagedFileDef { name: "sites-enabled/api", path: "/etc/nginx", pill: "OK", color: OK, is_selected: false },
        ManagedFileDef { name: "authorized_keys", path: "/root/.ssh", pill: "DRIFT", color: CRIT, is_selected: false },
        ManagedFileDef { name: "fail2ban/jail.local", path: "/etc/fail2ban", pill: "OK", color: OK, is_selected: false },
        ManagedFileDef { name: "sysctl.d/99-tuning", path: "/etc", pill: "OK", color: OK, is_selected: false },
        ManagedFileDef { name: "crontab", path: "/etc", pill: "OK", color: OK, is_selected: false },
        ManagedFileDef { name: "resolv.conf", path: "/etc", pill: "LOCKED", color: TEXT_DIMMER, is_selected: false },
        ManagedFileDef { name: "docker/daemon.json", path: "/etc", pill: "OK", color: OK, is_selected: false },
    ]
}

pub fn managed_files_rail() -> impl IntoElement {
    div()
        .w(px(216.0))
        .flex_none()
        .h_full()
        .flex()
        .flex_col()
        .bg(BG_RAIL)
        .border_r_1()
        .border_color(BORDER_PANEL)
        // Header
        .child(
            div()
                .h(px(34.0))
                .flex_none()
                .flex()
                .items_center()
                .justify_between()
                .px(px(12.0))
                .bg(BG_PANEL)
                .border_b_1()
                .border_color(BORDER_PANEL)
                .child(
                    div()
                        .font_family("Inter")
                        .text_size(px(11.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .child("MANAGED FILES"),
                )
                .child(
                    div()
                        .font_family("JetBrains Mono")
                        .text_size(px(10.0))
                        .text_color(TEXT_FAINT)
                        .child("12"),
                ),
        )
        // File items
        .child(
            div()
                .id("managed-files-scroll")
                .flex_1()
                .min_h(px(0.0))
                .overflow_y_scroll()
                .flex()
                .flex_col()
                .children(default_managed_files().iter().map(|f| {
                    let is_sel = f.is_selected;
                    let pill_bg = match f.pill {
                        "OK" => OK_BG,
                        "EDITED" => WARN_BG,
                        "DRIFT" => CRIT_BG,
                        _ => BG_CHIP,
                    };

                    div()
                        .flex()
                        .flex_col()
                        .gap(px(2.0))
                        .px(px(12.0))
                        .py(px(6.0))
                        .border_b_1()
                        .border_color(BORDER_ROW)
                        .border_l_2()
                        .border_color(if is_sel { TEXT_PRIMARY } else { rgb(0x00000000) })
                        .bg(if is_sel { BG_NAV_ACTIVE } else { rgb(0x00000000) })
                        .cursor_pointer()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(6.0))
                                .child(
                                    div()
                                        .flex_1()
                                        .min_w(px(0.0))
                                        .font_family("JetBrains Mono")
                                        .text_size(px(11.0))
                                        .font_weight(if is_sel { FontWeight::SEMIBOLD } else { FontWeight::NORMAL })
                                        .text_color(if is_sel { TEXT_MAX } else { TEXT_SECONDARY })
                                        .child(f.name),
                                )
                                .child(
                                    div()
                                        .bg(pill_bg)
                                        .text_color(f.color)
                                        .font_family("JetBrains Mono")
                                        .text_size(px(8.5))
                                        .font_weight(FontWeight::BOLD)
                                        .px(px(4.0))
                                        .py(px(1.5))
                                        .flex_none()
                                        .child(f.pill),
                                ),
                        )
                        .child(
                            div()
                                .font_family("JetBrains Mono")
                                .text_size(px(9.5))
                                .text_color(TEXT_FAINTER)
                                .child(f.path),
                        )
                })),
        )
        // Schema packs footer
        .child(
            div()
                .border_t_1()
                .border_color(BORDER_PANEL)
                .p(px(9.0))
                .px(px(12.0))
                .flex()
                .flex_col()
                .gap(px(5.0))
                .child(
                    div()
                        .font_family("Inter")
                        .text_size(px(10.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(TEXT_DIMMER)
                        .child("SCHEMA PACKS"),
                )
                .child(
                    div()
                        .font_family("JetBrains Mono")
                        .text_size(px(10.0))
                        .text_color(TEXT_DIM)
                        .child("openssh 9.6 · postgres 16"),
                )
                .child(
                    div()
                        .font_family("JetBrains Mono")
                        .text_size(px(10.0))
                        .text_color(TEXT_DIM)
                        .child("ufw 0.36 · nginx 1.24"),
                ),
        )
}
