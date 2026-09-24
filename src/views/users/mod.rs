pub mod models;
pub mod passwd_inspector;
pub mod ssh_attach_modal;
pub mod new_user_modal;
pub mod state;
pub mod host_data;

#[allow(unused_imports)]
pub use models::{SystemUserRecord, UserAccountStatus, UserFilterTab, UserSshKeySummary};
pub use passwd_inspector::passwd_inspector;
pub use ssh_attach_modal::ssh_attach_modal;
pub use new_user_modal::{new_user_modal, NewUserInputs, NewUserState};
pub use state::UsersState;

use gpui_kit::component::input::InputState;
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use crate::theme::*;
use crate::app::CrowApp;
use crate::vault::SshKeyRecord;
use crate::components::icons::{TablerIcon, tabler_icon};

use crate::components::table_controls::{render_table_controls, Chip};

const INSPECTOR_WIDTH: f32 = 400.0;

/// The inspector's SET PASSWORD inputs (masked).
pub struct PasswordInputs {
    pub password: Entity<InputState>,
    pub confirm: Entity<InputState>,
    pub _events: Vec<Subscription>,
}

pub fn user_management_view(
    app: Entity<CrowApp>,
    users: &UsersState,
    enrolled_keys: &[SshKeyRecord],
    search: Option<&Entity<InputState>>,
    new_user_inputs: Option<&NewUserInputs>,
    password_inputs: Option<&PasswordInputs>,
) -> AnyElement {
    // Raw /etc/passwd view for one account.
    if let Some(target_uname) = &users.selected_for_passwd {
        if let Some(target_user) = users.users.iter().find(|u| u.username == *target_uname) {
            return passwd_inspector(target_user, &users.users, app.clone()).into_any_element();
        }
    }

    let visible = users.visible();
    let inspected = users.inspected();
    let chips: Vec<Chip> = UserFilterTab::ALL
        .into_iter()
        .map(|tab| {
            let app = app.clone();
            Chip {
                id: format!("user-filter-{}", tab.label()),
                label: tab.label(),
                count: users.users.iter().filter(|u| tab.matches(u)).count(),
                is_on: users.filter_tab == tab,
                alarming: false,
                on_click: Box::new(move |cx| app.update(cx, |this, cx| this.set_users_filter(tab, cx))),
            }
        })
        .collect();
    let app_new = app.clone();

    let header = div()
        .h(px(40.0))
        .flex_none()
        .flex()
        .items_center()
        .bg(BG_PANEL)
        .border_b_1()
        .border_color(BORDER_PANEL)
        .child(div().px(px(14.0)).h_full().flex().items_center().border_r_1().border_color(BORDER_PANEL).font_family(FONT_MONO).text_size(px(11.0)).font_weight(FontWeight::SEMIBOLD).text_color(TEXT_PRIMARY).child("USERS"))
        .child(render_table_controls(search, chips))
        .child(div().flex_1())
        .child(
            div()
                .id("btn-new-user")
                .mx(px(12.0))
                .px(px(10.0))
                .py(px(4.0))
                .border_1()
                .border_color(OK)
                .text_color(OK)
                .font_family(FONT_MONO)
                .text_size(px(10.5))
                .font_weight(FontWeight::BOLD)
                .cursor_pointer()
                .hover(|s| s.bg(OK_BG))
                .on_click(move |_ev, _window, cx| app_new.update(cx, |this, cx| this.open_new_user_modal(cx)))
                .child("+ NEW USER"),
        );

    let column_header = div()
        .h(px(28.0))
        .flex_none()
        .flex()
        .items_center()
        .gap(px(12.0))
        .px(px(14.0))
        .bg(BG_SUBHEAD)
        .border_b_1()
        .border_color(BORDER_PANEL)
        .font_family(FONT_MONO)
        .text_size(px(9.5))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(TEXT_DIMMER)
        .child(div().w(px(10.0)).flex_none())
        .child(div().flex_grow(2.0).flex_basis(px(0.0)).min_w(px(160.0)).child("USER"))
        .child(div().w(px(60.0)).flex_none().text_align(TextAlign::Right).child("UID"))
        .child(div().flex_grow(1.5).flex_basis(px(0.0)).min_w(px(120.0)).child("GROUPS"))
        .child(div().flex_grow(1.0).flex_basis(px(0.0)).min_w(px(110.0)).child("SHELL"))
        .child(div().w(px(50.0)).flex_none().text_align(TextAlign::Right).child("KEYS"))
        .child(div().w(px(76.0)).flex_none().child("STATUS"));

    let rows: Vec<AnyElement> = if let Some(e) = users.load_error.as_ref().filter(|_| users.users.is_empty()) {
        vec![message_row(format!("Couldn't read accounts: {e}"), CRIT)]
    } else if users.users.is_empty() {
        vec![message_row("Reading accounts…".into(), TEXT_FAINT)]
    } else if visible.is_empty() {
        vec![message_row("No accounts match — try ALL, or clear the search".into(), TEXT_FAINT)]
    } else {
        visible
            .iter()
            .enumerate()
            .map(|(i, u)| user_row(u, i, inspected.is_some_and(|s| s.username == u.username), users.sudo_group(), app.clone()))
            .collect()
    };

    let ssh_modal_element = users.selected_for_ssh.as_ref().and_then(|name| users.users.iter().find(|u| &u.username == name)).map(|u| ssh_attach_modal(u, enrolled_keys, app.clone()).into_any_element());
    let new_user_modal_element = users
        .show_new_user_modal
        .then(|| new_user_modal(&users.new_user, new_user_inputs, users.shell_choices(), users.group_choices(), users.sudo_group(), app.clone()).into_any_element());

    div()
        .id("user-management-view")
        .size_full()
        .relative()
        .flex()
        .flex_col()
        .bg(BG_APP)
        .child(header)
        .child(
            div()
                .flex_1()
                .min_h(px(0.0))
                .flex()
                // Table
                .child(
                    div()
                        .flex_1()
                        .min_w(px(0.0))
                        .flex()
                        .flex_col()
                        .child(column_header)
                        .child(div().id("users-table").flex_1().min_h(px(0.0)).overflow_y_scrollbar().flex().flex_col().children(rows)),
                )
                // Inspector
                .child(
                    div()
                        .id("user-inspector")
                        .w(px(INSPECTOR_WIDTH))
                        .flex_none()
                        .h_full()
                        .bg(BG_RAIL)
                        .border_l_1()
                        .border_color(BORDER_PANEL)
                        .overflow_y_scrollbar()
                        .child(match inspected {
                            Some(u) => inspector(u, users, password_inputs, app.clone()).into_any_element(),
                            None => message_row("Select an account to see its details.".into(), TEXT_FAINT),
                        }),
                ),
        )
        // Status of the command running on the server, or the last result.
        .children(users.pending.as_ref().map(|p| format!("Running: {p} …")).or_else(|| users.toast.clone()).map(|msg| {
            let app_dismiss = app.clone();
            div()
                .absolute()
                .bottom(px(16.0))
                .left(px(16.0))
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
                .child(div().size(px(7.0)).rounded_full().bg(if users.pending.is_some() { WARN } else { OK }))
                .child(div().font_family(FONT_MONO).text_size(px(10.5)).text_color(TEXT_PRIMARY).child(msg))
                .child(
                    div()
                        .id("btn-dismiss-user-toast")
                        .p(px(2.0))
                        .cursor_pointer()
                        .hover(|s| s.bg(BG_ROW_HOVER))
                        .on_click(move |_ev, _window, cx| app_dismiss.update(cx, |this, cx| {
                            this.users.toast = None;
                            cx.notify();
                        }))
                        .child(tabler_icon(TablerIcon::X).size(px(11.0)).text_color(TEXT_MUTED)),
                )
        }))
        .children(ssh_modal_element)
        .children(new_user_modal_element)
        .into_any_element()
}

fn message_row(text: String, color: Rgba) -> AnyElement {
    div().p(px(20.0)).font_family(FONT_MONO).text_size(px(11.0)).text_color(color).child(text).into_any_element()
}

fn status_badge(u: &SystemUserRecord) -> Div {
    let (bg, fg) = match u.status() {
        UserAccountStatus::Active => (OK_BG, OK),
        UserAccountStatus::Locked => (CRIT_BG, CRIT),
        UserAccountStatus::SystemDaemon => (BG_CHIP, TEXT_DIM),
    };
    div().flex_none().rounded_sm().px(px(5.0)).py(px(1.5)).bg(bg).text_color(fg).text_size(px(8.5)).font_weight(FontWeight::BOLD).child(u.status().label())
}

fn user_row(u: &SystemUserRecord, idx: usize, selected: bool, sudo_group: Option<&str>, app: Entity<CrowApp>) -> AnyElement {
    let name = u.username.clone();
    let full_name = u.gecos.split(',').next().unwrap_or("").trim().to_string();
    let other_groups: Vec<&String> = u.groups.iter().filter(|g| **g != u.primary_group).collect();
    let groups = match other_groups.len() {
        0 => u.primary_group.clone(),
        n => format!("{} +{}", u.primary_group, n),
    };
    let is_admin = u.username == "root" || sudo_group.is_some_and(|g| u.groups.iter().any(|x| x == g)) || u.is_sudoer();
    div()
        .id(ElementId::NamedInteger("user-row".into(), idx as u64))
        .relative()
        .flex()
        .items_center()
        .gap(px(12.0))
        .px(px(14.0))
        .py(px(7.0))
        .border_b_1()
        .border_color(BORDER_ROW)
        .bg(if selected { BG_NAV_ACTIVE } else { hex_rgba(0, 0.0) })
        .cursor_pointer()
        .hover(|s| s.bg(BG_ROW_HOVER))
        .on_click(move |_ev, _window, cx| {
            let name = name.clone();
            app.update(cx, |this, cx| this.select_user(&name, cx));
        })
        .children(selected.then(|| div().absolute().left_0().top_0().bottom_0().w(px(2.0)).bg(OK)))
        .font_family(FONT_MONO)
        .child(div().w(px(10.0)).flex_none().child(div().size(px(6.0)).rounded_full().bg(match u.status() {
            UserAccountStatus::Active => OK,
            UserAccountStatus::Locked => CRIT,
            UserAccountStatus::SystemDaemon => TEXT_FAINTER,
        })))
        .child(
            div()
                .flex_grow(2.0)
                .flex_basis(px(0.0))
                .min_w(px(160.0))
                .flex()
                .items_center()
                .gap(px(8.0))
                .overflow_hidden()
                .child(div().flex_none().text_size(px(11.5)).font_weight(if u.is_human() { FontWeight::SEMIBOLD } else { FontWeight::NORMAL }).text_color(if u.is_human() { TEXT_PRIMARY } else { TEXT_DIM }).child(u.username.clone()))
                .children(is_admin.then(|| div().flex_none().rounded_sm().px(px(4.0)).py(px(1.0)).bg(WARN_BG).text_color(WARN).text_size(px(8.0)).font_weight(FontWeight::BOLD).child("SUDO")))
                .children((!full_name.is_empty() && full_name != u.username).then(|| div().min_w(px(0.0)).text_size(px(10.0)).text_color(TEXT_FAINT).child(full_name))),
        )
        .child(div().w(px(60.0)).flex_none().text_align(TextAlign::Right).text_size(px(10.5)).text_color(TEXT_DIM).child(u.uid.to_string()))
        .child(div().flex_grow(1.5).flex_basis(px(0.0)).min_w(px(120.0)).overflow_hidden().text_size(px(10.5)).text_color(TEXT_SECONDARY).child(groups))
        .child(div().flex_grow(1.0).flex_basis(px(0.0)).min_w(px(110.0)).overflow_hidden().text_size(px(10.5)).text_color(TEXT_DIM).child(u.shell.clone()))
        .child(div().w(px(50.0)).flex_none().text_align(TextAlign::Right).text_size(px(10.5)).text_color(if u.authorized_keys.is_empty() { TEXT_FAINTER } else { TEXT_PRIMARY }).child(u.authorized_keys.len().to_string()))
        .child(div().w(px(76.0)).flex_none().flex().child(status_badge(u)))
        .into_any_element()
}

fn inspector_section(title: &'static str) -> Div {
    div().px(px(14.0)).pt(px(14.0)).pb(px(6.0)).font_family(FONT_MONO).text_size(px(9.0)).font_weight(FontWeight::BOLD).text_color(TEXT_FAINT).child(title)
}

fn detail(label: &'static str, value: String) -> Div {
    div()
        .flex()
        .gap(px(10.0))
        .px(px(14.0))
        .py(px(3.0))
        .font_family(FONT_MONO)
        .text_size(px(10.5))
        .child(div().w(px(90.0)).flex_none().text_color(TEXT_FAINT).child(label))
        .child(div().flex_1().min_w(px(0.0)).overflow_hidden().text_color(TEXT_PRIMARY).child(value))
}

fn action_button(id: String, label: String, color: Rgba, enabled: bool, on_click: impl Fn(&mut App) + 'static) -> Stateful<Div> {
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

/// Everything about one account, with its actions.
fn inspector(u: &SystemUserRecord, users: &UsersState, password_inputs: Option<&PasswordInputs>, app: Entity<CrowApp>) -> impl IntoElement {
    let busy = users.pending.is_some();
    let is_root = u.username == "root";
    let sudo_group = users.sudo_group();
    let in_sudo = sudo_group.is_some_and(|g| u.groups.iter().any(|x| x == g));
    let name = u.username.clone();
    let full_name = u.gecos.split(',').next().unwrap_or("").trim().to_string();

    // Header
    let mut panel = div().flex().flex_col().pb(px(16.0)).child(
        div()
            .px(px(14.0))
            .py(px(14.0))
            .border_b_1()
            .border_color(BORDER_PANEL)
            .flex()
            .flex_col()
            .gap(px(4.0))
            .font_family(FONT_MONO)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .child(div().text_size(px(16.0)).font_weight(FontWeight::BOLD).text_color(TEXT_MAX).child(u.username.clone()))
                    .child(status_badge(u))
                    .children(u.is_sudoer().then(|| div().rounded_sm().px(px(5.0)).py(px(1.5)).bg(WARN_BG).text_color(WARN).text_size(px(8.5)).font_weight(FontWeight::BOLD).child(if is_root { "ROOT" } else { "SUDO" }))),
            )
            .children((!full_name.is_empty() && full_name != u.username).then(|| div().text_size(px(10.5)).text_color(TEXT_DIM).child(full_name))),
    );

    // Account details
    panel = panel
        .child(inspector_section("ACCOUNT"))
        .child(detail("uid / gid", format!("{} / {}", u.uid, u.gid)))
        .child(detail("home", u.home_dir.clone()))
        .child(detail("shell", u.shell.clone()))
        .child(detail("can log in", if u.is_locked { "no — password locked".into() } else if u.shell.ends_with("nologin") || u.shell.ends_with("/false") { "no — no login shell".into() } else { "yes".into() }));

    // Groups, with admin rights as a toggle
    panel = panel.child(inspector_section("GROUPS")).child(
        div().flex().flex_wrap().gap(px(5.0)).px(px(14.0)).children(u.groups.iter().map(|g| {
            let is_primary = *g == u.primary_group;
            div()
                .rounded_sm()
                .px(px(6.0))
                .py(px(2.0))
                .bg(if is_primary { BG_CHIP } else { BG_APP })
                .border_1()
                .border_color(BORDER_DEFAULT)
                .font_family(FONT_MONO)
                .text_size(px(10.0))
                .text_color(if Some(g.as_str()) == sudo_group { WARN } else { TEXT_SECONDARY })
                .child(if is_primary { format!("{g} (primary)") } else { g.clone() })
        })),
    );
    if let (Some(group), false) = (sudo_group, is_root) {
        let (app, n) = (app.clone(), name.clone());
        panel = panel.child(div().px(px(14.0)).pt(px(8.0)).child(action_button(
            format!("user-sudo-{name}"),
            if in_sudo { format!("REMOVE FROM {group} (NO SUDO)") } else { format!("MAKE ADMIN: ADD TO {group} (SUDO)") },
            WARN,
            !busy,
            move |cx| {
                let n = n.clone();
                app.update(cx, |this, cx| this.user_toggle_group(&n, group, cx));
            },
        )));
    }

    // SSH keys
    panel = panel.child(inspector_section("AUTHORIZED SSH KEYS"));
    if u.authorized_keys.is_empty() {
        panel = panel.child(div().px(px(14.0)).py(px(2.0)).font_family(FONT_MONO).text_size(px(10.5)).text_color(TEXT_FAINT).child("No keys in ~/.ssh/authorized_keys."));
    }
    for (i, k) in u.authorized_keys.iter().enumerate() {
        let (app, n, id) = (app.clone(), name.clone(), k.id.clone());
        panel = panel.child(
            div()
                .mx(px(14.0))
                .mb(px(5.0))
                .p(px(8.0))
                .bg(BG_APP)
                .border_1()
                .border_color(BORDER_PANEL)
                .flex()
                .items_start()
                .gap(px(8.0))
                .font_family(FONT_MONO)
                .child(
                    div()
                        .flex_1()
                        .min_w(px(0.0))
                        .flex()
                        .flex_col()
                        .gap(px(2.0))
                        .child(div().text_size(px(10.5)).text_color(TEXT_PRIMARY).child(k.comment.clone().filter(|c| !c.is_empty()).unwrap_or_else(|| k.name.clone())))
                        .child(div().overflow_hidden().text_size(px(9.5)).text_color(TEXT_FAINT).child(format!("{} · {}", k.algorithm, k.fingerprint))),
                )
                .child(action_button(format!("user-revoke-{name}-{i}"), "REVOKE".into(), CRIT, !busy, move |cx| {
                    let (n, id) = (n.clone(), id.clone());
                    app.update(cx, |this, cx| this.user_revoke_key(&n, &id, cx));
                })),
        );
    }
    {
        let (app, n) = (app.clone(), name.clone());
        panel = panel.child(div().px(px(14.0)).pt(px(2.0)).child(action_button(format!("user-attach-{name}"), "+ ADD KEY FROM VAULT".into(), OK, !busy, move |cx| {
            let n = n.clone();
            app.update(cx, |this, cx| {
                this.users.selected_for_ssh = Some(n);
                cx.notify();
            });
        })));
    }

    // Login shell
    panel = panel.child(inspector_section("LOGIN SHELL")).child(
        div().flex().flex_wrap().gap(px(5.0)).px(px(14.0)).children(users.shell_choices().into_iter().enumerate().map(|(i, shell)| {
            let current = shell == u.shell;
            let (app, n, sh) = (app.clone(), name.clone(), shell.clone());
            div()
                .id(SharedString::from(format!("user-shell-{i}")))
                .rounded_sm()
                .px(px(7.0))
                .py(px(2.0))
                .border_1()
                .border_color(if current { OK } else { BORDER_DEFAULT })
                .bg(if current { OK_BG } else { hex_rgba(0, 0.0) })
                .text_color(if current { OK } else { TEXT_DIMMER })
                .font_family(FONT_MONO)
                .text_size(px(10.0))
                .when(!current && !busy, |d| d.cursor_pointer().hover(|s| s.bg(BG_ROW_HOVER)).on_click(move |_ev, _window, cx| {
                    let (n, sh) = (n.clone(), sh.clone());
                    app.update(cx, |this, cx| this.user_set_shell(&n, &sh, cx));
                }))
                .child(shell)
        })),
    );

    // Password
    let form_open = users.password_for.as_deref() == Some(u.username.as_str());
    panel = panel.child(inspector_section("PASSWORD")).child(
        div().px(px(14.0)).py(px(2.0)).font_family(FONT_MONO).text_size(px(10.5)).text_color(TEXT_SECONDARY).child(if u.is_locked {
            "locked — no password login (setting one unlocks it)"
        } else {
            "set (can log in with a password)"
        }),
    );
    if form_open {
        let (app_save, app_cancel, n_cancel) = (app.clone(), app.clone(), name.clone());
        panel = panel.child(
            div()
                .mx(px(14.0))
                .mt(px(6.0))
                .p(px(10.0))
                .bg(BG_APP)
                .border_1()
                .border_color(BORDER_PANEL)
                .flex()
                .flex_col()
                .gap(px(6.0))
                .children(password_inputs.map(|p| {
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(6.0))
                        .child(gpui_kit::component::input::Input::new(&p.password).font_family(FONT_MONO).text_size(px(11.0)).rounded(px(2.0)))
                        .child(gpui_kit::component::input::Input::new(&p.confirm).font_family(FONT_MONO).text_size(px(11.0)).rounded(px(2.0)))
                }))
                .child(div().font_family(FONT_MONO).text_size(px(9.5)).text_color(TEXT_FAINT).child("Set with chpasswd over stdin; Crow doesn't keep it."))
                .child(
                    div()
                        .flex()
                        .gap(px(6.0))
                        .child(action_button(format!("user-pw-save-{name}"), "SET PASSWORD".into(), OK, !busy, move |cx| app_save.update(cx, |this, cx| this.user_set_password(cx))))
                        .child(action_button(format!("user-pw-cancel-{name}"), "CANCEL".into(), TEXT_SECONDARY, true, move |cx| {
                            let n = n_cancel.clone();
                            app_cancel.update(cx, |this, cx| this.toggle_password_form(&n, cx));
                        })),
                ),
        );
    } else {
        let (app_pw, n_pw) = (app.clone(), name.clone());
        panel = panel.child(div().px(px(14.0)).pt(px(6.0)).child(action_button(format!("user-pw-{name}"), "SET PASSWORD…".into(), OK, !busy, move |cx| {
            let n = n_pw.clone();
            app_pw.update(cx, |this, cx| this.toggle_password_form(&n, cx));
        })));
    }

    // Account actions
    let confirming = users.confirm_delete.as_deref() == Some(u.username.as_str());
    let (app_lock, app_passwd, app_delete) = (app.clone(), app.clone(), app);
    let (n_lock, n_passwd, n_delete) = (name.clone(), name.clone(), name.clone());
    panel.child(inspector_section("ACTIONS")).child(
        div()
            .flex()
            .flex_wrap()
            .gap(px(6.0))
            .px(px(14.0))
            .child(action_button(format!("user-lock-{name}"), if u.is_locked { "UNLOCK PASSWORD".into() } else { "LOCK PASSWORD".into() }, WARN, !busy && !is_root, move |cx| {
                let n = n_lock.clone();
                app_lock.update(cx, |this, cx| this.user_toggle_lock(&n, cx));
            }))
            .child(action_button(format!("user-passwd-{name}"), "VIEW PASSWD ENTRY".into(), TEXT_SECONDARY, true, move |cx| {
                let n = n_passwd.clone();
                app_passwd.update(cx, |this, cx| {
                    this.users.selected_for_passwd = Some(n);
                    cx.notify();
                });
            }))
            .child(action_button(format!("user-delete-{name}"), if confirming { format!("CLICK AGAIN TO DELETE {}", u.username) } else { "DELETE USER".into() }, CRIT, !busy && !is_root && u.uid != 0, move |cx| {
                let n = n_delete.clone();
                app_delete.update(cx, |this, cx| this.user_delete_clicked(&n, cx));
            })),
    )
}
