use gpui_kit::component::input::{Editor, EditorState};
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::*;

use super::unit_collector::UnitDetails;
use super::service_inspector::WAND;
use crate::app::CrowApp;
use crate::components::icons::{tabler_icon, TablerIcon};
use crate::theme::*;
use crate::vault::ChangeRecord;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnitEditorTab {
    CombinedView,
    OverrideFile,
    CustomUnitFile,
}

#[derive(Clone, Debug, PartialEq)]
pub enum DryRunState {
    Idle,
    Running,
    Passed(String),
    Failed(String),
}

#[derive(Clone, Debug, PartialEq)]
pub enum ApplyState {
    Idle,
    Applying,
    Success(String),
    Error(String),
}

pub struct UnitEditorInputs {
    pub editor: Entity<EditorState>,
    pub _change_sub: Subscription,
}

pub struct UnitEditorState {
    pub _server_id: String,
    pub server_name: String,
    pub unit_name: String,
    pub details: Option<UnitDetails>,
    pub loading: bool,
    pub load_error: Option<String>,
    pub active_tab: UnitEditorTab,
    pub current_editor_text: String,
    pub pending_content: Option<String>,
    pub original_content: String,
    pub dry_run_state: DryRunState,
    pub apply_state: ApplyState,
    pub restart_after_apply: bool,
    pub show_history: bool,
    pub recent_changes: Vec<ChangeRecord>,
    pub rolling_back_id: Option<String>,
}

impl UnitEditorState {
    pub fn new(
        server_id: String,
        server_name: String,
        unit_name: String,
        recent_changes: Vec<ChangeRecord>,
    ) -> Self {
        Self {
            _server_id: server_id,
            server_name,
            unit_name,
            details: None,
            loading: true,
            load_error: None,
            active_tab: UnitEditorTab::CombinedView,
            current_editor_text: String::new(),
            pending_content: None,
            original_content: String::new(),
            dry_run_state: DryRunState::Idle,
            apply_state: ApplyState::Idle,
            restart_after_apply: true,
            show_history: false,
            recent_changes,
            rolling_back_id: None,
        }
    }

    pub fn target_file_path(&self) -> String {
        match self.active_tab {
            UnitEditorTab::OverrideFile => {
                format!("/etc/systemd/system/{}.d/override.conf", self.unit_name)
            }
            UnitEditorTab::CustomUnitFile => {
                if let Some(ref d) = self.details {
                    if d.is_custom && !d.fragment_path.is_empty() {
                        d.fragment_path.clone()
                    } else {
                        format!("/etc/systemd/system/{}", self.unit_name)
                    }
                } else {
                    format!("/etc/systemd/system/{}", self.unit_name)
                }
            }
            UnitEditorTab::CombinedView => String::new(),
        }
    }
}

pub fn unit_editor_modal(
    state: &UnitEditorState,
    inputs: &UnitEditorInputs,
    app: Entity<CrowApp>,
) -> impl IntoElement {
    let (app_scrim, app_close, app_verify, app_apply, app_toggle_restart, app_toggle_hist) = (
        app.clone(),
        app.clone(),
        app.clone(),
        app.clone(),
        app.clone(),
        app.clone(),
    );

    let is_loading = state.loading;
    let is_editable = matches!(state.active_tab, UnitEditorTab::OverrideFile | UnitEditorTab::CustomUnitFile);
    let is_vendor = state.details.as_ref().map_or(false, |d| d.is_vendor);
    let fragment_path = state.details.as_ref().map(|d| d.fragment_path.as_str()).unwrap_or("");
    let has_custom_tab = state.details.as_ref().map_or(false, |d| d.is_custom);

    div()
        .id("unit-editor-scrim")
        .absolute()
        .inset_0()
        .occlude()
        .bg(hex_rgba(0x060709, 0.75))
        .flex()
        .items_center()
        .justify_center()
        .on_click(move |_ev, _window, cx| {
            app_scrim.update(cx, |this, cx| this.close_unit_editor(cx));
        })
        .child(
            div()
                .id("unit-editor-card")
                .occlude()
                .on_click(|_ev, _window, cx| cx.stop_propagation())
                .w(px(920.0))
                .h(px(660.0))
                .bg(BG_PANEL)
                .border_1()
                .border_color(BORDER_DEFAULT)
                .shadow_lg()
                .flex()
                .flex_col()
                .font_family(FONT_MONO)
                // 1. Modal Header
                .child(
                    div()
                        .h(px(46.0))
                        .flex_none()
                        .flex()
                        .items_center()
                        .gap(px(10.0))
                        .px(px(16.0))
                        .border_b_1()
                        .border_color(BORDER_PANEL)
                        .child(tabler_icon(TablerIcon::FileText).size(px(18.0)).text_color(OK))
                        .child(
                            div()
                                .text_size(px(13.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(TEXT_PRIMARY)
                                .child(format!("{} · {}", state.unit_name, state.server_name)),
                        )
                        .child(
                            div()
                                .px(px(6.0))
                                .py(px(2.0))
                                .bg(if is_vendor { hex_rgba(0x38bdf8, 0.12) } else { hex_rgba(0xa855f7, 0.12) })
                                .border_1()
                                .border_color(if is_vendor { hex_rgb(0x38bdf8) } else { hex_rgb(0xa855f7) })
                                .text_color(if is_vendor { hex_rgb(0x38bdf8) } else { hex_rgb(0xa855f7) })
                                .text_size(px(9.5))
                                .font_weight(FontWeight::BOLD)
                                .child(if is_vendor { "VENDOR (DROP-IN SAFE)" } else { "CUSTOM UNIT (/etc)" }),
                        )
                        .child(
                            div()
                                .text_size(px(10.0))
                                .text_color(TEXT_DIM)
                                .child(fragment_path.to_string()),
                        )
                        .child(div().flex_1())
                        .child(
                            div()
                                .id("btn-close-unit-editor")
                                .p(px(4.0))
                                .cursor_pointer()
                                .hover(|s| s.bg(BG_ROW_HOVER))
                                .on_click(move |_ev, _window, cx| {
                                    app_close.update(cx, |this, cx| this.close_unit_editor(cx));
                                })
                                .child(tabler_icon(TablerIcon::X).size(px(16.0)).text_color(TEXT_DIM)),
                        ),
                )
                // 2. Tab Bar & Action Controls
                .child(
                    div()
                        .h(px(38.0))
                        .flex_none()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .px(px(14.0))
                        .bg(BG_SUBHEAD)
                        .border_b_1()
                        .border_color(BORDER_PANEL)
                        // Tabs
                        .child({
                            let app_tab = app.clone();
                            render_tab(
                                "tab-unit-cat",
                                "Full Definition (systemctl cat)",
                                state.active_tab == UnitEditorTab::CombinedView,
                                move |_ev, _window, cx| {
                                    app_tab.update(cx, |this, cx| {
                                        this.switch_unit_editor_tab(UnitEditorTab::CombinedView, cx);
                                    });
                                },
                            )
                        })
                        .child({
                            let app_tab = app.clone();
                            render_tab(
                                "tab-unit-override",
                                "Drop-in Override (override.conf)",
                                state.active_tab == UnitEditorTab::OverrideFile,
                                move |_ev, _window, cx| {
                                    app_tab.update(cx, |this, cx| {
                                        this.switch_unit_editor_tab(UnitEditorTab::OverrideFile, cx);
                                    });
                                },
                            )
                        })
                        .children(if has_custom_tab {
                            let app_tab = app.clone();
                            Some(render_tab(
                                "tab-unit-custom",
                                "Base Unit File",
                                state.active_tab == UnitEditorTab::CustomUnitFile,
                                move |_ev, _window, cx| {
                                    app_tab.update(cx, |this, cx| {
                                        this.switch_unit_editor_tab(UnitEditorTab::CustomUnitFile, cx);
                                    });
                                },
                            ))
                        } else {
                            None
                        })
                        .child(div().flex_1())
                        // Rollback / Revision History toggle
                        .child({
                            let hist_count = state.recent_changes.len();
                            div()
                                .id("btn-toggle-unit-history")
                                .flex()
                                .items_center()
                                .gap(px(5.0))
                                .px(px(8.0))
                                .py(px(4.0))
                                .bg(if state.show_history { BG_CONTROL_ALT } else { BG_CONTROL })
                                .border_1()
                                .border_color(if state.show_history { OK } else { BORDER_DEFAULT })
                                .cursor_pointer()
                                .hover(|s| s.bg(BG_ROW_HOVER))
                                .text_size(px(10.0))
                                .text_color(if state.show_history { OK } else { TEXT_SECONDARY })
                                .on_click(move |_ev, _window, cx| {
                                    app_toggle_hist.update(cx, |this, cx| {
                                        this.toggle_unit_editor_history(cx);
                                    });
                                })
                                .child(tabler_icon(TablerIcon::Clock).size(px(12.0)))
                                .child(format!("Revisions ({hist_count})"))
                        })
                        // Verify (Dry Run) button
                        .children(if is_editable {
                            let is_verifying = matches!(state.dry_run_state, DryRunState::Running);
                            Some(
                                div()
                                    .id("btn-unit-dry-run")
                                    .flex()
                                    .items_center()
                                    .gap(px(5.0))
                                    .px(px(10.0))
                                    .py(px(4.0))
                                    .bg(BG_CONTROL)
                                    .border_1()
                                    .border_color(BORDER_DEFAULT)
                                    .cursor_pointer()
                                    .hover(|s| s.bg(BG_ROW_HOVER))
                                    .text_size(px(10.0))
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(WARN)
                                    .on_click(move |_ev, _window, cx| {
                                        app_verify.update(cx, |this, cx| this.run_unit_dry_run(cx));
                                    })
                                    .child(if is_verifying {
                                        tabler_icon(TablerIcon::Refresh).size(px(12.0))
                                    } else {
                                        tabler_icon(TablerIcon::ShieldCheck).size(px(12.0))
                                    })
                                    .child(if is_verifying { "Verifying…" } else { "Dry Run (Verify)" }),
                            )
                        } else {
                            None
                        })
                        // Restart toggle checkbox
                        .children(if is_editable {
                            let checked = state.restart_after_apply;
                            Some(
                                div()
                                    .id("btn-toggle-restart-after")
                                    .flex()
                                    .items_center()
                                    .gap(px(5.0))
                                    .px(px(6.0))
                                    .py(px(4.0))
                                    .cursor_pointer()
                                    .hover(|s| s.text_color(TEXT_PRIMARY))
                                    .text_size(px(10.0))
                                    .text_color(if checked { TEXT_PRIMARY } else { TEXT_DIM })
                                    .on_click(move |_ev, _window, cx| {
                                        app_toggle_restart.update(cx, |this, cx| {
                                            this.toggle_unit_restart_after_apply(cx);
                                        });
                                    })
                                    .child(
                                        div()
                                            .size(px(12.0))
                                            .border_1()
                                            .border_color(if checked { OK } else { BORDER_DEFAULT })
                                            .bg(if checked { OK.opacity(0.2) } else { hex_rgba(0, 0.0) })
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .children(checked.then(|| {
                                                tabler_icon(TablerIcon::Check).size(px(10.0)).text_color(OK)
                                            })),
                                    )
                                    .child("Restart on apply"),
                            )
                        } else {
                            None
                        })
                        // Apply & Reload button
                        .children(if is_editable {
                            let is_applying = matches!(state.apply_state, ApplyState::Applying);
                            Some(
                                div()
                                    .id("btn-unit-apply")
                                    .flex()
                                    .items_center()
                                    .gap(px(5.0))
                                    .px(px(12.0))
                                    .py(px(5.0))
                                    .bg(OK)
                                    .cursor_pointer()
                                    .hover(|s| s.bg(hex_rgb(0x22c55e)))
                                    .text_size(px(10.5))
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(hex_rgb(0x0a0a0c))
                                    .on_click(move |_ev, _window, cx| {
                                        app_apply.update(cx, |this, cx| this.apply_unit_editor_changes(cx));
                                    })
                                    .child(if is_applying {
                                        tabler_icon(TablerIcon::Refresh).size(px(12.0))
                                    } else {
                                        tabler_icon(TablerIcon::PlayerPlay).size(px(12.0))
                                    })
                                    .child(if is_applying { "Applying…" } else { "Apply & Reload" }),
                            )
                        } else {
                            None
                        }),
                )
                // 3. Body: Editor or Full View
                .child(
                    div()
                        .flex_1()
                        .min_h(px(0.0))
                        .flex()
                        .flex_col()
                        .bg(BG_APP)
                        .children(if is_loading {
                            Some(
                                div()
                                    .flex_1()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .gap(px(8.0))
                                    .text_size(px(12.0))
                                    .text_color(TEXT_DIM)
                                    .child(tabler_icon(TablerIcon::Refresh).size(px(16.0)))
                                    .child("Loading unit definition from host…")
                                    .into_any_element(),
                            )
                        } else if let Some(ref err) = state.load_error {
                            Some(
                                div()
                                    .flex_1()
                                    .flex()
                                    .flex_col()
                                    .items_center()
                                    .justify_center()
                                    .gap(px(8.0))
                                    .p(px(24.0))
                                    .child(tabler_icon(TablerIcon::AlertTriangle).size(px(24.0)).text_color(CRIT))
                                    .child(div().text_size(px(12.0)).text_color(CRIT).child(err.clone()))
                                    .into_any_element(),
                            )
                        } else if state.active_tab == UnitEditorTab::CombinedView {
                            Some(render_combined_cat_view(state).into_any_element())
                        } else {
                            Some(
                                div()
                                    .flex_1()
                                    .min_h(px(0.0))
                                    .p(px(8.0))
                                    .child(
                                        Editor::new(&inputs.editor)
                                            .h_full()
                                            .appearance(false)
                                            .bordered(false)
                                            .font_family(FONT_MONO)
                                            .text_size(px(11.5)),
                                    )
                                    .into_any_element(),
                            )
                        }),
                )
                // 4. Verification / Status Banner
                .children(render_feedback_banner(state))
                // 5. Revision History Drawer (when toggled open)
                .children(if state.show_history {
                    Some(render_history_drawer(state, app.clone()))
                } else {
                    None
                }),
        )
}

fn render_tab(
    id: &'static str,
    label: &'static str,
    active: bool,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .id(id)
        .px(px(10.0))
        .py(px(4.0))
        .border_1()
        .border_color(if active { TEXT_PRIMARY } else { BORDER_DEFAULT })
        .bg(if active { BG_CONTROL_ALT } else { BG_CONTROL })
        .text_size(px(10.5))
        .font_weight(if active { FontWeight::BOLD } else { FontWeight::NORMAL })
        .text_color(if active { TEXT_PRIMARY } else { TEXT_DIM })
        .cursor_pointer()
        .hover(|s| s.bg(BG_ROW_HOVER).text_color(TEXT_PRIMARY))
        .on_click(on_click)
        .child(label)
}

fn render_combined_cat_view(state: &UnitEditorState) -> impl IntoElement {
    let lines: Vec<String> = state
        .details
        .as_ref()
        .map(|d| d.cat_content.lines().map(ToString::to_string).collect())
        .unwrap_or_default();

    div()
        .id("unit-cat-scroll")
        .flex_1()
        .min_h(px(0.0))
        .overflow_y_scrollbar()
        .p(px(12.0))
        .children(lines.into_iter().map(|line| {
            let is_header = line.starts_with("# /");
            let is_comment = line.starts_with('#') || line.starts_with(';');
            let is_section = line.starts_with('[') && line.ends_with(']');

            let color = if is_header {
                OK
            } else if is_section {
                WAND
            } else if is_comment {
                TEXT_DIMMER
            } else if line.contains('=') {
                TEXT_SECONDARY
            } else {
                TEXT_PRIMARY
            };

            div()
                .font_family(FONT_MONO)
                .text_size(px(11.0))
                .line_height(px(17.0))
                .font_weight(if is_header || is_section { FontWeight::BOLD } else { FontWeight::NORMAL })
                .text_color(color)
                .bg(if is_header { hex_rgba(0x4ade80, 0.06) } else { hex_rgba(0, 0.0) })
                .px(if is_header { px(6.0) } else { px(0.0) })
                .py(if is_header { px(2.0) } else { px(0.0) })
                .child(if line.is_empty() { " ".to_string() } else { line })
        }))
}

fn render_feedback_banner(state: &UnitEditorState) -> Option<Div> {
    match (&state.dry_run_state, &state.apply_state) {
        (_, ApplyState::Applying) => Some(
            div()
                .h(px(32.0))
                .flex_none()
                .flex()
                .items_center()
                .gap(px(8.0))
                .px(px(14.0))
                .bg(BG_CONTROL)
                .border_t_1()
                .border_color(BORDER_PANEL)
                .child(tabler_icon(TablerIcon::Refresh).size(px(14.0)).text_color(OK))
                .child(
                    div()
                        .text_size(px(10.5))
                        .text_color(TEXT_SECONDARY)
                        .child("Applying changes atomically, reloading systemd, and validating status…"),
                ),
        ),
        (_, ApplyState::Success(msg)) => Some(
            div()
                .h(px(32.0))
                .flex_none()
                .flex()
                .items_center()
                .gap(px(8.0))
                .px(px(14.0))
                .bg(OK_BG)
                .border_t_1()
                .border_color(OK)
                .child(tabler_icon(TablerIcon::Check).size(px(14.0)).text_color(OK))
                .child(div().text_size(px(10.5)).font_weight(FontWeight::BOLD).text_color(OK).child(msg.clone())),
        ),
        (_, ApplyState::Error(err)) => Some(
            div()
                .flex_none()
                .flex()
                .items_center()
                .gap(px(8.0))
                .p(px(10.0))
                .bg(CRIT_BG)
                .border_t_1()
                .border_color(CRIT)
                .child(tabler_icon(TablerIcon::AlertTriangle).size(px(14.0)).text_color(CRIT))
                .child(div().text_size(px(10.5)).text_color(CRIT).child(err.clone())),
        ),
        (DryRunState::Passed(out), _) => Some(
            div()
                .flex_none()
                .flex()
                .items_center()
                .gap(px(8.0))
                .p(px(8.0))
                .px(px(14.0))
                .bg(hex_rgba(0x4ade80, 0.1))
                .border_t_1()
                .border_color(OK)
                .child(tabler_icon(TablerIcon::Check).size(px(14.0)).text_color(OK))
                .child(
                    div()
                        .text_size(px(10.5))
                        .text_color(OK)
                        .child(format!("Dry-run verified: {out}")),
                ),
        ),
        (DryRunState::Failed(err), _) => Some(
            div()
                .flex_none()
                .flex()
                .items_start()
                .gap(px(8.0))
                .p(px(8.0))
                .px(px(14.0))
                .bg(hex_rgba(0xf87171, 0.1))
                .border_t_1()
                .border_color(CRIT)
                .child(tabler_icon(TablerIcon::AlertTriangle).size(px(14.0)).text_color(CRIT))
                .child(
                    div()
                        .flex_1()
                        .text_size(px(10.5))
                        .line_height(px(15.0))
                        .text_color(CRIT)
                        .child(format!("Verification failed: {err}")),
                ),
        ),
        _ => None,
    }
}

fn render_history_drawer(state: &UnitEditorState, app: Entity<CrowApp>) -> impl IntoElement {
    let recent = &state.recent_changes;
    let unit = state.unit_name.clone();

    div()
        .h(px(150.0))
        .flex_none()
        .flex()
        .flex_col()
        .bg(BG_SUBHEAD)
        .border_t_1()
        .border_color(BORDER_PANEL)
        .child(
            div()
                .h(px(26.0))
                .flex_none()
                .flex()
                .items_center()
                .px(px(12.0))
                .bg(BG_PANEL)
                .border_b_1()
                .border_color(BORDER_PANEL)
                .text_size(px(9.5))
                .font_weight(FontWeight::BOLD)
                .text_color(TEXT_DIM)
                .child(format!("REVISION HISTORY & 1-CLICK ROLLBACK FOR {unit}")),
        )
        .child(
            div()
                .id("unit-history-scroll")
                .flex_1()
                .min_h(px(0.0))
                .overflow_y_scrollbar()
                .children(if recent.is_empty() {
                    Some(
                        div()
                            .p(px(12.0))
                            .text_size(px(10.5))
                            .text_color(TEXT_DIMMER)
                            .child("No recorded changes for this service yet.")
                            .into_any_element(),
                    )
                } else {
                    None
                })
                .children(recent.iter().map(|rec| {
                    let rec_id = rec.id.clone();
                    let is_rolling_back = state.rolling_back_id.as_deref() == Some(&rec_id);
                    let (dot_color, label) = match rec.outcome.as_str() {
                        "success" => (OK, "SUCCESS"),
                        "failed" => (CRIT, "FAILED"),
                        _ => (WARN, "PENDING"),
                    };
                    let app_rb = app.clone();
                    let rec_owned = rec.clone();

                    div()
                        .flex()
                        .items_center()
                        .gap(px(10.0))
                        .px(px(12.0))
                        .py(px(5.0))
                        .border_b_1()
                        .border_color(BORDER_PANEL)
                        .text_size(px(10.0))
                        .child(div().size(px(6.0)).rounded_full().bg(dot_color).flex_none())
                        .child(
                            div()
                                .w(px(130.0))
                                .text_color(TEXT_DIM)
                                .child(rec.started_at.get(0..19).unwrap_or(&rec.started_at).to_string()),
                        )
                        .child(
                            div()
                                .w(px(120.0))
                                .text_color(TEXT_PRIMARY)
                                .font_weight(FontWeight::BOLD)
                                .child(rec.action_kind.replace("service_unit_", "").to_uppercase()),
                        )
                        .child(
                            div()
                                .flex_1()
                                .text_color(TEXT_SECONDARY)
                                .child(rec.blast_radius.clone().unwrap_or_default()),
                        )
                        .child(
                            div()
                                .w(px(60.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(dot_color)
                                .child(label),
                        )
                        .child(
                            div()
                                .id(ElementId::Name(format!("btn-rb-{}", rec.id).into()))
                                .px(px(8.0))
                                .py(px(2.0))
                                .bg(BG_CONTROL)
                                .border_1()
                                .border_color(WARN)
                                .cursor_pointer()
                                .hover(|s| s.bg(WARN.opacity(0.15)))
                                .text_size(px(9.5))
                                .font_weight(FontWeight::BOLD)
                                .text_color(WARN)
                                .on_click(move |_ev, _window, cx| {
                                    let r = rec_owned.clone();
                                    app_rb.update(cx, |this, cx| {
                                        this.rollback_unit_action(&r, cx);
                                    });
                                })
                                .child(if is_rolling_back { "Restoring…" } else { "Rollback" }),
                        )
                        .into_any_element()
                })),
        )
}

#[cfg(test)]
mod tests {
    // Avoid importing GPUI's test macro
    use super::{UnitEditorState, UnitEditorTab};

    #[test]
    fn test_unit_editor_target_path() {
        let mut state = UnitEditorState::new(
            "srv-1".into(),
            "web-1".into(),
            "nginx.service".into(),
            Vec::new(),
        );

        state.active_tab = UnitEditorTab::OverrideFile;
        assert_eq!(
            state.target_file_path(),
            "/etc/systemd/system/nginx.service.d/override.conf"
        );

        state.active_tab = UnitEditorTab::CombinedView;
        assert_eq!(state.target_file_path(), "");

        state.active_tab = UnitEditorTab::CustomUnitFile;
        assert_eq!(state.target_file_path(), "/etc/systemd/system/nginx.service");
    }
}
