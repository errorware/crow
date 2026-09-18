use gpui_kit::*;
use crate::theme::*;
use crate::app::CrowApp;
use crate::components::icons::{TablerIcon, tabler_icon};
use crate::components::terminal_text_input_styled;
use crate::keys::{
    KeyAlgorithm, KeyGenFieldFocus,
};

pub fn render_keys_center_column(app: Entity<CrowApp>, app_data: &CrowApp) -> impl IntoElement {
    let enrolled_keys = &app_data.enrolled_keys;
    let key_groups = &app_data.key_groups;
    let discovered_keys = &app_data.discovered_keys;
    let selected_group_filter = app_data.selected_key_group_filter.as_deref();
    let scan_status_message = app_data.scan_status_message.as_deref().unwrap_or("Scan idle");

    let unenrolled_discovered: Vec<_> = discovered_keys.iter().filter(|d| !d.is_enrolled).collect();

    let filtered_enrolled: Vec<_> = enrolled_keys.iter().filter(|k| {
        if let Some(grp) = selected_group_filter {
            k.group_id == grp
        } else {
            true
        }
    }).collect();

    let app_scan = app.clone();
    let app_add_path = app.clone();
    let app_gen = app.clone();
    let app_new_grp = app.clone();
    let app_filter_all = app.clone();

    div()
        .flex_1()
        .min_w(px(0.0))
        .flex()
        .flex_col()
        .bg(BG_APP)
        // 1. Keys Subheader & Action Bar
        .child(
            div()
                .h(px(40.0))
                .flex_none()
                .flex()
                .items_center()
                .px(px(14.0))
                .bg(BG_PANEL)
                .border_b_1()
                .border_color(BORDER_PANEL)
                .gap(px(8.0))
                // Group Filter Tabs
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(4.0))
                        // "ALL" tab
                        .child(
                            div()
                                .id("filter-grp-all")
                                .px(px(8.0))
                                .py(px(3.5))
                                .border_1()
                                .border_color(if selected_group_filter.is_none() { TEXT_PRIMARY } else { BORDER_DEFAULT })
                                .bg(if selected_group_filter.is_none() { BG_OVERLAY_PANEL } else { hex_rgba(0, 0.0) })
                                .cursor_pointer()
                                .hover(|s| s.bg(BG_ROW_HOVER))
                                .on_click(move |_ev, _window, cx| {
                                    app_filter_all.update(cx, |this, cx| {
                                        this.set_key_group_filter(None, cx);
                                    });
                                })
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.5))
                                        .font_weight(if selected_group_filter.is_none() { FontWeight::BOLD } else { FontWeight::NORMAL })
                                        .text_color(if selected_group_filter.is_none() { TEXT_MAX } else { TEXT_DIM })
                                        .child(format!("ALL ({})", enrolled_keys.len())),
                                ),
                        )
                        // Group pills
                        .children(key_groups.iter().map(|grp| {
                            let app_grp = app.clone();
                            let grp_id = grp.id.clone();
                            let is_active = selected_group_filter == Some(&grp_id);
                            let count = enrolled_keys.iter().filter(|k| k.group_id == grp_id).count();

                            div()
                                .id(ElementId::Name(format!("filter-grp-{}", grp.id).into()))
                                .flex()
                                .items_center()
                                .gap(px(5.0))
                                .px(px(8.0))
                                .py(px(3.5))
                                .border_1()
                                .border_color(if is_active { TEXT_PRIMARY } else { BORDER_DEFAULT })
                                .bg(if is_active { BG_OVERLAY_PANEL } else { hex_rgba(0, 0.0) })
                                .cursor_pointer()
                                .hover(|s| s.bg(BG_ROW_HOVER))
                                .on_click(move |_ev, _window, cx| {
                                    let gid = grp_id.clone();
                                    app_grp.update(cx, |this, cx| {
                                        this.set_key_group_filter(Some(gid), cx);
                                    });
                                })
                                .child(
                                    div()
                                        .size(px(6.0))
                                        .rounded_full()
                                        .bg(match grp.id.as_str() {
                                            "fleet" => OK,
                                            "bastions" => hex_rgb(0x60a5fa),
                                            "production" => WARN,
                                            "legacy" => CRIT,
                                            _ => TEXT_FAINT,
                                        }),
                                )
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.5))
                                        .font_weight(if is_active { FontWeight::BOLD } else { FontWeight::NORMAL })
                                        .text_color(if is_active { TEXT_MAX } else { TEXT_DIM })
                                        .child(format!("{} ({})", grp.name.to_uppercase(), count)),
                                )
                        }))
                        .child(
                            div()
                                .id("btn-add-group")
                                .px(px(6.0))
                                .py(px(3.5))
                                .border_1()
                                .border_color(BORDER_PANEL)
                                .text_color(TEXT_FAINTER)
                                .cursor_pointer()
                                .hover(|s| s.bg(BG_ROW_HOVER).text_color(TEXT_PRIMARY))
                                .on_click(move |_ev, _window, cx| {
                                    app_new_grp.update(cx, |this, cx| {
                                        this.open_new_group_modal(cx);
                                    });
                                })
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(9.5))
                                        .child("+ GROUP"),
                                ),
                        ),
                )
                .child(div().flex_1())
                // Right Action Buttons
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        // Scan Paths Button
                        .child(
                            div()
                                .id("btn-scan-paths")
                                .h(px(26.0))
                                .px(px(8.0))
                                .bg(BG_CONTROL)
                                .border_1()
                                .border_color(BORDER_DEFAULT)
                                .flex()
                                .items_center()
                                .gap(px(5.0))
                                .cursor_pointer()
                                .hover(|s| s.bg(BG_ROW_HOVER))
                                .on_click(move |_ev, _window, cx| {
                                    app_scan.update(cx, |this, cx| {
                                        this.refresh_keys(cx);
                                    });
                                })
                                .child(tabler_icon(TablerIcon::Refresh).size(px(11.0)).text_color(TEXT_TERTIARY))
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.0))
                                        .text_color(TEXT_SECONDARY)
                                        .child("SCAN ⟳"),
                                ),
                        )
                        // Add Scan Path Button
                        .child(
                            div()
                                .id("btn-add-scan-path")
                                .h(px(26.0))
                                .px(px(8.0))
                                .bg(BG_CONTROL)
                                .border_1()
                                .border_color(BORDER_DEFAULT)
                                .flex()
                                .items_center()
                                .gap(px(5.0))
                                .cursor_pointer()
                                .hover(|s| s.bg(BG_ROW_HOVER))
                                .on_click(move |_ev, _window, cx| {
                                    app_add_path.update(cx, |this, cx| {
                                        this.open_add_scan_path_modal(cx);
                                    });
                                })
                                .child(tabler_icon(TablerIcon::Folder).size(px(11.0)).text_color(TEXT_TERTIARY))
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.0))
                                        .text_color(TEXT_SECONDARY)
                                        .child("+ PATH"),
                                ),
                        )
                        // Generate Key Button
                        .child(
                            div()
                                .id("btn-generate-key")
                                .h(px(26.0))
                                .px(px(10.0))
                                .bg(OK_BG)
                                .border_1()
                                .border_color(OK)
                                .flex()
                                .items_center()
                                .gap(px(5.0))
                                .cursor_pointer()
                                .hover(|s| s.bg(rgb(0x225530)))
                                .on_click(move |_ev, _window, cx| {
                                    app_gen.update(cx, |this, cx| {
                                        this.open_key_gen_modal(cx);
                                    });
                                })
                                .child(tabler_icon(TablerIcon::Key).size(px(11.0)).text_color(OK))
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.5))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(OK_INK)
                                        .child("+ GENERATE KEY"),
                                ),
                        ),
                ),
        )
        // 2. Scrollable Body: Scan Status, Discovered Keys, Enrolled Keys Cards
        .child(
            div()
                .id("keys-scrollable-body")
                .flex_1()
                .overflow_y_scroll()
                .p(px(14.0))
                .flex()
                .flex_col()
                .gap(px(14.0))
                // A. Scan Status Strip
                .child(
                    div()
                        .px(px(12.0))
                        .py(px(7.0))
                        .bg(BG_PANEL)
                        .border_1()
                        .border_color(BORDER_PANEL)
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
                                        .size(px(6.0))
                                        .rounded_full()
                                        .bg(OK),
                                )
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.5))
                                        .text_color(TEXT_SECONDARY)
                                        .child(scan_status_message.to_string()),
                                ),
                        )
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(9.5))
                                .text_color(TEXT_FAINTER)
                                .child("auto-monitored paths"),
                        ),
                )
                // B. Discovered Keys On Disk (shown when there are unenrolled keys)
                .children(if !unenrolled_discovered.is_empty() {
                    Some(
                        div()
                            .p(px(12.0))
                            .bg(hex_rgb(0x0e0c08))
                            .border_1()
                            .border_color(WARN)
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
                                            .flex()
                                            .items_center()
                                            .gap(px(6.0))
                                            .child(div().font_family(FONT_MONO).text_size(px(12.0)).text_color(WARN).child("▲"))
                                            .child(
                                                div()
                                                    .font_family(FONT_MONO)
                                                    .text_size(px(11.0))
                                                    .font_weight(FontWeight::BOLD)
                                                    .text_color(WARN)
                                                    .child(format!("{} UNENROLLED SSH KEY{} DISCOVERED ON DISK", unenrolled_discovered.len(), if unenrolled_discovered.len() == 1 { "" } else { "S" })),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .font_family(FONT_MONO)
                                            .text_size(px(9.5))
                                            .text_color(WARN_INK)
                                            .child("found in scan paths · click Import to enroll"),
                                    ),
                            )
                            .children(unenrolled_discovered.into_iter().enumerate().map(|(idx, disc)| {
                                let app_import = app.clone();
                                let fp = disc.fingerprint.clone();
                                let target_grp = selected_group_filter.unwrap_or("fleet").to_string();

                                div()
                                    .id(ElementId::NamedInteger("disc-key-row".into(), idx as u64))
                                    .p(px(8.0))
                                    .bg(BG_APP)
                                    .border_1()
                                    .border_color(BORDER_DEFAULT)
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .gap(px(12.0))
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap(px(10.0))
                                            .child(
                                                div()
                                                    .px(px(5.0))
                                                    .py(px(2.0))
                                                    .bg(BG_CONTROL)
                                                    .border_1()
                                                    .border_color(BORDER_STRONG)
                                                    .font_family(FONT_MONO)
                                                    .text_size(px(9.0))
                                                    .font_weight(FontWeight::BOLD)
                                                    .text_color(TEXT_SECONDARY)
                                                    .child(disc.algorithm.clone()),
                                            )
                                            .child(
                                                div()
                                                    .flex()
                                                    .flex_col()
                                                    .gap(px(2.0))
                                                    .child(
                                                        div()
                                                            .font_family(FONT_MONO)
                                                            .text_size(px(11.5))
                                                            .font_weight(FontWeight::BOLD)
                                                            .text_color(TEXT_MAX)
                                                            .child(disc.suggested_name.clone()),
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
                                                                    .text_color(TEXT_DIMMER)
                                                                    .child(disc.file_path.clone()),
                                                            )
                                                            .child(
                                                                div()
                                                                    .font_family(FONT_MONO)
                                                                    .text_size(px(9.5))
                                                                    .text_color(TEXT_FAINT)
                                                                    .child(format!("fp: {}", disc.fingerprint)),
                                                            ),
                                                    ),
                                            ),
                                    )
                                    .child({
                                        let import_grp = target_grp.clone();
                                        div()
                                            .id(ElementId::NamedInteger("btn-import-disc".into(), idx as u64))
                                            .px(px(10.0))
                                            .py(px(4.0))
                                            .bg(WARN_BG)
                                            .border_1()
                                            .border_color(WARN)
                                            .cursor_pointer()
                                            .hover(|s| s.bg(hex_rgb(0x3e3016)))
                                            .on_click(move |_ev, _window, cx| {
                                                let fprint = fp.clone();
                                                let grp = import_grp.clone();
                                                app_import.update(cx, |this, cx| {
                                                    this.import_discovered_key(&fprint, Some(&grp), cx);
                                                });
                                            })
                                            .child(
                                                div()
                                                    .font_family(FONT_MONO)
                                                    .text_size(px(10.0))
                                                    .font_weight(FontWeight::BOLD)
                                                    .text_color(WARN_INK)
                                                    .child(format!("+ IMPORT TO {}", target_grp.to_uppercase())),
                                            )
                                    })
                            })),
                    )
                } else {
                    None
                })
                // C. Enrolled Keys Section Header
                .child(
                    div()
                        .flex()
                        .items_baseline()
                        .justify_between()
                        .child(
                            div()
                                .flex()
                                .items_baseline()
                                .gap(px(8.0))
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(11.5))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(TEXT_PRIMARY)
                                        .child(format!("ENROLLED KEYS IN MEMORY ({})", filtered_enrolled.len())),
                                )
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.0))
                                        .text_color(TEXT_DIMMER)
                                        .child(if let Some(g) = selected_group_filter {
                                            format!("filtered by group: {}", g)
                                        } else {
                                            "all groups".to_string()
                                        }),
                                ),
                        )
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(9.5))
                                .text_color(TEXT_FAINT)
                                .child("persisted in ~/.config/crow/crow.db"),
                        ),
                )
                // D. Enrolled Key Cards
                .children(if filtered_enrolled.is_empty() {
                    Some(
                        div()
                            .p(px(32.0))
                            .bg(BG_PANEL)
                            .border_1()
                            .border_color(BORDER_PANEL)
                            .flex()
                            .flex_col()
                            .items_center()
                            .justify_center()
                            .gap(px(8.0))
                            .child(tabler_icon(TablerIcon::Key).size(px(24.0)).text_color(TEXT_FAINT))
                            .child(
                                div()
                                    .font_family(FONT_MONO)
                                    .text_size(px(12.0))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(TEXT_TERTIARY)
                                    .child("NO ENROLLED KEYS MATCHING THIS FILTER"),
                            )
                            .child(
                                div()
                                    .font_family(FONT_MONO)
                                    .text_size(px(10.5))
                                    .text_color(TEXT_FAINTER)
                                    .child("Scan ~/.ssh to import existing keys, or click '+ GENERATE KEY' to create one."),
                            ),
                    )
                } else {
                    None
                })
                .children(filtered_enrolled.into_iter().enumerate().map(|(idx, key)| {
                    let app_copy_fp = app.clone();
                    let app_copy_pk = app.clone();
                    let app_edit = app.clone();
                    let app_del = app.clone();
                    let app_attach = app.clone();

                    let key_id = key.id.clone();
                    let key_id_for_del = key.id.clone();
                    let key_id_for_attach = key.id.clone();
                    let fp = key.fingerprint.clone();
                    let pub_key_str = key.public_key.clone();

                    let grp_color = match key.group_id.as_str() {
                        "fleet" => OK,
                        "bastions" => hex_rgb(0x60a5fa),
                        "production" => WARN,
                        "legacy" => CRIT,
                        _ => TEXT_FAINT,
                    };

                    div()
                        .id(ElementId::NamedInteger("enrolled-key-card".into(), idx as u64))
                        .relative()
                        .bg(BG_PANEL)
                        .border_1()
                        .border_color(BORDER_DEFAULT)
                        .hover(|s| s.border_color(BORDER_STRONG))
                        .p(px(12.0))
                        .flex()
                        .flex_col()
                        .gap(px(8.0))
                        .child(left_indicator(grp_color))
                        // Header row
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .gap(px(10.0))
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap(px(8.0))
                                        // Algo badge
                                        .child(
                                            div()
                                                .px(px(5.0))
                                                .py(px(1.5))
                                                .bg(BG_CONTROL)
                                                .border_1()
                                                .border_color(BORDER_DEFAULT)
                                                .font_family(FONT_MONO)
                                                .text_size(px(9.0))
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(TEXT_SECONDARY)
                                                .child(key.algorithm.clone()),
                                        )
                                        // Key Name
                                        .child(
                                            div()
                                                .font_family(FONT_MONO)
                                                .text_size(px(13.0))
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(TEXT_MAX)
                                                .child(key.name.clone()),
                                        )
                                        // Group pill
                                        .child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .gap(px(4.0))
                                                .px(px(6.0))
                                                .py(px(1.5))
                                                .bg(BG_OVERLAY_PANEL)
                                                .border_1()
                                                .border_color(BORDER_PANEL)
                                                .child(div().size(px(5.0)).rounded_full().bg(grp_color))
                                                .child(
                                                    div()
                                                        .font_family(FONT_MONO)
                                                        .text_size(px(9.5))
                                                        .text_color(TEXT_SECONDARY)
                                                        .child(key.group_id.to_uppercase()),
                                                ),
                                        )
                                        // Private key badge
                                        .children(if key.private_key_path.is_some() {
                                            Some(
                                                div()
                                                    .font_family(FONT_MONO)
                                                    .text_size(px(9.5))
                                                    .text_color(OK)
                                                    .child("● PRIVATE KEY LINKED"),
                                            )
                                        } else {
                                            Some(
                                                div()
                                                    .font_family(FONT_MONO)
                                                    .text_size(px(9.5))
                                                    .text_color(TEXT_FAINT)
                                                    .child("○ PUBLIC KEY ONLY"),
                                            )
                                        }),
                                )
                                // Attached Servers Chips
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap(px(4.0))
                                        .child(
                                            div()
                                                .font_family(FONT_MONO)
                                                .text_size(px(9.0))
                                                .text_color(TEXT_FAINTER)
                                                .child("REACHABLE SERVERS:"),
                                        )
                                        .children(if key.attached_servers.is_empty() {
                                            Some(
                                                div()
                                                    .id(ElementId::NamedInteger("btn-attach-server".into(), idx as u64))
                                                    .px(px(6.0))
                                                    .py(px(2.0))
                                                    .border_1()
                                                    .border_color(BORDER_DEFAULT)
                                                    .font_family(FONT_MONO)
                                                    .text_size(px(9.0))
                                                    .text_color(TEXT_DIM)
                                                    .cursor_pointer()
                                                    .hover(|s| s.bg(BG_ROW_HOVER).text_color(TEXT_PRIMARY))
                                                    .on_click(move |_ev, _window, cx| {
                                                        let kid = key_id_for_attach.clone();
                                                        app_attach.update(cx, |this, cx| {
                                                            this.open_edit_key_modal(&kid, cx);
                                                        });
                                                    })
                                                    .child("+ ATTACH HOST"),
                                            )
                                        } else {
                                            None
                                        })
                                        .children(key.attached_servers.iter().map(|srv| {
                                            div()
                                                .px(px(6.0))
                                                .py(px(2.0))
                                                .bg(BG_CHIP)
                                                .border_1()
                                                .border_color(BORDER_KEY)
                                                .font_family(FONT_MONO)
                                                .text_size(px(9.5))
                                                .font_weight(FontWeight::MEDIUM)
                                                .text_color(OK_INK)
                                                .child(format!("🖥 {}", srv))
                                        })),
                                ),
                        )
                        // Metadata Row: Fingerprint, Path, Comment
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .gap(px(16.0))
                                .p(px(8.0))
                                .bg(BG_APP)
                                .border_1()
                                .border_color(BORDER_ROW)
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
                                                        .font_family(FONT_MONO)
                                                        .text_size(px(9.5))
                                                        .text_color(TEXT_FAINTER)
                                                        .child("SHA256 FINGERPRINT:"),
                                                )
                                                .child(
                                                    div()
                                                        .font_family(FONT_MONO)
                                                        .text_size(px(10.5))
                                                        .text_color(TEXT_PRIMARY)
                                                        .child(key.fingerprint.clone()),
                                                ),
                                        )
                                        .child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .gap(px(12.0))
                                                .children(if let Some(ref path) = key.private_key_path {
                                                    Some(
                                                        div()
                                                            .font_family(FONT_MONO)
                                                            .text_size(px(9.5))
                                                            .text_color(TEXT_DIM)
                                                            .child(format!("path: {}", path)),
                                                    )
                                                } else {
                                                    None
                                                })
                                                .children(if let Some(ref c) = key.comment {
                                                    Some(
                                                        div()
                                                            .font_family(FONT_MONO)
                                                            .text_size(px(9.5))
                                                            .text_color(TEXT_FAINT)
                                                            .child(format!("comment: {}", c)),
                                                    )
                                                } else {
                                                    None
                                                }),
                                        ),
                                )
                                // Copy Actions
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap(px(6.0))
                                        .child(
                                            div()
                                                .id(ElementId::NamedInteger("btn-copy-fp".into(), idx as u64))
                                                .px(px(8.0))
                                                .py(px(4.0))
                                                .bg(BG_CONTROL)
                                                .border_1()
                                                .border_color(BORDER_DEFAULT)
                                                .cursor_pointer()
                                                .hover(|s| s.bg(BG_ROW_HOVER).border_color(TEXT_SECONDARY))
                                                .on_click(move |_ev, _window, cx| {
                                                    let f = fp.clone();
                                                    app_copy_fp.update(cx, |this, cx| {
                                                        this.copy_text_with_toast(&f, "Copied fingerprint to clipboard", cx);
                                                    });
                                                })
                                                .child(
                                                    div()
                                                        .font_family(FONT_MONO)
                                                        .text_size(px(9.5))
                                                        .text_color(TEXT_SECONDARY)
                                                        .child("COPY FINGERPRINT"),
                                                ),
                                        )
                                        .child(
                                            div()
                                                .id(ElementId::NamedInteger("btn-copy-pub".into(), idx as u64))
                                                .px(px(8.0))
                                                .py(px(4.0))
                                                .bg(BG_CONTROL)
                                                .border_1()
                                                .border_color(BORDER_DEFAULT)
                                                .cursor_pointer()
                                                .hover(|s| s.bg(BG_ROW_HOVER).border_color(TEXT_SECONDARY))
                                                .on_click(move |_ev, _window, cx| {
                                                    let pk = pub_key_str.clone();
                                                    app_copy_pk.update(cx, |this, cx| {
                                                        this.copy_text_with_toast(&pk, "Copied OpenSSH public key to clipboard", cx);
                                                    });
                                                })
                                                .child(
                                                    div()
                                                        .font_family(FONT_MONO)
                                                        .text_size(px(9.5))
                                                        .text_color(TEXT_SECONDARY)
                                                        .child("COPY PUBLIC KEY"),
                                                ),
                                        ),
                                ),
                        )
                        // Footer Actions row
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .pt(px(2.0))
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(9.0))
                                        .text_color(TEXT_FAINTER)
                                        .child(format!("enrolled: {}", &key.created_at.chars().take(10).collect::<String>())),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap(px(8.0))
                                        // Edit button
                                        .child(
                                            div()
                                                .id(ElementId::NamedInteger("btn-edit-key".into(), idx as u64))
                                                .px(px(8.0))
                                                .py(px(3.0))
                                                .border_1()
                                                .border_color(BORDER_DEFAULT)
                                                .cursor_pointer()
                                                .hover(|s| s.bg(BG_ROW_HOVER).border_color(TEXT_PRIMARY))
                                                .on_click(move |_ev, _window, cx| {
                                                    let kid = key_id.clone();
                                                    app_edit.update(cx, |this, cx| {
                                                        this.open_edit_key_modal(&kid, cx);
                                                    });
                                                })
                                                .child(
                                                    div()
                                                        .font_family(FONT_MONO)
                                                        .text_size(px(9.5))
                                                        .text_color(TEXT_SECONDARY)
                                                        .child("EDIT / ATTACH SERVERS ✎"),
                                                ),
                                        )
                                        // Delete from Crow button
                                        .child(
                                            div()
                                                .id(ElementId::NamedInteger("btn-del-key".into(), idx as u64))
                                                .px(px(8.0))
                                                .py(px(3.0))
                                                .border_1()
                                                .border_color(BORDER_DANGER_BTN)
                                                .cursor_pointer()
                                                .hover(|s| s.bg(CRIT_BG).border_color(CRIT))
                                                .on_click(move |_ev, _window, cx| {
                                                    let kid = key_id_for_del.clone();
                                                    app_del.update(cx, |this, cx| {
                                                        this.delete_enrolled_key(&kid, cx);
                                                    });
                                                })
                                                .child(
                                                    div()
                                                        .font_family(FONT_MONO)
                                                        .text_size(px(9.5))
                                                        .text_color(CRIT_INK_DIM)
                                                        .child("REMOVE FROM MEMORY ✕"),
                                                ),
                                        ),
                                ),
                        )
                })),
        )
}

pub fn render_keys_right_rail(app: Entity<CrowApp>, app_data: &CrowApp) -> impl IntoElement {
    let scan_paths = &app_data.scan_paths;
    let key_groups = &app_data.key_groups;
    let enrolled_keys = &app_data.enrolled_keys;

    let app_add_path = app.clone();
    let app_new_grp = app.clone();

    div()
        .size_full()
        .flex()
        .flex_col()
        .bg(BG_RAIL)
        // 1. Scan Paths Header
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
                        .font_family(FONT_MONO)
                        .text_size(px(10.5))
                        .font_weight(FontWeight::BOLD)
                        .text_color(TEXT_PRIMARY)
                        .child("SCAN PATHS"),
                )
                .child(
                    div()
                        .id("rail-btn-add-path")
                        .px(px(6.0))
                        .py(px(2.0))
                        .border_1()
                        .border_color(BORDER_PANEL)
                        .cursor_pointer()
                        .hover(|s| s.bg(BG_ROW_HOVER).border_color(TEXT_SECONDARY))
                        .on_click(move |_ev, _window, cx| {
                            app_add_path.update(cx, |this, cx| {
                                this.open_add_scan_path_modal(cx);
                            });
                        })
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(9.0))
                                .text_color(TEXT_TERTIARY)
                                .child("+ ADD PATH"),
                        ),
                ),
        )
        // Scan Paths List
        .child(
            div()
                .p(px(12.0))
                .flex()
                .flex_col()
                .gap(px(6.0))
                .border_b_1()
                .border_color(BORDER_PANEL)
                .children(scan_paths.iter().map(|p| {
                    let app_del_path = app.clone();
                    let pid = p.id;
                    let is_default = p.path.contains(".ssh");

                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .font_family(FONT_MONO)
                        .text_size(px(10.5))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(7.0))
                                .child(tabler_icon(TablerIcon::Folder).size(px(12.0)).text_color(OK))
                                .child(
                                    div()
                                        .text_color(TEXT_SECONDARY)
                                        .child(p.path.clone()),
                                ),
                        )
                        .children(if !is_default {
                            Some(
                                div()
                                    .id(ElementId::NamedInteger("del-scan-path".into(), pid as u64))
                                    .cursor_pointer()
                                    .text_color(TEXT_FAINTER)
                                    .hover(|s| s.text_color(CRIT))
                                    .on_click(move |_ev, _window, cx| {
                                        app_del_path.update(cx, |this, cx| {
                                            this.remove_scan_path(pid, cx);
                                        });
                                    })
                                    .child("✕"),
                            )
                        } else {
                            Some(
                                div()
                                    .id(ElementId::NamedInteger("def-scan-path".into(), pid as u64))
                                    .text_color(TEXT_FAINTER)
                                    .child("default"),
                            )
                        })
                })),
        )
        // 2. Key Groups Taxonomy Header
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
                        .font_family(FONT_MONO)
                        .text_size(px(10.5))
                        .font_weight(FontWeight::BOLD)
                        .text_color(TEXT_PRIMARY)
                        .child("GROUP TAXONOMY"),
                )
                .child(
                    div()
                        .id("rail-btn-add-group")
                        .px(px(6.0))
                        .py(px(2.0))
                        .border_1()
                        .border_color(BORDER_PANEL)
                        .cursor_pointer()
                        .hover(|s| s.bg(BG_ROW_HOVER).border_color(TEXT_SECONDARY))
                        .on_click(move |_ev, _window, cx| {
                            app_new_grp.update(cx, |this, cx| {
                                this.open_new_group_modal(cx);
                            });
                        })
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(9.0))
                                .text_color(TEXT_TERTIARY)
                                .child("+ NEW GROUP"),
                        ),
                ),
        )
        // Key Groups List
        .child(
            div()
                .p(px(12.0))
                .flex()
                .flex_col()
                .gap(px(8.0))
                .children(key_groups.iter().map(|g| {
                    let app_grp_filter = app.clone();
                    let gid = g.id.clone();
                    let count = enrolled_keys.iter().filter(|k| k.group_id == gid).count();
                    let is_active = app_data.selected_key_group_filter.as_deref() == Some(&gid);
                    let is_custom = !matches!(g.id.as_str(), "default" | "fleet" | "bastions" | "production" | "legacy");

                    let grp_c = match g.id.as_str() {
                        "fleet" => OK,
                        "bastions" => hex_rgb(0x60a5fa),
                        "production" => WARN,
                        "legacy" => CRIT,
                        _ => TEXT_FAINT,
                    };

                    div()
                        .id(ElementId::Name(format!("grp-taxo-{}", gid).into()))
                        .flex()
                        .items_center()
                        .justify_between()
                        .cursor_pointer()
                        .p(px(4.0))
                        .bg(if is_active { BG_OVERLAY_PANEL } else { hex_rgba(0, 0.0) })
                        .hover(|s| s.bg(BG_ROW_HOVER))
                        .on_click(move |_ev, _window, cx| {
                            let gid_c = gid.clone();
                            app_grp_filter.update(cx, |this, cx| {
                                if this.selected_key_group_filter.as_deref() == Some(&gid_c) {
                                    this.set_key_group_filter(None, cx);
                                } else {
                                    this.set_key_group_filter(Some(gid_c), cx);
                                }
                            });
                        })
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(8.0))
                                .child(div().size(px(7.0)).rounded_full().bg(grp_c))
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(11.0))
                                        .text_color(if is_active { TEXT_MAX } else { TEXT_PRIMARY })
                                        .child(g.name.clone()),
                                ),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(6.0))
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.0))
                                        .text_color(TEXT_DIMMER)
                                        .child(format!("{} key{}", count, if count == 1 { "" } else { "s" })),
                                )
                                .children(if is_custom {
                                    let app_del_grp = app.clone();
                                    let del_gid = g.id.clone();
                                    Some(
                                        div()
                                            .id(ElementId::Name(format!("del-grp-{}", del_gid).into()))
                                            .cursor_pointer()
                                            .text_color(TEXT_FAINTER)
                                            .hover(|s| s.text_color(CRIT))
                                            .on_click(move |_ev, _window, cx| {
                                                let dgid = del_gid.clone();
                                                app_del_grp.update(cx, |this, cx| {
                                                    this.delete_key_group(&dgid, cx);
                                                });
                                            })
                                            .child("✕"),
                                    )
                                } else {
                                    None
                                }),
                        )
                })),
        )
        // Guidance Box
        .child(
            div()
                .m(px(12.0))
                .p(px(10.0))
                .bg(hex_rgb(0x070709))
                .border_1()
                .border_color(BORDER_PANEL)
                .flex()
                .flex_col()
                .gap(px(4.0))
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(9.5))
                        .font_weight(FontWeight::BOLD)
                        .text_color(TEXT_TERTIARY)
                        .child("KEY ROTATION & AUDIT"),
                )
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(9.5))
                        .text_color(TEXT_FAINT)
                        .line_height(relative(1.45))
                        .child("Keys enrolled in Crow remain on your filesystem and are never transmitted. Rotate keys older than 90 days in accordance with the key policy configured below."),
                ),
        )
}

pub fn render_key_modals(app: Entity<CrowApp>, app_data: &CrowApp) -> Vec<Div> {
    let mut overlays = Vec::new();

    // 1. Key Generation Modal
    if let Some(ref gen) = app_data.key_gen_modal {
        let app_close = app.clone();
        let app_submit = app.clone();
        let app_copy = app.clone();
        let app_cycle_focus = app.clone();
        let app_focus_name = app.clone();
        let app_focus_comment = app.clone();
        let app_focus_dir = app.clone();

        let is_success = gen.generated_public_key.is_some();
        let pubkey_to_copy = gen.generated_public_key.clone().unwrap_or_default();
        let priv_path = gen.generated_priv_path.clone().unwrap_or_default();
        let fp = gen.generated_fingerprint.clone().unwrap_or_default();
        let err = gen.error_message.clone();

        overlays.push(
            div()
                .absolute()
                .inset_0()
                .bg(hex_rgba(0x050507, 0.85))
                .flex()
                .items_center()
                .justify_center()
                .child(
                    div()
                        .w(px(520.0))
                        .bg(BG_PANEL)
                        .border_1()
                        .border_color(if is_success { OK } else { BORDER_STRONG })
                        .p(px(20.0))
                        .flex()
                        .flex_col()
                        .gap(px(14.0))
                        // Modal Header
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(13.0))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(if is_success { OK } else { TEXT_MAX })
                                        .child(if is_success { "✓ KEYPAIR GENERATED AND ENROLLED" } else { "+ GENERATE NEW OPENSSH KEYPAIR" }),
                                )
                                .child(
                                    div()
                                        .id("btn-close-keygen-esc")
                                        .cursor_pointer()
                                        .font_family(FONT_MONO)
                                        .text_size(px(11.0))
                                        .text_color(TEXT_DIM)
                                        .hover(|s| s.text_color(TEXT_MAX))
                                        .on_click(move |_ev, _window, cx| {
                                            app_close.update(cx, |this, cx| {
                                                this.close_key_gen_modal(cx);
                                            });
                                        })
                                        .child("esc"),
                                ),
                        )
                        // Success view
                        .children(if is_success {
                            Some(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(10.0))
                                    .child(
                                        div()
                                            .font_family(FONT_MONO)
                                            .text_size(px(10.5))
                                            .text_color(TEXT_MUTED)
                                            .child(format!("Private key written to disk with permissions 0600:\n{}", priv_path)),
                                    )
                                    .child(
                                        div()
                                            .font_family(FONT_MONO)
                                            .text_size(px(10.0))
                                            .text_color(TEXT_FAINT)
                                            .child(format!("Fingerprint: {}", fp)),
                                    )
                                    .child(
                                        div()
                                            .p(px(8.0))
                                            .bg(hex_rgb(0x070709))
                                            .border_1()
                                            .border_color(BORDER_DEFAULT)
                                            .font_family(FONT_MONO)
                                            .text_size(px(10.0))
                                            .text_color(OK_INK)
                                            .line_height(relative(1.4))
                                            .child(pubkey_to_copy.clone()),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .justify_end()
                                            .gap(px(8.0))
                                            .child(
                                                div()
                                                    .id("btn-copy-generated-pubkey")
                                                    .h(px(30.0))
                                                    .px(px(12.0))
                                                    .bg(BG_KEY)
                                                    .border_1()
                                                    .border_color(BORDER_DEFAULT)
                                                    .cursor_pointer()
                                                    .flex()
                                                    .items_center()
                                                    .hover(|s| s.bg(BG_ROW_HOVER))
                                                    .on_click(move |_ev, _window, cx| {
                                                        let pk = pubkey_to_copy.clone();
                                                        app_copy.update(cx, |this, cx| {
                                                            this.copy_text_with_toast(&pk, "Copied public key to clipboard", cx);
                                                        });
                                                    })
                                                    .child(
                                                        div()
                                                            .font_family(FONT_MONO)
                                                            .text_size(px(10.5))
                                                            .text_color(TEXT_PRIMARY)
                                                            .child("COPY PUBLIC KEY"),
                                                    ),
                                            ),
                                    ),
                            )
                        } else {
                            None
                        })
                        // Input Fields (if not success yet)
                        .children(if !is_success {
                            Some(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(10.0))
                                    // Field 1: Key Name
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .gap(px(3.0))
                                            .child(
                                                div()
                                                    .font_family(FONT_MONO)
                                                    .text_size(px(10.0))
                                                    .font_weight(FontWeight::BOLD)
                                                    .text_color(if gen.active_focus == KeyGenFieldFocus::Name { TEXT_MAX } else { TEXT_DIM })
                                                    .child("KEY NAME:"),
                                            )
                                            .child(
                                                terminal_text_input_styled(
                                                    "input-keygen-name",
                                                    &gen.name_input,
                                                    "e.g. id_ed25519_bastion",
                                                    gen.active_focus == KeyGenFieldFocus::Name,
                                                    false,
                                                    28.0,
                                                    11.0,
                                                    if gen.active_focus == KeyGenFieldFocus::Name { app_data.input_cursor } else { 0 },
                                                    if gen.active_focus == KeyGenFieldFocus::Name { app_data.input_selection } else { None },
                                                    if gen.active_focus == KeyGenFieldFocus::Name { app_data.input_drag_anchor } else { None },
                                                    app_data.cursor_blink,
                                                    {
                                                        let app = app_focus_name;
                                                        move |cursor, anchor, selection, _window, cx| {
                                                            app.update(cx, |this, cx| {
                                                                if let Some(ref mut g) = this.key_gen_modal {
                                                                    g.active_focus = KeyGenFieldFocus::Name;
                                                                }
                                                                this.input_cursor = cursor;
                                                                this.input_drag_anchor = anchor;
                                                                this.input_selection = selection;
                                                                this.cursor_blink = true;
                                                                cx.notify();
                                                            });
                                                        }
                                                    },
                                                ),
                                            ),
                                    )
                                    // Field 2: Algorithm
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .gap(px(3.0))
                                            .child(
                                                div()
                                                    .font_family(FONT_MONO)
                                                    .text_size(px(10.0))
                                                    .font_weight(FontWeight::BOLD)
                                                    .text_color(TEXT_DIM)
                                                    .child("ALGORITHM:"),
                                            )
                                            .child(
                                                div()
                                                    .flex()
                                                    .gap(px(8.0))
                                                    .child(
                                                        div()
                                                            .px(px(10.0))
                                                            .py(px(4.0))
                                                            .bg(if gen.algo == KeyAlgorithm::Ed25519 { OK_BG } else { BG_CONTROL })
                                                            .border_1()
                                                            .border_color(if gen.algo == KeyAlgorithm::Ed25519 { OK } else { BORDER_DEFAULT })
                                                            .child(
                                                                div()
                                                                    .font_family(FONT_MONO)
                                                                    .text_size(px(10.5))
                                                                    .font_weight(FontWeight::BOLD)
                                                                    .text_color(if gen.algo == KeyAlgorithm::Ed25519 { OK_INK } else { TEXT_SECONDARY })
                                                                    .child("Ed25519 (Recommended)"),
                                                            ),
                                                    )
                                                    .child(
                                                        div()
                                                            .px(px(10.0))
                                                            .py(px(4.0))
                                                            .bg(if gen.algo == KeyAlgorithm::Rsa4096 { WARN_BG } else { BG_CONTROL })
                                                            .border_1()
                                                            .border_color(if gen.algo == KeyAlgorithm::Rsa4096 { WARN } else { BORDER_DEFAULT })
                                                            .child(
                                                                div()
                                                                    .font_family(FONT_MONO)
                                                                    .text_size(px(10.5))
                                                                    .font_weight(FontWeight::BOLD)
                                                                    .text_color(if gen.algo == KeyAlgorithm::Rsa4096 { WARN_INK } else { TEXT_DIM })
                                                                    .child("RSA 4096-bit"),
                                                            ),
                                                    ),
                                            ),
                                    )
                                    // Field 3: Comment
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .gap(px(3.0))
                                            .child(
                                                div()
                                                    .font_family(FONT_MONO)
                                                    .text_size(px(10.0))
                                                    .font_weight(FontWeight::BOLD)
                                                    .text_color(if gen.active_focus == KeyGenFieldFocus::Comment { TEXT_MAX } else { TEXT_DIM })
                                                    .child("COMMENT (OPTIONAL):"),
                                            )
                                            .child(
                                                terminal_text_input_styled(
                                                    "input-keygen-comment",
                                                    &gen.comment_input,
                                                    "e.g. nelson@crow",
                                                    gen.active_focus == KeyGenFieldFocus::Comment,
                                                    false,
                                                    28.0,
                                                    11.0,
                                                    if gen.active_focus == KeyGenFieldFocus::Comment { app_data.input_cursor } else { 0 },
                                                    if gen.active_focus == KeyGenFieldFocus::Comment { app_data.input_selection } else { None },
                                                    if gen.active_focus == KeyGenFieldFocus::Comment { app_data.input_drag_anchor } else { None },
                                                    app_data.cursor_blink,
                                                    {
                                                        let app = app_focus_comment;
                                                        move |cursor, anchor, selection, _window, cx| {
                                                            app.update(cx, |this, cx| {
                                                                if let Some(ref mut g) = this.key_gen_modal {
                                                                    g.active_focus = KeyGenFieldFocus::Comment;
                                                                }
                                                                this.input_cursor = cursor;
                                                                this.input_drag_anchor = anchor;
                                                                this.input_selection = selection;
                                                                this.cursor_blink = true;
                                                                cx.notify();
                                                            });
                                                        }
                                                    },
                                                ),
                                            ),
                                    )
                                    // Field 4: Destination directory
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .gap(px(3.0))
                                            .child(
                                                div()
                                                    .font_family(FONT_MONO)
                                                    .text_size(px(10.0))
                                                    .font_weight(FontWeight::BOLD)
                                                    .text_color(if gen.active_focus == KeyGenFieldFocus::Directory { TEXT_MAX } else { TEXT_DIM })
                                                    .child("SAVE DIRECTORY:"),
                                            )
                                            .child(
                                                terminal_text_input_styled(
                                                    "input-keygen-dir",
                                                    &gen.custom_dir_input,
                                                    "~/.ssh",
                                                    gen.active_focus == KeyGenFieldFocus::Directory,
                                                    false,
                                                    28.0,
                                                    11.0,
                                                    if gen.active_focus == KeyGenFieldFocus::Directory { app_data.input_cursor } else { 0 },
                                                    if gen.active_focus == KeyGenFieldFocus::Directory { app_data.input_selection } else { None },
                                                    if gen.active_focus == KeyGenFieldFocus::Directory { app_data.input_drag_anchor } else { None },
                                                    app_data.cursor_blink,
                                                    {
                                                        let app = app_focus_dir;
                                                        move |cursor, anchor, selection, _window, cx| {
                                                            app.update(cx, |this, cx| {
                                                                if let Some(ref mut g) = this.key_gen_modal {
                                                                    g.active_focus = KeyGenFieldFocus::Directory;
                                                                }
                                                                this.input_cursor = cursor;
                                                                this.input_drag_anchor = anchor;
                                                                this.input_selection = selection;
                                                                this.cursor_blink = true;
                                                                cx.notify();
                                                            });
                                                        }
                                                    },
                                                ),
                                            ),
                                    )
                                    // Error message
                                    .children(if let Some(ref e) = err {
                                        Some(
                                            div()
                                                .font_family(FONT_MONO)
                                                .text_size(px(10.5))
                                                .text_color(CRIT)
                                                .child(e.clone()),
                                        )
                                    } else {
                                        None
                                    })
                                    // Modal action buttons
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .justify_between()
                                            .pt(px(6.0))
                                            .child(
                                                div()
                                                    .font_family(FONT_MONO)
                                                    .text_size(px(9.5))
                                                    .text_color(TEXT_FAINTER)
                                                    .child("tab to switch fields · enter to generate"),
                                            )
                                            .child(
                                                div()
                                                    .flex()
                                                    .items_center()
                                                    .gap(px(8.0))
                                                    .child(
                                                        div()
                                                            .id("btn-modal-cancel-gen")
                                                            .px(px(10.0))
                                                            .py(px(5.0))
                                                            .border_1()
                                                            .border_color(BORDER_DEFAULT)
                                                            .cursor_pointer()
                                                            .hover(|s| s.bg(BG_ROW_HOVER))
                                                            .on_click(move |_ev, _window, cx| {
                                                                app_cycle_focus.update(cx, |this, cx| {
                                                                    this.close_key_gen_modal(cx);
                                                                });
                                                            })
                                                            .child(
                                                                div()
                                                                    .font_family(FONT_MONO)
                                                                    .text_size(px(10.0))
                                                                    .text_color(TEXT_TERTIARY)
                                                                    .child("CANCEL"),
                                                            ),
                                                    )
                                                    .child(
                                                        div()
                                                            .id("btn-modal-submit-gen")
                                                            .px(px(12.0))
                                                            .py(px(5.0))
                                                            .bg(OK_BG)
                                                            .border_1()
                                                            .border_color(OK)
                                                            .cursor_pointer()
                                                            .hover(|s| s.bg(rgb(0x225530)))
                                                            .on_click(move |_ev, _window, cx| {
                                                                app_submit.update(cx, |this, cx| {
                                                                    this.submit_key_generation(cx);
                                                                });
                                                            })
                                                            .child(
                                                                div()
                                                                    .font_family(FONT_MONO)
                                                                    .text_size(px(10.5))
                                                                    .font_weight(FontWeight::BOLD)
                                                                    .text_color(OK_INK)
                                                                    .child("GENERATE KEYPAIR ↵"),
                                                            ),
                                                    ),
                                            ),
                                    ),
                            )
                        } else {
                            None
                        }),
                ),
        );
    } else if let Some(ref sp) = app_data.add_scan_path_modal {
        let app_close = app.clone();
        let app_submit = app.clone();
        let path_text = sp.path_input.clone();
        let err = sp.error_message.clone();

        overlays.push(
            div()
                .absolute()
                .inset_0()
                .bg(hex_rgba(0x050507, 0.85))
                .flex()
                .items_center()
                .justify_center()
                .child(
                    div()
                        .w(px(460.0))
                        .bg(BG_PANEL)
                        .border_1()
                        .border_color(BORDER_STRONG)
                        .p(px(20.0))
                        .flex()
                        .flex_col()
                        .gap(px(12.0))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(12.5))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(TEXT_MAX)
                                        .child("ADD SSH KEY SCAN DIRECTORY"),
                                )
                                .child(
                                    div()
                                        .id("btn-close-scanpath-esc")
                                        .cursor_pointer()
                                        .font_family(FONT_MONO)
                                        .text_size(px(11.0))
                                        .text_color(TEXT_DIM)
                                        .on_click(move |_ev, _window, cx| {
                                            app_close.update(cx, |this, cx| {
                                                this.close_add_scan_path_modal(cx);
                                            });
                                        })
                                        .child("esc"),
                                ),
                        )
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(10.5))
                                .text_color(TEXT_DIM)
                                .child("Enter directory path (e.g. ~/work-keys or /etc/ssh):"),
                        )
                        .child(
                            terminal_text_input_styled(
                                "input-scan-path",
                                &path_text,
                                "e.g. ~/work-keys or /etc/ssh",
                                true,
                                false,
                                30.0,
                                11.0,
                                app_data.input_cursor,
                                app_data.input_selection,
                                app_data.input_drag_anchor,
                                app_data.cursor_blink,
                                {
                                    let app = app.clone();
                                    move |cursor, anchor, selection, _window, cx| {
                                        app.update(cx, |this, cx| {
                                            this.input_cursor = cursor;
                                            this.input_drag_anchor = anchor;
                                            this.input_selection = selection;
                                            this.cursor_blink = true;
                                            cx.notify();
                                        });
                                    }
                                },
                            ),
                        )
                        .children(if let Some(ref e) = err {
                            Some(
                                div()
                                    .font_family(FONT_MONO)
                                    .text_size(px(10.0))
                                    .text_color(CRIT)
                                    .child(e.clone()),
                            )
                        } else {
                            None
                        })
                        .child(
                            div()
                                .flex()
                                .justify_end()
                                .gap(px(8.0))
                                .child(
                                    div()
                                        .id("btn-submit-add-path")
                                        .px(px(12.0))
                                        .py(px(5.0))
                                        .bg(BG_KEY)
                                        .border_1()
                                        .border_color(BORDER_DEFAULT)
                                        .cursor_pointer()
                                        .hover(|s| s.bg(BG_ROW_HOVER))
                                        .on_click(move |_ev, _window, cx| {
                                            app_submit.update(cx, |this, cx| {
                                                this.submit_add_scan_path(cx);
                                            });
                                        })
                                        .child(
                                            div()
                                                .font_family(FONT_MONO)
                                                .text_size(px(10.5))
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(OK)
                                                .child("ADD & SCAN ↵"),
                                        ),
                                ),
                        ),
                ),
        );
    } else if let Some(ref grp) = app_data.new_group_modal {
        let app_close = app.clone();
        let app_submit = app.clone();
        let name_text = grp.name_input.clone();
        let err = grp.error_message.clone();

        overlays.push(
            div()
                .absolute()
                .inset_0()
                .bg(hex_rgba(0x050507, 0.85))
                .flex()
                .items_center()
                .justify_center()
                .child(
                    div()
                        .w(px(420.0))
                        .bg(BG_PANEL)
                        .border_1()
                        .border_color(BORDER_STRONG)
                        .p(px(20.0))
                        .flex()
                        .flex_col()
                        .gap(px(12.0))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(12.5))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(TEXT_MAX)
                                        .child("CREATE NEW KEY GROUP"),
                                )
                                .child(
                                    div()
                                        .id("btn-close-newgroup-esc")
                                        .cursor_pointer()
                                        .font_family(FONT_MONO)
                                        .text_size(px(11.0))
                                        .text_color(TEXT_DIM)
                                        .on_click(move |_ev, _window, cx| {
                                            app_close.update(cx, |this, cx| {
                                                this.close_new_group_modal(cx);
                                            });
                                        })
                                        .child("esc"),
                                ),
                        )
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(10.5))
                                .text_color(TEXT_DIM)
                                .child("Enter group name (e.g. Staging Fleet, Edge Bastions):"),
                        )
                        .child(
                            terminal_text_input_styled(
                                "input-new-group-name",
                                &name_text,
                                "e.g. Staging Fleet, Edge Bastions",
                                true,
                                false,
                                30.0,
                                11.0,
                                app_data.input_cursor,
                                app_data.input_selection,
                                app_data.input_drag_anchor,
                                app_data.cursor_blink,
                                {
                                    let app = app.clone();
                                    move |cursor, anchor, selection, _window, cx| {
                                        app.update(cx, |this, cx| {
                                            this.input_cursor = cursor;
                                            this.input_drag_anchor = anchor;
                                            this.input_selection = selection;
                                            this.cursor_blink = true;
                                            cx.notify();
                                        });
                                    }
                                },
                            ),
                        )
                        .children(if let Some(ref e) = err {
                            Some(
                                div()
                                    .font_family(FONT_MONO)
                                    .text_size(px(10.0))
                                    .text_color(CRIT)
                                    .child(e.clone()),
                            )
                        } else {
                            None
                        })
                        .child(
                            div()
                                .flex()
                                .justify_end()
                                .gap(px(8.0))
                                .child(
                                    div()
                                        .id("btn-submit-new-group")
                                        .px(px(12.0))
                                        .py(px(5.0))
                                        .bg(BG_KEY)
                                        .border_1()
                                        .border_color(BORDER_DEFAULT)
                                        .cursor_pointer()
                                        .hover(|s| s.bg(BG_ROW_HOVER))
                                        .on_click(move |_ev, _window, cx| {
                                            app_submit.update(cx, |this, cx| {
                                                this.submit_new_group(cx);
                                            });
                                        })
                                        .child(
                                            div()
                                                .font_family(FONT_MONO)
                                                .text_size(px(10.5))
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(OK)
                                                .child("CREATE GROUP ↵"),
                                        ),
                                ),
                        ),
                ),
        );
    } else if let Some(ref edit) = app_data.edit_key_modal {
        let app_close = app.clone();
        let app_submit = app.clone();
        let name_text = edit.name_input.clone();
        let attached = edit.attached_servers.clone();
        let key_groups = &app_data.key_groups;
        let tabs = &app_data.tabs;
        let err = edit.error_message.clone();

        overlays.push(
            div()
                .absolute()
                .inset_0()
                .bg(hex_rgba(0x050507, 0.85))
                .flex()
                .items_center()
                .justify_center()
                .child(
                    div()
                        .w(px(500.0))
                        .bg(BG_PANEL)
                        .border_1()
                        .border_color(BORDER_STRONG)
                        .p(px(20.0))
                        .flex()
                        .flex_col()
                        .gap(px(12.0))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(12.5))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(TEXT_MAX)
                                        .child("EDIT KEY & ATTACHED SERVERS"),
                                )
                                .child(
                                    div()
                                        .id("btn-close-editkey-esc")
                                        .cursor_pointer()
                                        .font_family(FONT_MONO)
                                        .text_size(px(11.0))
                                        .text_color(TEXT_DIM)
                                        .on_click(move |_ev, _window, cx| {
                                            app_close.update(cx, |this, cx| {
                                                this.close_edit_key_modal(cx);
                                            });
                                        })
                                        .child("esc"),
                                ),
                        )
                        // Name input
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(3.0))
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.0))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(TEXT_DIM)
                                        .child("NAME:"),
                                )
                                .child(
                                    terminal_text_input_styled(
                                        "input-edit-key-name",
                                        &name_text,
                                        "Enter key name…",
                                        true,
                                        false,
                                        28.0,
                                        11.0,
                                        app_data.input_cursor,
                                        app_data.input_selection,
                                        app_data.input_drag_anchor,
                                        app_data.cursor_blink,
                                        {
                                            let app = app.clone();
                                            move |cursor, anchor, selection, _window, cx| {
                                                app.update(cx, |this, cx| {
                                                    this.input_cursor = cursor;
                                                    this.input_drag_anchor = anchor;
                                                    this.input_selection = selection;
                                                    this.cursor_blink = true;
                                                    cx.notify();
                                                });
                                            }
                                        },
                                    ),
                                ),
                        )
                        // Group selector chips
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(3.0))
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.0))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(TEXT_DIM)
                                        .child("GROUP ASSIGNMENT:"),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .gap(px(6.0))
                                        .children(key_groups.iter().map(|g| {
                                            let app_set_grp = app.clone();
                                            let gid = g.id.clone();
                                            let is_sel = edit.group_id == gid;

                                            div()
                                                .id(ElementId::Name(format!("grp-chip-{}", gid).into()))
                                                .px(px(8.0))
                                                .py(px(3.0))
                                                .bg(if is_sel { BG_OVERLAY_PANEL } else { BG_CONTROL })
                                                .border_1()
                                                .border_color(if is_sel { TEXT_PRIMARY } else { BORDER_DEFAULT })
                                                .cursor_pointer()
                                                .on_click(move |_ev, _window, cx| {
                                                    let gid_c = gid.clone();
                                                    app_set_grp.update(cx, |this, cx| {
                                                        if let Some(ref mut st) = this.edit_key_modal {
                                                            st.group_id = gid_c;
                                                            cx.notify();
                                                        }
                                                    });
                                                })
                                                .child(
                                                    div()
                                                        .font_family(FONT_MONO)
                                                        .text_size(px(10.0))
                                                        .font_weight(if is_sel { FontWeight::BOLD } else { FontWeight::NORMAL })
                                                        .text_color(if is_sel { TEXT_MAX } else { TEXT_DIM })
                                                        .child(g.name.clone()),
                                                )
                                        })),
                                ),
                        )
                        // Attached Servers selector
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
                                        .text_color(TEXT_DIM)
                                        .child("ATTACH TO FLEET SERVERS:"),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_wrap()
                                        .gap(px(6.0))
                                        .children(tabs.iter().map(|t| {
                                            let app_tog = app.clone();
                                            let srv_id = t.id.to_string();
                                            let is_attached = attached.iter().any(|s| s == &srv_id);

                                            div()
                                                .id(ElementId::Name(format!("srv-chip-{}", srv_id).into()))
                                                .px(px(8.0))
                                                .py(px(4.0))
                                                .bg(if is_attached { OK_BG } else { BG_CONTROL })
                                                .border_1()
                                                .border_color(if is_attached { OK } else { BORDER_DEFAULT })
                                                .cursor_pointer()
                                                .hover(|s| s.bg(BG_ROW_HOVER))
                                                .on_click(move |_ev, _window, cx| {
                                                    let sid = srv_id.clone();
                                                    app_tog.update(cx, |this, cx| {
                                                        this.toggle_edit_key_server(&sid, cx);
                                                    });
                                                })
                                                .child(
                                                    div()
                                                        .font_family(FONT_MONO)
                                                        .text_size(px(10.0))
                                                        .font_weight(if is_attached { FontWeight::BOLD } else { FontWeight::NORMAL })
                                                        .text_color(if is_attached { OK_INK } else { TEXT_SECONDARY })
                                                        .child(format!("{} {}", if is_attached { "✓" } else { "+" }, t.name)),
                                                )
                                        })),
                                ),
                        )
                        .children(if let Some(ref e) = err {
                            Some(
                                div()
                                    .font_family(FONT_MONO)
                                    .text_size(px(10.0))
                                    .text_color(CRIT)
                                    .child(e.clone()),
                            )
                        } else {
                            None
                        })
                        .child(
                            div()
                                .flex()
                                .justify_end()
                                .gap(px(8.0))
                                .pt(px(6.0))
                                .child(
                                    div()
                                        .id("btn-submit-edit-key")
                                        .px(px(12.0))
                                        .py(px(5.0))
                                        .bg(BG_KEY)
                                        .border_1()
                                        .border_color(BORDER_DEFAULT)
                                        .cursor_pointer()
                                        .hover(|s| s.bg(BG_ROW_HOVER))
                                        .on_click(move |_ev, _window, cx| {
                                            app_submit.update(cx, |this, cx| {
                                                this.submit_edit_key(cx);
                                            });
                                        })
                                        .child(
                                            div()
                                                .font_family(FONT_MONO)
                                                .text_size(px(10.5))
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(OK)
                                                .child("SAVE CHANGES ↵"),
                                        ),
                                ),
                        ),
                ),
        );
    }

    // 5. Floating Toast Feedback
    if let Some(ref toast_msg) = app_data.key_toast {
        let app_dismiss = app.clone();
        overlays.push(
            div()
                .absolute()
                .bottom(px(16.0))
                .right(px(16.0))
                .child(
                    div()
                        .id("floating-toast")
                        .bg(hex_rgb(0x102618))
                        .border_1()
                        .border_color(OK)
                        .p(px(10.0))
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .cursor_pointer()
                        .on_click(move |_ev, _window, cx| {
                            app_dismiss.update(cx, |this, cx| {
                                this.clear_key_toast(cx);
                            });
                        })
                        .child(div().font_family(FONT_MONO).text_size(px(12.0)).text_color(OK).child("✓"))
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(10.5))
                                .font_weight(FontWeight::BOLD)
                                .text_color(OK_INK)
                                .child(toast_msg.clone()),
                        ),
                ),
        );
    }

    overlays
}
