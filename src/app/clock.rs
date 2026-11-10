//! Your time and the server's: the titlebar's clocks. Your time is this
//! computer's zone, or one picked from the zones you keep in Settings →
//! Personalisation → Time zones; the pick is remembered. The server's time
//! comes from its own clock's offset, read when Crow reaches it, with how
//! far ahead (green) or behind (red) of you it is.

use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use chrono::{DateTime, FixedOffset, Offset, TimeZone, Utc};
use chrono_tz::Tz;
use gpui_kit::component::input::{InputEvent, InputState};
use gpui_kit::*;

use super::CrowApp;
use crate::host::{host_for, DEFAULT_TIMEOUT};

/// The zones you keep, comma separated (`Europe/Madrid, Asia/Tokyo`).
pub const TIME_ZONES: &str = "appearance.time_zones";
/// The one your time is shown in; empty for this computer's.
pub const TIME_ZONE: &str = "appearance.time_zone";

/// A server's offset is read again after this, for daylight saving.
const REREAD: Duration = Duration::from_secs(30 * 60);

/// A server's clock as last read.
#[derive(Clone, Debug)]
pub struct ServerClock {
    /// Seconds east of UTC.
    pub offset: i32,
    /// Its zone's abbreviation (`CEST`, `UTC`), or the offset when it has none.
    pub abbr: String,
    pub read_at: Instant,
}

#[derive(Default)]
pub struct ClockState {
    /// The titlebar's zone picker is open.
    pub menu_open: bool,
    pub servers: HashMap<String, ServerClock>,
    /// Servers being read now.
    reading: HashSet<String>,
    /// Settings' zone search box (created on first render).
    pub search: Option<Entity<InputState>>,
    pub query: String,
    _subs: Vec<Subscription>,
}

/// Which zone your time is shown in.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Zone {
    /// This computer's.
    Local,
    Named(Tz),
}

impl Zone {
    pub fn offset_at(self, at: DateTime<Utc>) -> i32 {
        match self {
            Zone::Local => chrono::Local.offset_from_utc_datetime(&at.naive_utc()).fix().local_minus_utc(),
            Zone::Named(tz) => tz.offset_from_utc_datetime(&at.naive_utc()).fix().local_minus_utc(),
        }
    }

    /// The short name the titlebar shows: the city.
    pub fn label(self) -> String {
        match self {
            Zone::Local => local_name().map(|n| city(&n)).unwrap_or_else(|| t!("app-clock-local").to_string()),
            Zone::Named(tz) => city(tz.name()),
        }
    }
}

/// This computer's zone name (`Europe/Madrid`), when the system says.
pub fn local_name() -> Option<String> {
    iana_time_zone::get_timezone()
        .ok()
        .filter(|n| !n.is_empty())
        .or_else(|| {
            let link = std::fs::read_link("/etc/localtime").ok()?.into_os_string().into_string().ok()?;
            let (_, zone) = link.split_once("zoneinfo/")?;
            (!zone.is_empty()).then(|| zone.to_string())
        })
}

/// `America/Argentina/Buenos_Aires` → `Buenos Aires`.
pub fn city(name: &str) -> String {
    name.rsplit('/').next().unwrap_or(name).replace('_', " ")
}

/// The known zones in `list`, each once, in its order.
pub fn parse_zones(list: &str) -> Vec<Tz> {
    let mut out: Vec<Tz> = Vec::new();
    for tz in list.split(',').filter_map(|n| n.trim().parse::<Tz>().ok()) {
        if !out.contains(&tz) {
            out.push(tz);
        }
    }
    out
}

pub fn join_zones(zones: &[Tz]) -> String {
    zones.iter().map(|z| z.name()).collect::<Vec<_>>().join(", ")
}

/// `date +%z` (`+0530`, `-0800`) in seconds east of UTC.
pub fn parse_offset(s: &str) -> Option<i32> {
    let s = s.trim();
    let (sign, digits) = match s.as_bytes().first()? {
        b'+' => (1, &s[1..]),
        b'-' => (-1, &s[1..]),
        _ => return None,
    };
    let digits = digits.replace(':', "");
    if digits.len() != 4 || !digits.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let (h, m): (i32, i32) = (digits[..2].parse().ok()?, digits[2..].parse().ok()?);
    (h <= 14 && m < 60).then_some(sign * (h * 3600 + m * 60))
}

/// A `date '+%z %Z'` line as a server clock's offset and abbreviation.
pub fn parse_date_line(line: &str) -> Option<(i32, String)> {
    let mut parts = line.split_whitespace();
    let raw = parts.next()?;
    let offset = parse_offset(raw)?;
    let abbr = parts.next().filter(|a| a.chars().all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '-') && a.len() <= 8).unwrap_or(raw);
    Some((offset, abbr.to_string()))
}

/// How far the server is from you: `+2h`, `-5h30m`; `None` when level.
pub fn delta_label(secs: i32) -> Option<String> {
    if secs == 0 {
        return None;
    }
    let sign = if secs > 0 { '+' } else { '-' };
    let (h, m) = (secs.abs() / 3600, secs.abs() % 3600 / 60);
    Some(match (h, m) {
        (0, m) => format!("{sign}{m}m"),
        (h, 0) => format!("{sign}{h}h"),
        (h, m) => format!("{sign}{h}h{m:02}m"),
    })
}

/// `UTC+02:00`, `UTC-05:30`, `UTC`.
pub fn utc_label(secs: i32) -> String {
    if secs == 0 {
        return "UTC".into();
    }
    let sign = if secs > 0 { '+' } else { '-' };
    format!("UTC{sign}{:02}:{:02}", secs.abs() / 3600, secs.abs() % 3600 / 60)
}

/// What the titlebar's clocks show, worked out at render.
pub struct ClockLine {
    pub zone: String,
    pub local_time: String,
    /// The server's time, its zone, and how far it is from you
    /// (with whether it's ahead).
    pub server: Option<(String, String, Option<(String, bool)>)>,
    pub menu_open: bool,
}

fn config_text(config: &crate::config::CrowConfigManager, row_id: &str) -> String {
    config.get_field(row_id).and_then(|f| f.value.as_str().map(str::to_string)).unwrap_or_default()
}

/// The zones you keep.
pub fn zones_in(config: &crate::config::CrowConfigManager) -> Vec<Tz> {
    parse_zones(&config_text(config, TIME_ZONES))
}

/// The zone your time is in: the remembered pick while it's still one of
/// yours, else this computer's.
pub fn selected_in(config: &crate::config::CrowConfigManager) -> Zone {
    let picked = config_text(config, TIME_ZONE);
    zones_in(config).into_iter().find(|z| z.name() == picked.trim()).map_or(Zone::Local, Zone::Named)
}

impl CrowApp {
    pub fn saved_time_zones(&self) -> Vec<Tz> {
        zones_in(&self.config)
    }

    pub fn selected_zone(&self) -> Zone {
        selected_in(&self.config)
    }

    pub fn clock_line(&self, server_id: Option<&str>) -> ClockLine {
        let now = Utc::now();
        let zone = self.selected_zone();
        let mine = zone.offset_at(now);
        let at = |offset: i32| FixedOffset::east_opt(offset).map(|o| now.with_timezone(&o).format("%H:%M:%S").to_string()).unwrap_or_default();
        let server = server_id.map(|id| match self.clock.servers.get(id) {
            Some(c) => (at(c.offset), c.abbr.clone(), delta_label(c.offset - mine).map(|d| (d, c.offset > mine))),
            // Not read yet: UTC, which it is either way.
            None => (at(0), t!("titlebar-utc").to_string(), None),
        });
        ClockLine { zone: zone.label(), local_time: at(mine), server, menu_open: self.clock.menu_open }
    }

    pub fn toggle_clock_menu(&mut self, cx: &mut Context<Self>) {
        self.clock.menu_open = !self.clock.menu_open;
        cx.notify();
    }

    pub fn close_clock_menu(&mut self, cx: &mut Context<Self>) {
        if self.clock.menu_open {
            self.clock.menu_open = false;
            cx.notify();
        }
    }

    pub fn open_time_zone_settings(&mut self, cx: &mut Context<Self>) {
        self.clock.menu_open = false;
        self.set_screen(super::Screen::Settings, cx);
        self.set_settings_section(super::SettingsSection::Personalisation, cx);
    }

    /// Shows your time in `zone` (`None`: this computer's), and remembers it.
    pub fn select_time_zone(&mut self, zone: Option<Tz>, cx: &mut Context<Self>) {
        let name = zone.map(|z| z.name().to_string()).unwrap_or_default();
        self.update_config_field(TIME_ZONE, serde_json::Value::String(name), cx);
        let _ = self.config.save();
        self.clock.menu_open = false;
        cx.notify();
    }

    pub fn add_time_zone(&mut self, zone: Tz, cx: &mut Context<Self>) {
        let mut zones = self.saved_time_zones();
        if !zones.contains(&zone) {
            zones.push(zone);
            self.update_config_field(TIME_ZONES, serde_json::Value::String(join_zones(&zones)), cx);
            let _ = self.config.save();
        }
    }

    /// Drops `zone` from yours; your time goes back to this computer's if
    /// it was shown in it.
    pub fn remove_time_zone(&mut self, zone: Tz, cx: &mut Context<Self>) {
        let was_selected = self.selected_zone() == Zone::Named(zone);
        let zones: Vec<Tz> = self.saved_time_zones().into_iter().filter(|z| *z != zone).collect();
        self.update_config_field(TIME_ZONES, serde_json::Value::String(join_zones(&zones)), cx);
        if was_selected {
            self.update_config_field(TIME_ZONE, serde_json::Value::String(String::new()), cx);
        }
        let _ = self.config.save();
    }

    pub fn ensure_time_zone_search(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.clock.search.is_some() {
            return;
        }
        let input = cx.new(|cx| InputState::new(window, cx).placeholder(t!("app-clock-search-placeholder")));
        let sub = cx.subscribe(&input, |this, input, ev: &InputEvent, cx| {
            if matches!(ev, InputEvent::Change) {
                this.clock.query = input.read(cx).value().to_string();
                cx.notify();
            }
        });
        self.clock.search = Some(input);
        self.clock._subs.push(sub);
    }

    /// Reads the active server's clock offset in the background, unless it
    /// was read lately.
    pub fn read_server_clock(&mut self, cx: &mut Context<Self>) {
        let Some(srv) = self.fleet.active_server() else { return };
        if self.clock.servers.get(&srv.id).is_some_and(|c| c.read_at.elapsed() < REREAD) || !self.clock.reading.insert(srv.id.clone()) {
            return;
        }
        cx.spawn(async move |entity, cx| {
            let target = srv.clone();
            let read = cx
                .background_executor()
                .spawn(async move { host_for(&target).exec(&["date", "+%z %Z"], DEFAULT_TIMEOUT).ok().and_then(|o| parse_date_line(&o.stdout)) })
                .await;
            let _ = entity.update(cx, |this, cx| {
                this.clock.reading.remove(&srv.id);
                if let Some((offset, abbr)) = read {
                    this.clock.servers.insert(srv.id.clone(), ServerClock { offset, abbr, read_at: Instant::now() });
                    cx.notify();
                }
            });
        })
        .detach();
    }
}

/// Every zone whose name has each word of `query`, best first: names
/// starting with it, then the rest, alphabetical. At most `limit`.
pub fn search_zones(query: &str, limit: usize) -> Vec<Tz> {
    let q = query.trim().to_lowercase().replace(' ', "_");
    if q.is_empty() {
        return Vec::new();
    }
    let words: Vec<&str> = q.split('_').filter(|w| !w.is_empty()).collect();
    let mut hits: Vec<(bool, Tz)> = chrono_tz::TZ_VARIANTS
        .iter()
        .filter(|tz| {
            let n = tz.name().to_lowercase();
            words.iter().all(|w| n.contains(w))
        })
        .map(|tz| {
            let n = tz.name().to_lowercase();
            (!(n.starts_with(&q) || city(&n).replace(' ', "_").starts_with(&q)), *tz)
        })
        .collect();
    hits.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.name().cmp(b.1.name())));
    hits.into_iter().take(limit).map(|(_, tz)| tz).collect()
}

#[cfg(test)]
mod tests {
    use super::{city, delta_label, parse_date_line, parse_offset, parse_zones, search_zones, utc_label};

    #[test]
    fn offsets_from_date() {
        assert_eq!(parse_offset("+0200"), Some(7200));
        assert_eq!(parse_offset("-0530"), Some(-19800));
        assert_eq!(parse_offset("+05:45"), Some(20700));
        assert_eq!(parse_offset("0200"), None);
        assert_eq!(parse_offset("+9900"), None);
        assert_eq!(parse_date_line("+0200 CEST\n"), Some((7200, "CEST".into())));
        assert_eq!(parse_date_line("+0530 +0530"), Some((19800, "+0530".into())));
        assert_eq!(parse_date_line("-0300"), Some((-10800, "-0300".into())));
        assert_eq!(parse_date_line("date: invalid"), None);
    }

    #[test]
    fn deltas_say_ahead_or_behind() {
        assert_eq!(delta_label(0), None);
        assert_eq!(delta_label(7200).as_deref(), Some("+2h"));
        assert_eq!(delta_label(-19800).as_deref(), Some("-5h30m"));
        assert_eq!(delta_label(-2700).as_deref(), Some("-45m"));
        assert_eq!(utc_label(0), "UTC");
        assert_eq!(utc_label(-12600), "UTC-03:30");
    }

    #[test]
    fn zone_lists_keep_known_zones_once() {
        let z = parse_zones("Europe/Madrid, Nowhere/Land,Asia/Tokyo, Europe/Madrid,");
        assert_eq!(z.iter().map(|t| t.name()).collect::<Vec<_>>(), ["Europe/Madrid", "Asia/Tokyo"]);
        assert!(parse_zones("").is_empty());
        assert_eq!(city("America/Argentina/Buenos_Aires"), "Buenos Aires");
    }

    #[test]
    fn search_puts_starts_first() {
        let hits = search_zones("tok", 5);
        assert_eq!(hits.first().map(|t| t.name()), Some("Asia/Tokyo"));
        let hits = search_zones("new york", 5);
        assert_eq!(hits.first().map(|t| t.name()), Some("America/New_York"));
        assert!(search_zones("  ", 5).is_empty());
    }
}
