use gpui_kit::*;
use crate::theme::*;
use crate::app::CrowApp;
use crate::components::icons::{TablerIcon, tabler_icon};
use super::models::SystemUserRecord;

pub fn ssh_attach_modal(
    user: &SystemUserRecord,
    app_data: &CrowApp,
    app: Entity<CrowApp>,
) -> impl IntoElement {
    let app_close = app.clone();
    let app_backdrop = app.clone();
    let uname = user.username.clone();
    let target_auth_path = if user.username == "root" {
        "/root/.ssh/authorized_keys".to_string()
    } else {
        format!("{}/.ssh/authorized_keys", user.home_dir)
    };

    div()
        .id("ssh-attach-modal-backdrop")
        .absolute()
        .inset_0()
        .bg(hex_rgba(0x000000, 0.65))
        .flex()
        .items_center()
        .justify_center()
        .on_click(move |_ev, _window, cx| {
            app_backdrop.update(cx, |this, cx| {
                this.close_ssh_attach_modal(cx);
            });
        })
        .child(
            div()
                .id("ssh-attach-modal-panel")
                .w(px(640.0))
                .max_h(px(580.0))
                .bg(BG_PANEL)
                .border_1()
                .border_color(BORDER_PANEL)
                .rounded_md()
                .flex()
                .flex_col()
                .on_click(|_ev, _window, _cx| {
                    // prevent click bubbling to backdrop
                })
                // Modal Header
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
                                .child(tabler_icon(TablerIcon::Key).size(px(16.0)).text_color(hex_rgb(0x8ab4ff)))
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(12.0))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(TEXT_MAX)
                                        .child(format!("SSH KEYCHAIN ATTACHMENT · {}", uname)),
                                )
                                .child(
                                    div()
                                        .bg(hex_rgba(0x8ab4ff, 0.12))
                                        .text_color(hex_rgb(0x8ab4ff))
                                        .px(px(6.0))
                                        .py(px(1.5))
                                        .rounded_sm()
                                        .font_family(FONT_MONO)
                                        .text_size(px(9.0))
                                        .font_weight(FontWeight::BOLD)
                                        .child(format!("{} ATTACHED", user.authorized_keys.len())),
                                ),
                        )
                        .child(
                            div()
                                .id("btn-close-ssh-modal")
                                .p(px(4.0))
                                .rounded_sm()
                                .cursor_pointer()
                                .hover(|s| s.bg(BG_ROW_HOVER))
                                .on_click(move |_ev, _window, cx| {
                                    app_close.update(cx, |this, cx| {
                                        this.close_ssh_attach_modal(cx);
                                    });
                                })
                                .child(tabler_icon(TablerIcon::X).size(px(14.0)).text_color(TEXT_MUTED)),
                        ),
                )
                // Target File Banner
                .child(
                    div()
                        .px(px(16.0))
                        .py(px(8.0))
                        .bg(hex_rgba(0x000000, 0.35))
                        .border_b_1()
                        .border_color(BORDER_ROW)
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(10.0))
                                .text_color(TEXT_TERTIARY)
                                .child(format!("TARGET FILE: {}", target_auth_path)),
                        )
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(9.0))
                                .text_color(OK)
                                .child("PERMISSIONS: 0600 (SECURE)"),
                        ),
                )
                // Scrollable Body
                .child(
                    div()
                        .id("ssh-attach-scroll")
                        .flex_1()
                        .overflow_y_scroll()
                        .p(px(16.0))
                        .flex()
                        .flex_col()
                        .gap(px(16.0))
                        // SECTION 1: Currently Authorized Keys for this User
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(8.0))
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.5))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(TEXT_PRIMARY)
                                        .child("CURRENTLY AUTHORIZED KEYS FOR USER"),
                                )
                                .child(if user.authorized_keys.is_empty() {
                                    div()
                                        .p(px(12.0))
                                        .bg(hex_rgba(0xffffff, 0.02))
                                        .border_1()
                                        .border_color(BORDER_ROW)
                                        .rounded_sm()
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.0))
                                        .text_color(TEXT_MUTED)
                                        .child("No SSH keys currently installed for this user. Attach a key below.")
                                        .into_any_element()
                                } else {
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap(px(6.0))
                                        .children(user.authorized_keys.iter().map(|k| {
                                            let app_rv = app.clone();
                                            let u_target = user.username.clone();
                                            let kid = k.id.clone();
                                            div()
                                                .p(px(8.0))
                                                .bg(hex_rgba(0xffffff, 0.03))
                                                .border_1()
                                                .border_color(BORDER_DEFAULT)
                                                .rounded_sm()
                                                .flex()
                                                .items_center()
                                                .justify_between()
                                                .child(
                                                    div()
                                                        .flex()
                                                        .flex_col()
                                                        .gap(px(2.0))
                                                        .child(
                                                            div()
                                                                .flex()
                                                                .items_center()
                                                                .gap(px(6.0))
                                                                .child(
                                                                    div()
                                                                        .px(px(4.0))
                                                                        .py(px(1.0))
                                                                        .bg(hex_rgba(0x38bdf8, 0.15))
                                                                        .rounded_sm()
                                                                        .font_family(FONT_MONO)
                                                                        .text_size(px(8.5))
                                                                        .font_weight(FontWeight::BOLD)
                                                                        .text_color(hex_rgb(0x38bdf8))
                                                                        .child(k.algorithm.clone()),
                                                                )
                                                                .child(
                                                                    div()
                                                                        .font_family(FONT_MONO)
                                                                        .text_size(px(11.0))
                                                                        .font_weight(FontWeight::BOLD)
                                                                        .text_color(TEXT_MAX)
                                                                        .child(k.name.clone()),
                                                                ),
                                                        )
                                                        .child(
                                                            div()
                                                                .font_family(FONT_MONO)
                                                                .text_size(px(9.0))
                                                                .text_color(TEXT_FAINT)
                                                                .child(k.fingerprint.clone()),
                                                        ),
                                                )
                                                .child(
                                                    div()
                                                        .id(ElementId::NamedInteger(format!("btn-revoke-{}", kid).into(), 0))
                                                        .px(px(8.0))
                                                        .py(px(3.0))
                                                        .bg(hex_rgba(0xef4444, 0.12))
                                                        .border_1()
                                                        .border_color(hex_rgba(0xef4444, 0.35))
                                                        .rounded_sm()
                                                        .cursor_pointer()
                                                        .hover(|s| s.bg(hex_rgba(0xef4444, 0.25)))
                                                        .font_family(FONT_MONO)
                                                        .text_size(px(9.0))
                                                        .font_weight(FontWeight::BOLD)
                                                        .text_color(CRIT)
                                                        .on_click(move |_ev, _window, cx| {
                                                            let u = u_target.clone();
                                                            let k = kid.clone();
                                                            app_rv.update(cx, |this, cx| {
                                                                this.revoke_ssh_key_from_user(&u, &k, cx);
                                                            });
                                                        })
                                                        .child("REVOKE"),
                                                )
                                        }))
                                        .into_any_element()
                                }),
                        )
                        // SECTION 2: Available Keys from Crow Keychain
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(8.0))
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
                                                .text_color(TEXT_PRIMARY)
                                                .child("SELECT FROM CROW SSH KEYCHAIN"),
                                        )
                                        .child(
                                            div()
                                                .font_family(FONT_MONO)
                                                .text_size(px(9.0))
                                                .text_color(TEXT_FAINT)
                                                .child(format!("{} ENROLLED VAULT KEYS", app_data.enrolled_keys.len())),
                                        ),
                                )
                                .child(if app_data.enrolled_keys.is_empty() {
                                    div()
                                        .p(px(12.0))
                                        .bg(hex_rgba(0xffffff, 0.02))
                                        .border_1()
                                        .border_color(BORDER_ROW)
                                        .rounded_sm()
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.0))
                                        .text_color(TEXT_MUTED)
                                        .child("No enrolled keys found in Crow Vault. Go to Settings > SSH Keys to generate or scan keys.")
                                } else {
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap(px(6.0))
                                        .children(app_data.enrolled_keys.iter().map(|k| {
                                            let is_attached = user.authorized_keys.iter().any(|ak| ak.id == k.id || ak.fingerprint == k.fingerprint);
                                            let app_at = app.clone();
                                            let u_target = user.username.clone();
                                            let kid = k.id.clone();
                                            let kname = k.name.clone();
                                            let kalgo = k.algorithm.clone();
                                            let kfp = k.fingerprint.clone();
                                            let kcomment = k.comment.clone();
                                            let kpub = k.public_key.clone();

                                            div()
                                                .p(px(10.0))
                                                .bg(BG_APP)
                                                .border_1()
                                                .border_color(if is_attached { hex_rgba(0x10b981, 0.3) } else { BORDER_DEFAULT })
                                                .rounded_sm()
                                                .flex()
                                                .items_center()
                                                .justify_between()
                                                .child(
                                                    div()
                                                        .flex()
                                                        .flex_col()
                                                        .gap(px(3.0))
                                                        .child(
                                                            div()
                                                                .flex()
                                                                .items_center()
                                                                .gap(px(8.0))
                                                                .child(
                                                                    div()
                                                                        .px(px(5.0))
                                                                        .py(px(1.5))
                                                                        .bg(if k.algorithm.to_lowercase().contains("ed25519") {
                                                                            hex_rgba(0x38bdf8, 0.15)
                                                                        } else {
                                                                            hex_rgba(0xfbbf24, 0.15)
                                                                        })
                                                                        .rounded_sm()
                                                                        .font_family(FONT_MONO)
                                                                        .text_size(px(8.5))
                                                                        .font_weight(FontWeight::BOLD)
                                                                        .text_color(if k.algorithm.to_lowercase().contains("ed25519") {
                                                                            hex_rgb(0x38bdf8)
                                                                        } else {
                                                                            hex_rgb(0xfbbf24)
                                                                        })
                                                                        .child(k.algorithm.clone()),
                                                                )
                                                                .child(
                                                                    div()
                                                                        .font_family(FONT_MONO)
                                                                        .text_size(px(11.0))
                                                                        .font_weight(FontWeight::BOLD)
                                                                        .text_color(TEXT_MAX)
                                                                        .child(k.name.clone()),
                                                                )
                                                                .children(if let Some(comm) = &k.comment {
                                                                    Some(
                                                                        div()
                                                                            .font_family(FONT_MONO)
                                                                            .text_size(px(9.5))
                                                                            .text_color(TEXT_MUTED)
                                                                            .child(format!("({})", comm)),
                                                                    )
                                                                } else {
                                                                    None
                                                                }),
                                                        )
                                                        .child(
                                                            div()
                                                                .font_family(FONT_MONO)
                                                                .text_size(px(9.0))
                                                                .text_color(TEXT_FAINT)
                                                                .child(k.fingerprint.clone()),
                                                        ),
                                                )
                                                .child(if is_attached {
                                                    div()
                                                        .px(px(8.0))
                                                        .py(px(4.0))
                                                        .bg(OK_BG)
                                                        .rounded_sm()
                                                        .font_family(FONT_MONO)
                                                        .text_size(px(9.0))
                                                        .font_weight(FontWeight::BOLD)
                                                        .text_color(OK)
                                                        .child("✓ ATTACHED")
                                                        .into_any_element()
                                                } else {
                                                    div()
                                                        .id(ElementId::NamedInteger(format!("btn-attach-{}", kid).into(), 0))
                                                        .px(px(10.0))
                                                        .py(px(4.0))
                                                        .bg(hex_rgba(0x8ab4ff, 0.15))
                                                        .border_1()
                                                        .border_color(hex_rgba(0x8ab4ff, 0.4))
                                                        .rounded_sm()
                                                        .cursor_pointer()
                                                        .hover(|s| s.bg(hex_rgba(0x8ab4ff, 0.3)))
                                                        .font_family(FONT_MONO)
                                                        .text_size(px(9.5))
                                                        .font_weight(FontWeight::BOLD)
                                                        .text_color(hex_rgb(0x8ab4ff))
                                                        .on_click(move |_ev, _window, cx| {
                                                            let u = u_target.clone();
                                                            let key_summary = super::models::UserSshKeySummary {
                                                                id: kid.clone(),
                                                                name: kname.clone(),
                                                                algorithm: kalgo.clone(),
                                                                fingerprint: kfp.clone(),
                                                                comment: kcomment.clone(),
                                                                key_preview: kpub.clone(),
                                                                added_at: chrono::Utc::now().format("%Y-%m-%d %H:%M").to_string(),
                                                            };
                                                            app_at.update(cx, |this, cx| {
                                                                this.attach_key_summary_to_user(&u, key_summary, cx);
                                                            });
                                                        })
                                                        .child("+ ATTACH KEY")
                                                        .into_any_element()
                                                })
                                        }))
                                }),
                        ),
                ),
        )
}
