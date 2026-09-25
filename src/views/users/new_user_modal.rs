use gpui_kit::*;
use gpui_kit::component::input::{Input, InputState};
use gpui_kit::prelude::FluentBuilder as _;
use crate::theme::*;
use crate::app::CrowApp;
use crate::components::icons::{TablerIcon, tabler_icon};

#[derive(Clone, Debug)]
pub struct NewUserState {
    pub username: String,
    pub gecos: String,
    pub shell: String,
    pub grant_sudo: bool,
    pub create_home: bool,
    pub selected_groups: Vec<String>,
    /// Optional; set with chpasswd after useradd, never stored.
    pub password: crate::secret_string::SecretString,
    pub password_confirm: crate::secret_string::SecretString,
}

impl Default for NewUserState {
    fn default() -> Self {
        Self {
            username: String::new(),
            gecos: String::new(),
            shell: "/bin/bash".to_string(),
            grant_sudo: false,
            create_home: true,
            selected_groups: Vec::new(),
            password: Default::default(),
            password_confirm: Default::default(),
        }
    }
}

/// The dialog's text fields (real inputs, created by the app on open).
pub struct NewUserInputs {
    pub username: Entity<InputState>,
    pub gecos: Entity<InputState>,
    pub password: Entity<InputState>,
    pub password_confirm: Entity<InputState>,
    pub _events: Vec<Subscription>,
}

fn field(input: Option<&Entity<InputState>>) -> Div {
    div().w_full().children(input.map(|i| Input::new(i).font_family(FONT_MONO).text_size(px(11.5)).bg(BG_APP).rounded(px(2.0))))
}

/// `shells`: offered login shells; `groups`: supplementary groups that
/// exist on this server; `sudo_group`: its admin group (sudo or wheel).
pub fn new_user_modal(
    state: &NewUserState,
    inputs: Option<&NewUserInputs>,
    shells: Vec<String>,
    groups: Vec<String>,
    sudo_group: Option<&'static str>,
    app: Entity<CrowApp>,
) -> impl IntoElement {
    let app_close = app.clone();
    let app_backdrop = app.clone();
    let app_submit = app.clone();
    let app_sudo = app.clone();
    let app_home = app.clone();
    let current_sudo = state.grant_sudo;
    let current_home = state.create_home;

    div()
        .id("new-user-modal-backdrop")
        .absolute()
        .inset_0()
        .bg(hex_rgba(0x000000, 0.65))
        .flex()
        .items_center()
        .justify_center()
        .on_click(move |_ev, _window, cx| {
            app_backdrop.update(cx, |this, cx| {
                this.users.show_new_user_modal = false; cx.notify();
            });
        })
        .child(
            div()
                .id("new-user-modal-panel")
                .w(px(520.0))
                .bg(BG_PANEL)
                .border_1()
                .border_color(BORDER_PANEL)
                .rounded_md()
                .flex()
                .flex_col()
                .on_mouse_down(MouseButton::Left, |_ev, _window, cx| cx.stop_propagation()) // keep clicks inside from reaching the backdrop (which closes)
                // Header
                .child(
                    div()
                        .h(px(46.0))
                        .flex_none()
                        .flex()
                        .items_center()
                        .justify_between()
                        .px(px(16.0))
                        .border_b_1()
                        .border_color(BORDER_PANEL)
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(8.0))
                                .child(tabler_icon(TablerIcon::Users).size(px(16.0)).text_color(OK))
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(12.5))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(TEXT_MAX)
                                        .child("PROVISION NEW SYSTEM USER"),
                                ),
                        )
                        .child(
                            div()
                                .id("btn-close-new-user")
                                .p(px(4.0))
                                .rounded_sm()
                                .cursor_pointer()
                                .hover(|s| s.bg(BG_ROW_HOVER))
                                .on_click(move |_ev, _window, cx| {
                                    app_close.update(cx, |this, cx| {
                                        this.users.show_new_user_modal = false; cx.notify();
                                    });
                                })
                                .child(tabler_icon(TablerIcon::X).size(px(14.0)).text_color(TEXT_MUTED)),
                        ),
                )
                // Form Body
                .child(
                    div()
                        .p(px(16.0))
                        .flex()
                        .flex_col()
                        .gap(px(12.0))
                        // Username Field
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(4.0))
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.0))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(TEXT_MUTED)
                                        .child("LOGIN USERNAME (POSIX)"),
                                )
                                .child(field(inputs.map(|i| &i.username))),
                        )
                        // Full Name / GECOS
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(4.0))
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.0))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(TEXT_MUTED)
                                        .child("FULL NAME (GECOS METADATA)"),
                                )
                                .child(field(inputs.map(|i| &i.gecos))),
                        )
                        // Password (optional)
                        .child({
                            let mismatch = !state.password_confirm.is_empty() && state.password != state.password_confirm;
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(4.0))
                                .child(div().font_family(FONT_MONO).text_size(px(10.0)).font_weight(FontWeight::BOLD).text_color(TEXT_MUTED).child("PASSWORD (OPTIONAL)"))
                                .child(div().flex().gap(px(8.0)).child(div().flex_1().child(field(inputs.map(|i| &i.password)))).child(div().flex_1().child(field(inputs.map(|i| &i.password_confirm)))))
                                .child(div().font_family(FONT_MONO).text_size(px(9.5)).text_color(if mismatch { CRIT } else { TEXT_FAINT }).child(if mismatch {
                                    "The passwords don't match."
                                } else if state.password.is_empty() {
                                    "Leave empty for key-only login (add a key from the inspector afterwards)."
                                } else {
                                    "Set with chpasswd over stdin; Crow doesn't keep it."
                                }))
                        })
                        // Shell & Privileges Row
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .gap(px(12.0))
                                .child(
                                    div()
                                        .flex_1()
                                        .flex()
                                        .flex_col()
                                        .gap(px(4.0))
                                        .child(
                                            div()
                                                .font_family(FONT_MONO)
                                                .text_size(px(10.0))
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(TEXT_MUTED)
                                                .child("LOGIN SHELL"),
                                        )
                                        .child(div().flex().flex_wrap().gap(px(4.0)).children(shells.into_iter().enumerate().map(|(i, shell)| {
                                            let current = shell == state.shell;
                                            let (app, sh) = (app.clone(), shell.clone());
                                            div()
                                                .id(SharedString::from(format!("new-user-shell-{i}")))
                                                .px(px(7.0))
                                                .py(px(3.0))
                                                .rounded_sm()
                                                .border_1()
                                                .border_color(if current { OK } else { BORDER_DEFAULT })
                                                .bg(if current { OK_BG } else { BG_APP })
                                                .text_color(if current { OK } else { TEXT_DIMMER })
                                                .font_family(FONT_MONO)
                                                .text_size(px(10.0))
                                                .when(!current, |d| d.cursor_pointer().hover(|s| s.bg(BG_ROW_HOVER)).on_click(move |_ev, _window, cx| {
                                                    let sh = sh.clone();
                                                    app.update(cx, |this, cx| {
                                                        this.users.new_user.shell = sh;
                                                        cx.notify();
                                                    });
                                                }))
                                                .child(shell)
                                        }))),
                                )
                                .child(
                                    div()
                                        .flex_1()
                                        .flex()
                                        .flex_col()
                                        .gap(px(4.0))
                                        .child(
                                            div()
                                                .font_family(FONT_MONO)
                                                .text_size(px(10.0))
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(TEXT_MUTED)
                                                .child("INITIAL PRIVILEGES"),
                                        )
                                        .child(
                                            div()
                                                .id("btn-toggle-new-sudo")
                                                .px(px(10.0))
                                                .py(px(6.0))
                                                .bg(if current_sudo { hex_rgba(0x10b981, 0.15) } else { BG_APP })
                                                .border_1()
                                                .border_color(if current_sudo { hex_rgba(0x10b981, 0.4) } else { BORDER_DEFAULT })
                                                .rounded_sm()
                                                .cursor_pointer()
                                                .hover(|s| s.bg(BG_ROW_HOVER))
                                                .when(sudo_group.is_some(), |d| d.on_click(move |_ev, _window, cx| {
                                                    app_sudo.update(cx, |this, cx| {
                                                        this.users.new_user.grant_sudo = !current_sudo;
                                                        cx.notify();
                                                    });
                                                }))
                                                .flex()
                                                .items_center()
                                                .justify_between()
                                                .child(
                                                    div()
                                                        .font_family(FONT_MONO)
                                                        .text_size(px(11.0))
                                                        .font_weight(FontWeight::BOLD)
                                                        .text_color(if current_sudo { OK } else { TEXT_MUTED })
                                                        .child(match (current_sudo, sudo_group) {
                                                            (true, Some(g)) => format!("✓ ADMIN ({g} group)"),
                                                            (false, Some(g)) => format!("STANDARD USER · click to add to {g}"),
                                                            (_, None) => "STANDARD USER (no sudo/wheel group here)".to_string(),
                                                        }),
                                                ),
                                        ),
                                ),
                        )
                        // Group Quick Pills
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(4.0))
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.0))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(TEXT_MUTED)
                                        .child("SUPPLEMENTARY GROUPS (usermod -aG)"),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_wrap()
                                        .gap(px(6.0))
                                        .children(groups.iter().map(|grp| {
                                            let is_sel = state.selected_groups.contains(&grp.to_string());
                                            let g_name = grp.to_string();
                                            let app_grp = app.clone();
                                            div()
                                                .id(ElementId::NamedInteger(format!("new-user-grp-{}", grp).into(), 0))
                                                .px(px(8.0))
                                                .py(px(3.0))
                                                .bg(if is_sel { hex_rgba(0x8ab4ff, 0.15) } else { BG_CONTROL })
                                                .border_1()
                                                .border_color(if is_sel { hex_rgba(0x8ab4ff, 0.4) } else { BORDER_DEFAULT })
                                                .rounded_sm()
                                                .cursor_pointer()
                                                .hover(|s| s.bg(BG_ROW_HOVER))
                                                .font_family(FONT_MONO)
                                                .text_size(px(9.5))
                                                .font_weight(if is_sel { FontWeight::BOLD } else { FontWeight::NORMAL })
                                                .text_color(if is_sel { hex_rgb(0x8ab4ff) } else { TEXT_TERTIARY })
                                                .on_click(move |_ev, _window, cx| {
                                                    let g = g_name.clone();
                                                    app_grp.update(cx, |this, cx| {
                                                        if this.users.new_user.selected_groups.contains(&g) {
                                                            this.users.new_user.selected_groups.retain(|x| x != &g);
                                                        } else {
                                                            this.users.new_user.selected_groups.push(g);
                                                        }
                                                        cx.notify();
                                                    });
                                                })
                                                .child(if is_sel { format!("✓ {}", grp) } else { format!("+ {}", grp) })
                                        })),
                                ),
                        )
                        // Home directory switch
                        .child(
                            div()
                                .id("btn-toggle-new-home")
                                .p(px(8.0))
                                .bg(BG_CONTROL)
                                .border_1()
                                .border_color(BORDER_DEFAULT)
                                .rounded_sm()
                                .cursor_pointer()
                                .hover(|s| s.bg(BG_ROW_HOVER))
                                .on_click(move |_ev, _window, cx| {
                                    app_home.update(cx, |this, cx| {
                                        this.users.new_user.create_home = !current_home;
                                        cx.notify();
                                    });
                                })
                                .flex()
                                .items_center()
                                .justify_between()
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.0))
                                        .text_color(TEXT_SECONDARY)
                                        .child("Create user home directory (/home/<username>) with skeleton dotfiles"),
                                )
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.0))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(if current_home { OK } else { TEXT_MUTED })
                                        .child(if current_home { "ENABLED" } else { "DISABLED" }),
                                ),
                        ),
                )
                // Footer
                .child(
                    div()
                        .h(px(46.0))
                        .flex_none()
                        .flex()
                        .items_center()
                        .justify_between()
                        .px(px(16.0))
                        .bg(BG_APP)
                        .border_t_1()
                        .border_color(BORDER_PANEL)
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(9.5))
                                .text_color(TEXT_FAINT)
                                .child("Executes useradd & usermod -aG under sudo"),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(8.0))
                                .child(
                                    div()
                                        .id("btn-create-user-confirm")
                                        .px(px(12.0))
                                        .py(px(5.0))
                                        .bg(OK)
                                        .rounded_sm()
                                        .cursor_pointer()
                                        .hover(|s| s.bg(hex_rgb(0x34d399)))
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.5))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(rgb(0x0a0a0c))
                                        .on_click(move |_ev, _window, cx| {
                                            app_submit.update(cx, |this, cx| {
                                                this.user_create(cx);
                                            });
                                        })
                                        .child("PROVISION ACCOUNT"),
                                ),
                        ),
                ),
        )
}
