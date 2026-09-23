pub mod models;
pub mod passwd_inspector;
pub mod ssh_attach_modal;
pub mod new_user_modal;
pub mod state;

#[allow(unused_imports)]
pub use models::{default_system_users, SystemUserRecord, UserAccountStatus, UserFilterTab, UserSshKeySummary};
pub use passwd_inspector::passwd_inspector;
pub use ssh_attach_modal::ssh_attach_modal;
pub use new_user_modal::{new_user_modal, NewUserState};
pub use state::UsersState;

use gpui_kit::*;
use crate::theme::*;
use crate::app::CrowApp;
use crate::vault::SshKeyRecord;
use crate::components::icons::{TablerIcon, tabler_icon};

pub fn user_management_view(
    app: Entity<CrowApp>,
    users: &UsersState,
    enrolled_keys: &[SshKeyRecord],
) -> AnyElement {
    // If user clicked to inspect /etc/passwd for a specific user
    if let Some(target_uname) = &users.selected_for_passwd {
        if let Some(target_user) = users.users.iter().find(|u| u.username == *target_uname) {
            return passwd_inspector(target_user, &users.users, app.clone()).into_any_element();
        }
    }

    let all_users = &users.users;
    let query = users.search_query.to_lowercase();
    let filter_tab = users.filter_tab;

    let filtered_users: Vec<&SystemUserRecord> = all_users
        .iter()
        .filter(|u| {
            // Tab filter
            match filter_tab {
                UserFilterTab::All => true,
                UserFilterTab::Human => u.is_human(),
                UserFilterTab::Sudoers => u.is_sudoer(),
                UserFilterTab::System => u.is_system_user,
            }
        })
        .filter(|u| {
            // Search filter
            if query.is_empty() {
                return true;
            }
            u.username.to_lowercase().contains(&query)
                || u.gecos.to_lowercase().contains(&query)
                || u.shell.to_lowercase().contains(&query)
                || u.groups.iter().any(|g| g.to_lowercase().contains(&query))
        })
        .collect();

    let total_count = all_users.len();
    let human_count = all_users.iter().filter(|u| u.is_human()).count();
    let sudoers_count = all_users.iter().filter(|u| u.is_sudoer()).count();
    let system_count = all_users.iter().filter(|u| u.is_system_user).count();

    let app_new = app.clone();
    let app_passwd_all = app.clone();

    let ssh_modal_element = if let Some(target_uname) = &users.selected_for_ssh {
        users.users.iter().find(|u| u.username == *target_uname).map(|u| {
            ssh_attach_modal(u, enrolled_keys, app.clone()).into_any_element()
        })
    } else {
        None
    };

    let new_user_modal_element = if users.show_new_user_modal {
        Some(new_user_modal(&users.new_user, app.clone()).into_any_element())
    } else {
        None
    };

    div()
        .id("user-management-view")
        .size_full()
        .relative()
        .flex()
        .flex_col()
        .bg(BG_APP)
        // 1. Header Toolbar
        .child(
            div()
                .h(px(44.0))
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
                        .child(tabler_icon(TablerIcon::Users).size(px(16.0)).text_color(hex_rgb(0x8ab4ff)))
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(12.5))
                                .font_weight(FontWeight::BOLD)
                                .text_color(TEXT_MAX)
                                .child("USER ACCOUNTS & AUTHENTICATION"),
                        )
                        .child(
                            div()
                                .bg(hex_rgba(0x8ab4ff, 0.12))
                                .text_color(hex_rgb(0x8ab4ff))
                                .font_family(FONT_MONO)
                                .text_size(px(9.0))
                                .font_weight(FontWeight::BOLD)
                                .px(px(6.0))
                                .py(px(2.0))
                                .rounded_sm()
                                .child(format!("{} ACCOUNTS", total_count)),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        // Inspect /etc/passwd button
                        .child(
                            div()
                                .id("btn-open-passwd-overview")
                                .px(px(10.0))
                                .py(px(4.5))
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
                                    app_passwd_all.update(cx, |this, cx| {
                                        this.users.selected_for_passwd = Some("nelson".to_string()); cx.notify();
                                    });
                                })
                                .child("INSPECT /etc/passwd"),
                        )
                        // + NEW USER button
                        .child(
                            div()
                                .id("btn-open-new-user-modal")
                                .px(px(12.0))
                                .py(px(4.5))
                                .bg(OK)
                                .rounded_sm()
                                .cursor_pointer()
                                .hover(|s| s.bg(hex_rgb(0x34d399)))
                                .font_family(FONT_MONO)
                                .text_size(px(10.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(0x0a0a0c))
                                .on_click(move |_ev, _window, cx| {
                                    app_new.update(cx, |this, cx| {
                                        this.users.open_new_user_modal(); cx.notify();
                                    });
                                })
                                .child("+ NEW USER"),
                        ),
                ),
        )
        // 2. Filter & Stat Sub-Bar
        .child(
            div()
                .h(px(38.0))
                .flex_none()
                .flex()
                .items_center()
                .justify_between()
                .px(px(16.0))
                .bg(hex_rgba(0x000000, 0.25))
                .border_b_1()
                .border_color(BORDER_ROW)
                .child(
                    // Filter Tab Pills
                    div()
                        .flex()
                        .items_center()
                        .gap(px(4.0))
                        .children([
                            (UserFilterTab::All, "All Accounts", total_count),
                            (UserFilterTab::Human, "Login Users", human_count),
                            (UserFilterTab::Sudoers, "Sudoers", sudoers_count),
                            (UserFilterTab::System, "System Daemons", system_count),
                        ].iter().map(|(tab, label, count)| {
                            let is_sel = filter_tab == *tab;
                            let app_tab = app.clone();
                            let t = *tab;
                            div()
                                .id(ElementId::NamedInteger(format!("user-tab-{:?}", t).into(), 0))
                                .px(px(8.0))
                                .py(px(3.0))
                                .rounded_sm()
                                .border_1()
                                .border_color(if is_sel { hex_rgba(0x8ab4ff, 0.4) } else { hex_rgba(0, 0.0) })
                                .bg(if is_sel { BG_NAV_ACTIVE } else { hex_rgba(0, 0.0) })
                                .cursor_pointer()
                                .hover(|s| s.bg(BG_ROW_HOVER))
                                .font_family(FONT_MONO)
                                .text_size(px(10.0))
                                .font_weight(if is_sel { FontWeight::BOLD } else { FontWeight::NORMAL })
                                .text_color(if is_sel { TEXT_MAX } else { TEXT_MUTED })
                                .on_click(move |_ev, _window, cx| {
                                    app_tab.update(cx, |this, cx| {
                                        this.users.filter_tab = t; cx.notify();
                                    });
                                })
                                .child(format!("{} ({})", label, count))
                        })),
                )
                .child(
                    // Search Filter
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        .px(px(8.0))
                        .py(px(3.0))
                        .bg(BG_APP)
                        .border_1()
                        .border_color(BORDER_DEFAULT)
                        .rounded_sm()
                        .child(tabler_icon(TablerIcon::Search).size(px(11.0)).text_color(TEXT_MUTED))
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(9.5))
                                .text_color(if users.search_query.is_empty() { TEXT_FAINTER } else { TEXT_PRIMARY })
                                .child(if users.search_query.is_empty() {
                                    "Filter users by name, shell, group...".to_string()
                                } else {
                                    users.search_query.clone()
                                }),
                        )
                        .children(if !users.search_query.is_empty() {
                            let app_clear = app.clone();
                            Some(
                                div()
                                    .id("btn-clear-user-search")
                                    .p(px(2.0))
                                    .cursor_pointer()
                                    .on_click(move |_ev, _window, cx| {
                                        app_clear.update(cx, |this, cx| {
                                            this.users.search_query = "".to_string(); cx.notify();
                                        });
                                    })
                                    .child(tabler_icon(TablerIcon::X).size(px(10.0)).text_color(TEXT_MUTED))
                            )
                        } else {
                            None
                        }),
                ),
        )
        // 3. User Cards Grid / List
        .child(
            div()
                .id("user-cards-scroll")
                .flex_1()
                .overflow_y_scroll()
                .p(px(16.0))
                .flex()
                .flex_col()
                .gap(px(12.0))
                .children(if filtered_users.is_empty() {
                    vec![
                        div()
                            .p(px(32.0))
                            .flex()
                            .flex_col()
                            .items_center()
                            .justify_center()
                            .gap(px(8.0))
                            .child(tabler_icon(TablerIcon::Users).size(px(28.0)).text_color(TEXT_FAINTER))
                            .child(
                                div()
                                    .font_family(FONT_MONO)
                                    .text_size(px(12.0))
                                    .text_color(TEXT_MUTED)
                                    .child("No system user accounts match the active filter criteria."),
                            )
                            .into_any_element()
                    ]
                } else {
                    filtered_users
                        .iter()
                        .enumerate()
                        .map(|(idx, user)| render_user_card(user, idx, app.clone()))
                        .collect()
                }),
        )
        // 4. Action Toast / Feedback Strip (if any)
        .children(if let Some(msg) = &users.toast {
            let app_dismiss = app.clone();
            Some(
                div()
                    .absolute()
                    .bottom(px(16.0))
                    .right(px(16.0))
                    .px(px(14.0))
                    .py(px(8.0))
                    .bg(BG_PANEL)
                    .border_1()
                    .border_color(BORDER_DEFAULT)
                    .rounded_md()
                    .shadow_lg()
                    .flex()
                    .items_center()
                    .gap(px(10.0))
                    .child(div().size(px(7.0)).rounded_full().bg(OK))
                    .child(
                        div()
                            .font_family(FONT_MONO)
                            .text_size(px(10.5))
                            .text_color(TEXT_PRIMARY)
                            .child(msg.clone()),
                    )
                    .child(
                        div()
                            .id("btn-dismiss-user-toast")
                            .p(px(2.0))
                            .cursor_pointer()
                            .hover(|s| s.bg(BG_ROW_HOVER))
                            .on_click(move |_ev, _window, cx| {
                                app_dismiss.update(cx, |this, cx| {
                                    this.users.toast = None;
                                    cx.notify();
                                });
                            })
                            .child(tabler_icon(TablerIcon::X).size(px(11.0)).text_color(TEXT_MUTED)),
                    ),
            )
        } else {
            None
        })
        .children(ssh_modal_element)
        .children(new_user_modal_element)
        .into_any_element()
}

fn render_user_card(
    user: &SystemUserRecord,
    idx: usize,
    app: Entity<CrowApp>,
) -> AnyElement {
    let uname = user.username.clone();
    let app_lock = app.clone();
    let app_del = app.clone();
    let app_ssh = app.clone();
    let app_inspect = app.clone();
    let app_shell = app.clone();

    let u_lock_target = uname.clone();
    let u_del_target = uname.clone();
    let u_ssh_target = uname.clone();
    let u_insp_target = uname.clone();
    let u_shell_target = uname.clone();

    let is_sudoer = user.is_sudoer();
    let is_locked = user.is_locked;
    let initial_char = user.username.chars().next().unwrap_or('?').to_ascii_uppercase();

    div()
        .id(ElementId::NamedInteger("user-card".into(), idx as u64))
        .p(px(14.0))
        .bg(BG_PANEL)
        .border_1()
        .border_color(if is_sudoer { hex_rgba(0x8ab4ff, 0.25) } else { BORDER_DEFAULT })
        .rounded_md()
        .flex()
        .flex_col()
        .gap(px(10.0))
        // Top Section: Avatar, Identity, Status, and Controls
        .child(
            div()
                .flex()
                .items_start()
                .justify_between()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(12.0))
                        // Avatar Badge
                        .child(
                            div()
                                .size(px(38.0))
                                .rounded_full()
                                .flex()
                                .items_center()
                                .justify_center()
                                .bg(if is_sudoer {
                                    hex_rgba(0x8ab4ff, 0.15)
                                } else if is_locked {
                                    hex_rgba(0xef4444, 0.15)
                                } else {
                                    hex_rgba(0xffffff, 0.05)
                                })
                                .border_1()
                                .border_color(if is_sudoer {
                                    hex_rgba(0x8ab4ff, 0.4)
                                } else if is_locked {
                                    hex_rgba(0xef4444, 0.4)
                                } else {
                                    BORDER_DEFAULT
                                })
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(15.0))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(if is_sudoer {
                                            hex_rgb(0x8ab4ff)
                                        } else if is_locked {
                                            CRIT
                                        } else {
                                            TEXT_PRIMARY
                                        })
                                        .child(initial_char.to_string()),
                                ),
                        )
                        // Name & IDs
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(2.0))
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap(px(8.0))
                                        .child(
                                            div()
                                                .font_family(FONT_MONO)
                                                .text_size(px(13.5))
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(TEXT_MAX)
                                                .child(user.username.clone()),
                                        )
                                        .children(if is_sudoer {
                                            Some(
                                                div()
                                                    .px(px(6.0))
                                                    .py(px(1.5))
                                                    .bg(hex_rgba(0x8ab4ff, 0.15))
                                                    .border_1()
                                                    .border_color(hex_rgba(0x8ab4ff, 0.4))
                                                    .rounded_sm()
                                                    .font_family(FONT_MONO)
                                                    .text_size(px(8.5))
                                                    .font_weight(FontWeight::BOLD)
                                                    .text_color(hex_rgb(0x8ab4ff))
                                                    .child("⚡ SUDOER"),
                                            )
                                        } else {
                                            None
                                        })
                                        .child(
                                            div()
                                                .px(px(5.0))
                                                .py(px(1.5))
                                                .bg(if is_locked {
                                                    hex_rgba(0xef4444, 0.15)
                                                } else {
                                                    OK_BG
                                                })
                                                .rounded_sm()
                                                .font_family(FONT_MONO)
                                                .text_size(px(8.5))
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(if is_locked { CRIT } else { OK })
                                                .child(user.status().label()),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap(px(8.0))
                                        .child(
                                            div()
                                                .font_family(FONT_MONO)
                                                .text_size(px(10.0))
                                                .text_color(TEXT_TERTIARY)
                                                .child(user.gecos.clone()),
                                        )
                                        .child(
                                            div()
                                                .font_family(FONT_MONO)
                                                .text_size(px(9.0))
                                                .text_color(TEXT_FAINT)
                                                .child(format!("UID {} · GID {}", user.uid, user.gid)),
                                        )
                                        .child(
                                            div()
                                                .font_family(FONT_MONO)
                                                .text_size(px(9.0))
                                                .text_color(TEXT_FAINTER)
                                                .child(user.home_dir.clone()),
                                        ),
                                ),
                        ),
                )
                // Top Action Buttons: Passwd Inspect, Lock/Unlock, Delete
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        // Passwd Inspector Button
                        .child(
                            div()
                                .id(ElementId::NamedInteger(format!("btn-insp-{}", uname).into(), 0))
                                .px(px(8.0))
                                .py(px(3.5))
                                .bg(BG_CONTROL)
                                .border_1()
                                .border_color(BORDER_DEFAULT)
                                .rounded_sm()
                                .cursor_pointer()
                                .hover(|s| s.bg(BG_ROW_HOVER).text_color(TEXT_PRIMARY))
                                .font_family(FONT_MONO)
                                .text_size(px(9.5))
                                .font_weight(FontWeight::BOLD)
                                .text_color(TEXT_SECONDARY)
                                .on_click(move |_ev, _window, cx| {
                                    let u = u_insp_target.clone();
                                    app_inspect.update(cx, |this, cx| {
                                        this.users.selected_for_passwd = Some(u.to_string()); cx.notify();
                                    });
                                })
                                .child("INSPECT PASSWD"),
                        )
                        // Lock / Unlock toggle
                        .child(
                            div()
                                .id(ElementId::NamedInteger(format!("btn-lock-{}", uname).into(), 0))
                                .px(px(8.0))
                                .py(px(3.5))
                                .bg(if is_locked { hex_rgba(0x10b981, 0.12) } else { hex_rgba(0xef4444, 0.12) })
                                .border_1()
                                .border_color(if is_locked { hex_rgba(0x10b981, 0.35) } else { hex_rgba(0xef4444, 0.35) })
                                .rounded_sm()
                                .cursor_pointer()
                                .hover(|s| s.bg(BG_ROW_HOVER))
                                .font_family(FONT_MONO)
                                .text_size(px(9.5))
                                .font_weight(FontWeight::BOLD)
                                .text_color(if is_locked { OK } else { CRIT })
                                .on_click(move |_ev, _window, cx| {
                                    let u = u_lock_target.clone();
                                    app_lock.update(cx, |this, cx| {
                                        this.users.toggle_lock(&u); cx.notify();
                                    });
                                })
                                .child(if is_locked { "UNLOCK" } else { "LOCK" }),
                        )
                        // Delete user button (disabled for root)
                        .children(if user.username != "root" {
                            Some(
                                div()
                                    .id(ElementId::NamedInteger(format!("btn-del-{}", uname).into(), 0))
                                    .p(px(4.0))
                                    .rounded_sm()
                                    .cursor_pointer()
                                    .hover(|s| s.bg(BG_ROW_HOVER))
                                    .on_click(move |_ev, _window, cx| {
                                        let u = u_del_target.clone();
                                        app_del.update(cx, |this, cx| {
                                            this.users.delete_user(&u); cx.notify();
                                        });
                                    })
                                    .child(tabler_icon(TablerIcon::Trash).size(px(13.0)).text_color(TEXT_MUTED))
                            )
                        } else {
                            None
                        }),
                ),
        )
        // Mid Section: Visual Group Membership (usermod -aG)
        .child(
            div()
                .p(px(10.0))
                .bg(hex_rgba(0x000000, 0.3))
                .border_1()
                .border_color(BORDER_ROW)
                .rounded_sm()
                .flex()
                .flex_col()
                .gap(px(6.0))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(9.5))
                                .font_weight(FontWeight::BOLD)
                                .text_color(TEXT_MUTED)
                                .child("GROUP MEMBERSHIP (CLICK TO TOGGLE usermod -aG):"),
                        )
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(8.5))
                                .text_color(TEXT_FAINT)
                                .child(format!("PRIMARY: {}", user.primary_group)),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .gap(px(5.0))
                        .children(["sudo", "docker", "adm", "systemd-journal", "www-data", "wheel"].iter().map(|grp| {
                            let is_in = user.groups.iter().any(|g| g == *grp);
                            let g_name = grp.to_string();
                            let u_name = user.username.clone();
                            let app_grp = app.clone();

                            div()
                                .id(ElementId::NamedInteger(format!("user-{}-grp-{}", user.username, grp).into(), 0))
                                .px(px(7.0))
                                .py(px(2.5))
                                .rounded_sm()
                                .border_1()
                                .border_color(if is_in {
                                    if *grp == "sudo" || *grp == "wheel" {
                                        hex_rgba(0x8ab4ff, 0.5)
                                    } else {
                                        hex_rgba(0x10b981, 0.4)
                                    }
                                } else {
                                    BORDER_DEFAULT
                                })
                                .bg(if is_in {
                                    if *grp == "sudo" || *grp == "wheel" {
                                        hex_rgba(0x8ab4ff, 0.15)
                                    } else {
                                        hex_rgba(0x10b981, 0.12)
                                    }
                                } else {
                                    BG_CONTROL
                                })
                                .cursor_pointer()
                                .hover(|s| s.bg(BG_ROW_HOVER))
                                .font_family(FONT_MONO)
                                .text_size(px(9.0))
                                .font_weight(if is_in { FontWeight::BOLD } else { FontWeight::NORMAL })
                                .text_color(if is_in {
                                    if *grp == "sudo" || *grp == "wheel" {
                                        hex_rgb(0x8ab4ff)
                                    } else {
                                        OK
                                    }
                                } else {
                                    TEXT_FAINT
                                })
                                .on_click(move |_ev, _window, cx| {
                                    let u = u_name.clone();
                                    let g = g_name.clone();
                                    app_grp.update(cx, |this, cx| {
                                        this.users.toggle_group(&u, &g); cx.notify();
                                    });
                                })
                                .child(if is_in { format!("✓ {}", grp) } else { format!("+ {}", grp) })
                        })),
                ),
        )
        // Bottom Section: Shell & Attached SSH Keys Strip
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .gap(px(12.0))
                // Shell Selector
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(9.5))
                                .text_color(TEXT_MUTED)
                                .child("SHELL:"),
                        )
                        .child(
                            div()
                                .id(ElementId::NamedInteger(format!("btn-shell-cycle-{}", uname).into(), 0))
                                .px(px(6.0))
                                .py(px(2.0))
                                .bg(BG_APP)
                                .border_1()
                                .border_color(BORDER_DEFAULT)
                                .rounded_sm()
                                .cursor_pointer()
                                .hover(|s| s.bg(BG_ROW_HOVER))
                                .font_family(FONT_MONO)
                                .text_size(px(9.5))
                                .font_weight(FontWeight::BOLD)
                                .text_color(hex_rgb(0x38bdf8))
                                .on_click(move |_ev, _window, cx| {
                                    let u = u_shell_target.clone();
                                    app_shell.update(cx, |this, cx| {
                                        this.users.cycle_shell(&u); cx.notify();
                                    });
                                })
                                .child(format!("{} ▾", user.shell)),
                        ),
                )
                // Attached SSH Keys
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(4.0))
                                .children(user.authorized_keys.iter().map(|k| {
                                    div()
                                        .px(px(6.0))
                                        .py(px(2.0))
                                        .bg(hex_rgba(0x8ab4ff, 0.12))
                                        .border_1()
                                        .border_color(hex_rgba(0x8ab4ff, 0.3))
                                        .rounded_sm()
                                        .font_family(FONT_MONO)
                                        .text_size(px(8.5))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(hex_rgb(0x8ab4ff))
                                        .child(format!("🔑 {}", k.name))
                                })),
                        )
                        .child(
                            div()
                                .id(ElementId::NamedInteger(format!("btn-attach-ssh-{}", uname).into(), 0))
                                .flex()
                                .items_center()
                                .gap(px(4.0))
                                .px(px(8.0))
                                .py(px(3.0))
                                .bg(BG_CONTROL)
                                .border_1()
                                .border_color(BORDER_DEFAULT)
                                .rounded_sm()
                                .cursor_pointer()
                                .hover(|s| s.bg(BG_ROW_HOVER).text_color(TEXT_PRIMARY))
                                .font_family(FONT_MONO)
                                .text_size(px(9.5))
                                .font_weight(FontWeight::BOLD)
                                .text_color(TEXT_SECONDARY)
                                .on_click(move |_ev, _window, cx| {
                                    let u = u_ssh_target.clone();
                                    app_ssh.update(cx, |this, cx| {
                                        this.users.selected_for_ssh = Some(u.to_string()); cx.notify();
                                    });
                                })
                                .child(tabler_icon(TablerIcon::Key).size(px(11.0)))
                                .child(if user.authorized_keys.is_empty() {
                                    "+ ATTACH SSH KEY"
                                } else {
                                    "MANAGE KEYS"
                                }),
                        ),
                ),
        )
        .into_any_element()
}
