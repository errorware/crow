use gpui_kit::*;
use crate::theme::*;
use crate::app::CrowApp;
use crate::components::icons::{TablerIcon, tabler_icon};
use crate::views::config::state::ConfigsState;

#[derive(Clone, Debug, PartialEq)]
pub struct CronJobDef {
    pub id: String,
    pub enabled: bool,
    pub minute: String,
    pub hour: String,
    pub day_of_month: String,
    pub month: String,
    pub day_of_week: String,
    pub user: String,
    pub command: String,
    pub comment: Option<String>,
    pub is_expanded: bool,
}

impl CronJobDef {
    pub fn new(
        id: impl Into<String>,
        enabled: bool,
        minute: impl Into<String>,
        hour: impl Into<String>,
        day_of_month: impl Into<String>,
        month: impl Into<String>,
        day_of_week: impl Into<String>,
        user: impl Into<String>,
        command: impl Into<String>,
        comment: Option<String>,
    ) -> Self {
        Self {
            id: id.into(),
            enabled,
            minute: minute.into(),
            hour: hour.into(),
            day_of_month: day_of_month.into(),
            month: month.into(),
            day_of_week: day_of_week.into(),
            user: user.into(),
            command: command.into(),
            comment,
            is_expanded: false,
        }
    }

    #[allow(dead_code)]
    pub fn schedule_expression(&self) -> String {
        format!(
            "{} {} {} {} {}",
            self.minute, self.hour, self.day_of_month, self.month, self.day_of_week
        )
    }

    pub fn human_schedule(&self) -> String {
        human_readable_schedule(
            &self.minute,
            &self.hour,
            &self.day_of_month,
            &self.month,
            &self.day_of_week,
        )
    }
}

pub fn human_readable_schedule(m: &str, h: &str, dom: &str, mon: &str, dow: &str) -> String {
    if m == "*" && h == "*" && dom == "*" && mon == "*" && dow == "*" {
        return "Every minute (* * * * *)".to_string();
    }
    if m.starts_with("*/") && h == "*" && dom == "*" && mon == "*" && dow == "*" {
        let step = &m[2..];
        return format!("Every {} minutes", step);
    }
    if m == "0" && h == "*" && dom == "*" && mon == "*" && dow == "*" {
        return "Every hour on the hour (0 * * * *)".to_string();
    }
    if h == "*" && dom == "*" && mon == "*" && dow == "*" {
        return format!("Every hour at minute {}", m);
    }
    if dom == "*" && mon == "*" && dow == "*" {
        return format!("Every day at {:0>2}:{:0>2}", h, m);
    }
    if dow != "*" && dom == "*" && mon == "*" {
        let day_name = match dow {
            "0" | "7" => "Sunday",
            "1" => "Monday",
            "2" => "Tuesday",
            "3" => "Wednesday",
            "4" => "Thursday",
            "5" => "Friday",
            "6" => "Saturday",
            "1-5" => "Weekday (Mon-Fri)",
            "6,7" | "0,6" => "Weekend (Sat-Sun)",
            _ => dow,
        };
        return format!("Weekly on {} at {:0>2}:{:0>2}", day_name, h, m);
    }
    if dom != "*" && mon == "*" && dow == "*" {
        return format!("Monthly on day {} at {:0>2}:{:0>2}", dom, h, m);
    }

    format!("Schedule: {} {} {} {} {}", m, h, dom, mon, dow)
}

pub fn cron_editor(
    jobs: &[CronJobDef],
    configs: &ConfigsState,
    app: Entity<CrowApp>,
) -> impl IntoElement {
    let file_state = configs.states.get("crontab");
    let is_modified = file_state.map(|s| s.is_modified()).unwrap_or(false);
    let rev_count = file_state.map(|s| s.revisions.len()).unwrap_or(1);
    let active_count = jobs.iter().filter(|j| j.enabled).count();

    let app_apply = app.clone();
    let app_revert = app.clone();
    let app_history = app.clone();
    let app_add = app.clone();

    div()
        .id("cron-editor")
        .flex_1()
        .min_w(px(0.0))
        .h_full()
        .flex()
        .flex_col()
        .bg(BG_APP)
        // 1. Header Bar
        .child(
            div()
                .h(px(40.0))
                .flex_none()
                .flex()
                .items_center()
                .gap(px(10.0))
                .px(px(16.0))
                .bg(BG_PANEL)
                .border_b_1()
                .border_color(BORDER_PANEL)
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(12.5))
                        .font_weight(FontWeight::BOLD)
                        .text_color(TEXT_MAX)
                        .child("crontab"),
                )
                .child(
                    div()
                        .bg(OK_BG)
                        .text_color(OK)
                        .font_family(FONT_MONO)
                        .text_size(px(9.0))
                        .font_weight(FontWeight::BOLD)
                        .px(px(6.0))
                        .py(px(2.0))
                        .rounded_sm()
                        .child(format!("{} ACTIVE SCHEDULES", active_count)),
                )
                .child(if is_modified {
                    div()
                        .bg(WARN_BG)
                        .text_color(WARN)
                        .font_family(FONT_MONO)
                        .text_size(px(9.0))
                        .font_weight(FontWeight::BOLD)
                        .px(px(6.0))
                        .py(px(2.0))
                        .rounded_sm()
                        .child("UNSAVED EDITS")
                } else {
                    div()
                        .bg(OK_BG)
                        .text_color(OK)
                        .font_family(FONT_MONO)
                        .text_size(px(9.0))
                        .font_weight(FontWeight::BOLD)
                        .px(px(6.0))
                        .py(px(2.0))
                        .rounded_sm()
                        .child("IN SYNC")
                })
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(10.0))
                        .text_color(TEXT_TERTIARY)
                        .child("/etc/crontab"),
                )
                .child(div().flex_1())
                // Quick + ADD CRON JOB button
                .child(
                    div()
                        .id("btn-add-cron-job")
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
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(TEXT_SECONDARY)
                        .on_click(move |_ev, _window, cx| {
                            app_add.update(cx, |this, cx| {
                                this.configs.add_cron_job(); cx.notify();
                            });
                        })
                        .child("+ NEW CRON JOB"),
                )
                // Revert button
                .children(if is_modified {
                    Some(
                        div()
                            .id("btn-cron-revert")
                            .px(px(8.0))
                            .py(px(4.0))
                            .bg(hex_rgba(0xef4444, 0.15))
                            .border_1()
                            .border_color(hex_rgba(0xef4444, 0.4))
                            .rounded_sm()
                            .cursor_pointer()
                            .hover(|s| s.bg(hex_rgba(0xef4444, 0.25)))
                            .font_family(FONT_MONO)
                            .text_size(px(10.0))
                            .font_weight(FontWeight::BOLD)
                            .text_color(CRIT)
                            .on_click(move |_ev, _window, cx| {
                                app_revert.update(cx, |this, cx| {
                                    this.revert_managed_config("crontab", cx);
                                });
                            })
                            .child("REVERT")
                    )
                } else {
                    None
                })
                // Stage & Apply button
                .children(if is_modified {
                    Some(
                        div()
                            .id("btn-cron-apply")
                            .px(px(10.0))
                            .py(px(4.0))
                            .bg(OK)
                            .rounded_sm()
                            .cursor_pointer()
                            .hover(|s| s.bg(hex_rgb(0x34d399)))
                            .font_family(FONT_MONO)
                            .text_size(px(10.0))
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0x0a0a0c))
                            .on_click(move |_ev, _window, cx| {
                                app_apply.update(cx, |this, cx| {
                                    this.stage_config_version("crontab", "Updated cron scheduled jobs", cx);
                                });
                            })
                            .child("STAGE & APPLY")
                    )
                } else {
                    None
                })
                // History button
                .child(
                    div()
                        .id("btn-cron-history")
                        .font_family(FONT_MONO)
                        .text_size(px(10.0))
                        .text_color(if configs.show_history { OK } else { TEXT_TERTIARY })
                        .border_1()
                        .border_color(if configs.show_history { OK } else { BORDER_DEFAULT })
                        .bg(if configs.show_history { OK_BG } else { hex_rgba(0, 0.0) })
                        .px(px(8.0))
                        .py(px(4.0))
                        .rounded_sm()
                        .cursor_pointer()
                        .hover(|s| s.bg(BG_ROW_HOVER))
                        .on_click(move |_ev, _window, cx| {
                            app_history.update(cx, |this, cx| {
                                this.toggle_config_history(cx);
                            });
                        })
                        .child(format!("HISTORY · {}", rev_count)),
                ),
        )
        // 2. Info Strip
        .child(
            div()
                .h(px(30.0))
                .flex_none()
                .flex()
                .items_center()
                .justify_between()
                .px(px(16.0))
                .bg(BG_SUBHEAD)
                .border_b_1()
                .border_color(BORDER_PANEL)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .child(
                            tabler_icon(TablerIcon::Clock)
                                .size(px(12.0))
                                .text_color(hex_rgb(0x38bdf8)),
                        )
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(10.0))
                                .text_color(TEXT_TERTIARY)
                                .child("cron.service active · Tasks run with system user permissions"),
                        ),
                )
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(9.5))
                        .text_color(TEXT_DIMMER)
                        .child("Format: MIN HOUR DOM MON DOW USER COMMAND"),
                ),
        )
        // 3. Scrollable List of Visual Cron Job Cards
        .child(
            div()
                .id("cron-jobs-list")
                .flex_1()
                .min_h(px(0.0))
                .overflow_y_scroll()
                .p(px(16.0))
                .flex()
                .flex_col()
                .gap(px(12.0))
                .children(jobs.iter().enumerate().map(|(idx, job)| {
                    render_cron_job_card(job, idx, jobs.len(), app.clone())
                })),
        )
}

fn render_cron_job_card(
    job: &CronJobDef,
    idx: usize,
    total_jobs: usize,
    app: Entity<CrowApp>,
) -> impl IntoElement {
    let job_id = job.id.clone();
    let is_expanded = job.is_expanded;
    let is_enabled = job.enabled;

    let app_toggle = app.clone();
    let app_expand = app.clone();
    let app_del = app.clone();
    let app_up = app.clone();
    let app_down = app.clone();
    let jid_toggle = job_id.clone();
    let jid_expand = job_id.clone();
    let jid_del = job_id.clone();
    let jid_up = job_id.clone();
    let jid_down = job_id.clone();

    div()
        .id(ElementId::NamedInteger("cron-card".into(), idx as u64))
        .bg(if is_enabled { BG_PANEL } else { hex_rgba(0x101116, 0.5) })
        .border_1()
        .border_color(if is_expanded { hex_rgb(0x38bdf8) } else if is_enabled { BORDER_DEFAULT } else { BORDER_PANEL })
        .rounded_sm()
        .flex()
        .flex_col()
        .child(
            // Top Row: Status, Human Schedule, Expression Pills, and Actions
            div()
                .p(px(12.0))
                .flex()
                .items_center()
                .gap(px(12.0))
                .child(
                    // Enabled toggle switch
                    div()
                        .id(ElementId::NamedInteger("btn-toggle-cron".into(), idx as u64))
                        .cursor_pointer()
                        .px(px(6.0))
                        .py(px(3.0))
                        .rounded_sm()
                        .bg(if is_enabled { OK_BG } else { hex_rgba(0xffffff, 0.05) })
                        .border_1()
                        .border_color(if is_enabled { OK } else { BORDER_DEFAULT })
                        .on_click(move |_ev, _window, cx| {
                            let id = jid_toggle.clone();
                            app_toggle.update(cx, |this, cx| {
                                this.configs.toggle_cron_job_enabled(&id); cx.notify();
                            });
                        })
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(9.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(if is_enabled { OK } else { TEXT_MUTED })
                                .child(if is_enabled { "ACTIVE" } else { "DISABLED" }),
                        ),
                )
                // Human Readable Schedule in Bold
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .flex_col()
                        .gap(px(1.0))
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(11.5))
                                .font_weight(FontWeight::BOLD)
                                .text_color(if is_enabled { TEXT_MAX } else { TEXT_MUTED })
                                .child(job.human_schedule()),
                        )
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(10.0))
                                .text_color(TEXT_FAINT)
                                .child(job.comment.clone().unwrap_or_else(|| "No description".to_string())),
                        ),
                )
                // 5 Expression Badges
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(3.0))
                        .child(render_cron_pill("m", &job.minute, hex_rgb(0x67e8f9)))
                        .child(render_cron_pill("h", &job.hour, hex_rgb(0xc084fc)))
                        .child(render_cron_pill("dom", &job.day_of_month, hex_rgb(0x38bdf8)))
                        .child(render_cron_pill("mon", &job.month, hex_rgb(0xfba060)))
                        .child(render_cron_pill("dow", &job.day_of_week, hex_rgb(0xf472b6))),
                )
                // User pill
                .child(
                    div()
                        .px(px(6.0))
                        .py(px(2.0))
                        .bg(BG_CONTROL)
                        .border_1()
                        .border_color(BORDER_DEFAULT)
                        .rounded_sm()
                        .flex()
                        .items_center()
                        .gap(px(4.0))
                        .child(tabler_icon(TablerIcon::Users).size(px(10.0)).text_color(TEXT_MUTED))
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(9.5))
                                .text_color(TEXT_SECONDARY)
                                .child(job.user.clone()),
                        ),
                )
                // Action Buttons: Up, Down, Delete, Expand
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(4.0))
                        .children(if idx > 0 {
                            Some(
                                div()
                                    .id(ElementId::NamedInteger("btn-cron-up".into(), idx as u64))
                                    .p(px(4.0))
                                    .rounded_sm()
                                    .hover(|s| s.bg(BG_ROW_HOVER))
                                    .cursor_pointer()
                                    .on_click(move |_ev, _window, cx| {
                                        let id = jid_up.clone();
                                        app_up.update(cx, |this, cx| {
                                            this.configs.move_cron_job_up(&id); cx.notify();
                                        });
                                    })
                                    .child(
                                        div()
                                            .font_family(FONT_MONO)
                                            .text_size(px(10.0))
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(TEXT_MUTED)
                                            .child("▲"),
                                    )
                                    .into_any_element()
                            )
                        } else {
                            None
                        })
                        .children(if idx + 1 < total_jobs {
                            Some(
                                div()
                                    .id(ElementId::NamedInteger("btn-cron-down".into(), idx as u64))
                                    .p(px(4.0))
                                    .rounded_sm()
                                    .hover(|s| s.bg(BG_ROW_HOVER))
                                    .cursor_pointer()
                                    .on_click(move |_ev, _window, cx| {
                                        let id = jid_down.clone();
                                        app_down.update(cx, |this, cx| {
                                            this.configs.move_cron_job_down(&id); cx.notify();
                                        });
                                    })
                                    .child(
                                        div()
                                            .font_family(FONT_MONO)
                                            .text_size(px(10.0))
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(TEXT_MUTED)
                                            .child("▼"),
                                    )
                                    .into_any_element()
                            )
                        } else {
                            None
                        })
                        .child(
                            div()
                                .id(ElementId::NamedInteger("btn-cron-del".into(), idx as u64))
                                .p(px(4.0))
                                .rounded_sm()
                                .hover(|s| s.bg(hex_rgba(0xef4444, 0.2)))
                                .cursor_pointer()
                                .on_click(move |_ev, _window, cx| {
                                    let id = jid_del.clone();
                                    app_del.update(cx, |this, cx| {
                                        this.configs.delete_cron_job(&id); cx.notify();
                                    });
                                })
                                .child(tabler_icon(TablerIcon::Trash).size(px(12.0)).text_color(TEXT_MUTED)),
                        )
                        .child(
                            div()
                                .id(ElementId::NamedInteger("btn-cron-expand".into(), idx as u64))
                                .px(px(6.0))
                                .py(px(3.0))
                                .bg(if is_expanded { hex_rgba(0x38bdf8, 0.2) } else { BG_CONTROL })
                                .border_1()
                                .border_color(if is_expanded { hex_rgb(0x38bdf8) } else { BORDER_DEFAULT })
                                .rounded_sm()
                                .cursor_pointer()
                                .hover(|s| s.bg(BG_ROW_HOVER))
                                .on_click(move |_ev, _window, cx| {
                                    let id = jid_expand.clone();
                                    app_expand.update(cx, |this, cx| {
                                        this.configs.toggle_cron_job_expanded(&id); cx.notify();
                                    });
                                })
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(9.5))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(if is_expanded { hex_rgb(0x38bdf8) } else { TEXT_SECONDARY })
                                        .child(if is_expanded { "COLLAPSE ▲" } else { "BUILDER ▼" }),
                                ),
                        ),
                ),
        )
        // Command Preview Bar
        .child(
            div()
                .mx(px(12.0))
                .mb(px(12.0))
                .p(px(8.0))
                .px(px(10.0))
                .bg(hex_rgb(0x060709))
                .border_1()
                .border_color(BORDER_PANEL)
                .rounded_sm()
                .flex()
                .items_center()
                .gap(px(8.0))
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(10.0))
                        .text_color(TEXT_FAINT)
                        .child("$"),
                )
                .child(
                    div()
                        .flex_1()
                        .font_family(FONT_MONO)
                        .text_size(px(10.5))
                        .text_color(if is_enabled { hex_rgb(0x86efac) } else { TEXT_DIMMER })
                        .child(job.command.clone()),
                ),
        )
        // Expanded Visual Schedule Builder
        .children(if is_expanded {
            Some(render_cron_builder(job, idx, app.clone()))
        } else {
            None
        })
}

fn render_cron_pill(label: &'static str, val: &str, color: Rgba) -> impl IntoElement {
    div()
        .px(px(5.0))
        .py(px(1.5))
        .bg(hex_rgba(0xffffff, 0.04))
        .border_1()
        .border_color(BORDER_DEFAULT)
        .rounded_sm()
        .flex()
        .items_center()
        .gap(px(2.0))
        .child(
            div()
                .font_family(FONT_MONO)
                .text_size(px(8.5))
                .text_color(TEXT_FAINT)
                .child(label),
        )
        .child(
            div()
                .font_family(FONT_MONO)
                .text_size(px(9.5))
                .font_weight(FontWeight::BOLD)
                .text_color(color)
                .child(val.to_string()),
        )
}

fn render_cron_builder(
    job: &CronJobDef,
    idx: usize,
    app: Entity<CrowApp>,
) -> impl IntoElement {
    let jid = job.id.clone();

    // Quick presets
    let presets = [
        ("Every 15m", "*/15", "*", "*", "*", "*"),
        ("Hourly", "0", "*", "*", "*", "*"),
        ("Daily (3 AM)", "0", "3", "*", "*", "*"),
        ("Daily (Midnight)", "0", "0", "*", "*", "*"),
        ("Weekly (Sun)", "0", "0", "*", "*", "7"),
        ("Monthly (1st)", "0", "0", "1", "*", "*"),
        ("Workdays (9 AM)", "0", "9", "*", "*", "1-5"),
    ];

    div()
        .p(px(14.0))
        .bg(hex_rgb(0x0a0b10))
        .border_t_1()
        .border_color(BORDER_PANEL)
        .flex()
        .flex_col()
        .gap(px(12.0))
        // Presets strip
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(6.0))
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(9.5))
                        .font_weight(FontWeight::BOLD)
                        .text_color(TEXT_DIMMER)
                        .child("QUICK SCHEDULE PRESETS"),
                )
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .gap(px(6.0))
                        .children(presets.into_iter().enumerate().map(|(p_idx, (name, m, h, dom, mon, dow))| {
                            let app_p = app.clone();
                            let j_id = jid.clone();
                            let m_str = m.to_string();
                            let h_str = h.to_string();
                            let dom_str = dom.to_string();
                            let mon_str = mon.to_string();
                            let dow_str = dow.to_string();

                            div()
                                .id(ElementId::NamedInteger("btn-preset".into(), (idx * 100 + p_idx) as u64))
                                .px(px(8.0))
                                .py(px(3.0))
                                .bg(BG_CONTROL)
                                .border_1()
                                .border_color(BORDER_DEFAULT)
                                .rounded_sm()
                                .cursor_pointer()
                                .hover(|s| s.bg(hex_rgba(0x38bdf8, 0.2)).border_color(hex_rgb(0x38bdf8)))
                                .font_family(FONT_MONO)
                                .text_size(px(9.5))
                                .text_color(TEXT_SECONDARY)
                                .on_click(move |_ev, _window, cx| {
                                    let id = j_id.clone();
                                    let (m, h, dom, mon, dow) = (m_str.clone(), h_str.clone(), dom_str.clone(), mon_str.clone(), dow_str.clone());
                                    app_p.update(cx, |this, cx| {
                                        this.configs.apply_cron_preset(&id, &m, &h, &dom, &mon, &dow); cx.notify();
                                    });
                                })
                                .child(name)
                        })),
                ),
        )
        // 5 Interactive Cron Fields
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(6.0))
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(9.5))
                        .font_weight(FontWeight::BOLD)
                        .text_color(TEXT_DIMMER)
                        .child("5-FIELD SCHEDULE EXPRESSION"),
                )
                .child(
                    div()
                        .flex()
                        .gap(px(10.0))
                        .child(render_field_stepper("Minute (0-59)", &job.minute, "m", &jid, app.clone()))
                        .child(render_field_stepper("Hour (0-23)", &job.hour, "h", &jid, app.clone()))
                        .child(render_field_stepper("Day of Month (1-31)", &job.day_of_month, "dom", &jid, app.clone()))
                        .child(render_field_stepper("Month (1-12)", &job.month, "mon", &jid, app.clone()))
                        .child(render_field_stepper("Day of Week (0-7)", &job.day_of_week, "dow", &jid, app.clone())),
                ),
        )
        // User & Command Inputs
        .child(
            div()
                .flex()
                .gap(px(12.0))
                .child(
                    // User
                    div()
                        .w(px(160.0))
                        .flex()
                        .flex_col()
                        .gap(px(4.0))
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(9.5))
                                .font_weight(FontWeight::BOLD)
                                .text_color(TEXT_DIMMER)
                                .child("EXECUTION USER"),
                        )
                        .child(
                            div()
                                .px(px(8.0))
                                .py(px(4.0))
                                .bg(BG_APP)
                                .border_1()
                                .border_color(BORDER_DEFAULT)
                                .rounded_sm()
                                .font_family(FONT_MONO)
                                .text_size(px(10.5))
                                .text_color(OK)
                                .child(job.user.clone()),
                        ),
                )
                .child(
                    // Command
                    div()
                        .flex_1()
                        .flex()
                        .flex_col()
                        .gap(px(4.0))
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(9.5))
                                .font_weight(FontWeight::BOLD)
                                .text_color(TEXT_DIMMER)
                                .child("SHELL COMMAND"),
                        )
                        .child(
                            div()
                                .px(px(10.0))
                                .py(px(4.0))
                                .bg(BG_APP)
                                .border_1()
                                .border_color(BORDER_DEFAULT)
                                .rounded_sm()
                                .font_family(FONT_MONO)
                                .text_size(px(10.5))
                                .text_color(TEXT_PRIMARY)
                                .child(job.command.clone()),
                        ),
                ),
        )
}

fn render_field_stepper(
    title: &str,
    current_val: &str,
    field_key: &str,
    job_id: &str,
    app: Entity<CrowApp>,
) -> impl IntoElement {
    let app_star = app.clone();
    let app_zero = app.clone();
    let jid_star = job_id.to_string();
    let jid_zero = job_id.to_string();
    let fk_star = field_key.to_string();
    let fk_zero = field_key.to_string();

    div()
        .flex_1()
        .bg(BG_PANEL)
        .border_1()
        .border_color(BORDER_DEFAULT)
        .rounded_sm()
        .p(px(8.0))
        .flex()
        .flex_col()
        .gap(px(6.0))
        .child(
            div()
                .font_family(FONT_MONO)
                .text_size(px(9.0))
                .text_color(TEXT_MUTED)
                .child(title.to_string()),
        )
        .child(
            div()
                .font_family(FONT_MONO)
                .text_size(px(14.0))
                .font_weight(FontWeight::BOLD)
                .text_color(TEXT_MAX)
                .child(current_val.to_string()),
        )
        .child(
            div()
                .flex()
                .gap(px(4.0))
                .child(
                    div()
                        .id(ElementId::NamedInteger(format!("cron-btn-star-{}-{}", job_id, field_key).into(), 0))
                        .px(px(5.0))
                        .py(px(2.0))
                        .bg(BG_CONTROL)
                        .border_1()
                        .border_color(BORDER_DEFAULT)
                        .rounded_sm()
                        .cursor_pointer()
                        .hover(|s| s.bg(BG_ROW_HOVER))
                        .font_family(FONT_MONO)
                        .text_size(px(9.0))
                        .text_color(TEXT_SECONDARY)
                        .on_click(move |_ev, _window, cx| {
                            let id = jid_star.clone();
                            let f = fk_star.clone();
                            app_star.update(cx, |this, cx| {
                                this.configs.update_cron_field(&id, &f, "*"); cx.notify();
                            });
                        })
                        .child("* (any)"),
                )
                .child(
                    div()
                        .id(ElementId::NamedInteger(format!("cron-btn-zero-{}-{}", job_id, field_key).into(), 0))
                        .px(px(5.0))
                        .py(px(2.0))
                        .bg(BG_CONTROL)
                        .border_1()
                        .border_color(BORDER_DEFAULT)
                        .rounded_sm()
                        .cursor_pointer()
                        .hover(|s| s.bg(BG_ROW_HOVER))
                        .font_family(FONT_MONO)
                        .text_size(px(9.0))
                        .text_color(TEXT_SECONDARY)
                        .on_click(move |_ev, _window, cx| {
                            let id = jid_zero.clone();
                            let f = fk_zero.clone();
                            app_zero.update(cx, |this, cx| {
                                this.configs.update_cron_field(&id, &f, "0"); cx.notify();
                            });
                        })
                        .child("0"),
                ),
        )
}

#[cfg(test)]
mod tests {
    use super::human_readable_schedule;
    use core::prelude::v1::test;

    #[test]
    fn test_human_readable_schedule_presets() {
        assert_eq!(
            human_readable_schedule("17", "*", "*", "*", "*"),
            "Every hour at minute 17"
        );
        assert_eq!(
            human_readable_schedule("0", "3", "*", "*", "*"),
            "Every day at 03:00"
        );
        assert_eq!(
            human_readable_schedule("0", "0", "*", "*", "7"),
            "Weekly on Sunday at 00:00"
        );
        assert_eq!(
            human_readable_schedule("*/15", "*", "*", "*", "*"),
            "Every 15 minutes"
        );
        assert_eq!(
            human_readable_schedule("0", "0", "1", "*", "*"),
            "Monthly on day 1 at 00:00"
        );
    }

}
