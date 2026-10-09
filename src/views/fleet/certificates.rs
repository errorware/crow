//! Fleet Setup → CERTIFICATES (ERR-146): every server's certificates, how
//! long they have, whether they renew on their own, and RENEW.

use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::app::certificates::{certbot_name, CertificatesState};
use crate::app::CrowApp;
use crate::components::icons::{inherited_icon, TablerIcon};
use crate::metrics::certs::{CERT_CRIT_DAYS, CERT_WARN_DAYS};
use crate::theme::*;
use crate::views::fleet::FleetState;

fn mono(size: f32, color: Rgba) -> Div {
    div().font_family(FONT_MONO).text_size(px(size)).text_color(color)
}

pub fn certificates_page(fleet: &FleetState, st: &CertificatesState, app: Entity<CrowApp>) -> impl IntoElement {
    let now = chrono::Utc::now().timestamp();
    let total: usize = st.certs.values().map(Vec::len).sum();
    let soon = st.certs.values().flatten().filter(|c| (c.expires - now) / 86_400 <= CERT_WARN_DAYS).count();
    let broken = st.inventory.values().filter_map(|i| i.as_ref().ok()).filter(|i| i.auto_renew_problem().is_some()).count();
    let app_refresh = app.clone();
    div()
        .id("certificates-page")
        .flex_1()
        .min_h(px(0.0))
        .overflow_y_scrollbar()
        .child(
            div()
                .max_w(px(1100.0))
                .p(px(18.0))
                .flex()
                .flex_col()
                .gap(px(14.0))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(12.0))
                        .child(mono(10.5, TEXT_SECONDARY).font_weight(FontWeight::BOLD).child("CERTIFICATES"))
                        .child(mono(10.5, if soon > 0 || broken > 0 { WARN } else { OK }).child(format!("{total} served · {soon} expiring within {CERT_WARN_DAYS} days · {broken} server{} not renewing on its own", if broken == 1 { "" } else { "s" })))
                        .child(div().flex_1())
                        .child(
                            div()
                                .id("btn-certs-refresh")
                                .flex()
                                .items_center()
                                .gap(px(6.0))
                                .h(px(26.0))
                                .px(px(10.0))
                                .border_1()
                                .border_color(BORDER_DEFAULT)
                                .font_family(FONT_MONO)
                                .text_size(px(10.0))
                                .text_color(TEXT_SECONDARY)
                                .cursor_pointer()
                                .hover(|s| s.bg(BG_ROW_HOVER).text_color(TEXT_PRIMARY))
                                .on_click(move |_ev, _w, cx| app_refresh.update(cx, |this, cx| this.refresh_certificates(true, cx)))
                                .child(inherited_icon(TablerIcon::Refresh, px(12.0)))
                                .child(if st.reading.is_empty() { "READ AGAIN".to_string() } else { format!("READING {}…", st.reading.len()) }),
                        ),
                )
                .children(st.message.clone().map(|(ok, m)| mono(10.5, if ok { OK } else { WARN }).line_height(px(15.0)).child(m)))
                .child(mono(9.5, TEXT_FAINT).line_height(px(13.0)).child("RENEW runs certbot's dry run first, then the renewal, then checks and reloads nginx or Apache (never a broken config), and reads the certificate again to confirm its new expiry. certbot only renews within 30 days of expiry; Let's Encrypt limits how often a certificate can be issued."))
                .children(fleet.servers.iter().filter(|s| st.inventory.contains_key(&s.id) || st.certs.contains_key(&s.id) || st.reading.contains(&s.id)).map(|s| {
                    let inv = st.inventory.get(&s.id);
                    let certs = st.certs.get(&s.id).cloned().unwrap_or_default();
                    let tool = match inv {
                        Some(Ok(i)) if i.certbot => "certbot".to_string(),
                        Some(Ok(i)) if i.acme_sh.is_some() => "acme.sh".to_string(),
                        Some(Ok(_)) => "no renewal tool".to_string(),
                        Some(Err(e)) => format!("couldn't read: {e}"),
                        None => "reading…".to_string(),
                    };
                    let problem = inv.and_then(|i| i.as_ref().ok()).and_then(|i| i.auto_renew_problem());
                    div()
                        .flex()
                        .flex_col()
                        .border_1()
                        .border_color(BORDER_PANEL)
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(10.0))
                                .px(px(10.0))
                                .h(px(28.0))
                                .bg(BG_SUBHEAD)
                                .child(mono(11.0, TEXT_PRIMARY).font_weight(FontWeight::BOLD).child(s.name.clone()))
                                .child(mono(9.5, TEXT_FAINT).child(tool))
                                .child(div().flex_1())
                                .child(match &problem {
                                    Some(p) => mono(9.5, WARN).child(p.clone()),
                                    None if inv.is_some_and(|i| i.as_ref().is_ok_and(|i| i.certbot || i.acme_sh.is_some())) => mono(9.5, OK).child("renews on its own"),
                                    None => mono(9.5, TEXT_FAINT).child(""),
                                }),
                        )
                        .when(certs.is_empty(), |d| d.child(div().px(px(10.0)).py(px(6.0)).bg(BG_PANEL).child(mono(10.0, TEXT_DIM).child("No certificates found (Let's Encrypt's live folder, nginx and Apache configs)."))))
                        .children(certs.iter().map(|c| {
                            let days = (c.expires - now) / 86_400;
                            let color = if days <= CERT_CRIT_DAYS { CRIT } else if days <= CERT_WARN_DAYS { WARN } else { OK };
                            let managed = certbot_name(&c.path).is_some() || inv.is_some_and(|i| i.as_ref().is_ok_and(|i| i.acme_sh.is_some()));
                            let (app, sid, path) = (app.clone(), s.id.clone(), c.path.clone());
                            div()
                                .flex()
                                .items_center()
                                .gap(px(10.0))
                                .px(px(10.0))
                                .min_h(px(30.0))
                                .border_t_1()
                                .border_color(BORDER_PANEL)
                                .bg(BG_PANEL)
                                .child(div().flex_none().size(px(7.0)).rounded_full().bg(color))
                                .child(mono(10.5, TEXT_PRIMARY).w(px(200.0)).flex_none().whitespace_nowrap().overflow_hidden().text_ellipsis().child(if c.name.is_empty() { "(no name)".to_string() } else { c.name.clone() }))
                                .child(mono(10.5, color).w(px(160.0)).flex_none().child(if days < 0 { format!("expired {} days ago", -days) } else { format!("{days} days left") }))
                                .child(mono(9.5, TEXT_FAINT).flex_1().min_w(px(0.0)).whitespace_nowrap().overflow_hidden().text_ellipsis().child(c.path.clone()))
                                .child(if managed {
                                    div()
                                        .id(ElementId::Name(format!("renew-{}-{}", s.id, c.path).into()))
                                        .flex_none()
                                        .px(px(8.0))
                                        .py(px(3.0))
                                        .border_1()
                                        .border_color(OK.opacity(0.6))
                                        .font_family(FONT_MONO)
                                        .text_size(px(9.5))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(OK)
                                        .cursor_pointer()
                                        .hover(|s| s.bg(OK.opacity(0.12)))
                                        .on_click(move |_ev, window, cx| {
                                            let (sid, path) = (sid.clone(), path.clone());
                                            app.update(cx, |this, cx| this.plan_cert_renewal(&sid, &path, window, cx))
                                        })
                                        .child("RENEW")
                                        .into_any_element()
                                } else {
                                    mono(9.0, TEXT_FAINT).flex_none().child("not certbot/acme.sh").into_any_element()
                                })
                        }))
                }))
                .when(st.inventory.is_empty() && st.reading.is_empty(), |d| d.child(mono(10.5, TEXT_DIM).child("Nothing read yet: READ AGAIN reads every reachable server."))),
        )
}
