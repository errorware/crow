//! Crow's own pages as tabs (ERR-190): Fleet configuration, Topology & DR
//! and Settings open as tabs beside the servers' and close the same way.
//! They're remembered with the others.

use gpui_kit::*;

use super::{CrowApp, Screen};

/// app_flags key: the open page tabs, in order, comma separated.
pub const OPEN_PAGES: &str = "fleet.open_pages";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PageTab {
    Setup,
    Topology,
    Settings,
}

impl PageTab {
    pub fn of(screen: Screen) -> Option<Self> {
        match screen {
            Screen::FleetSetup => Some(Self::Setup),
            Screen::Topology => Some(Self::Topology),
            Screen::Settings => Some(Self::Settings),
            _ => None,
        }
    }

    pub fn screen(self) -> Screen {
        match self {
            Self::Setup => Screen::FleetSetup,
            Self::Topology => Screen::Topology,
            Self::Settings => Screen::Settings,
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            Self::Setup => "setup",
            Self::Topology => "topology",
            Self::Settings => "settings",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Setup => t!("page-tabs-fleet-configuration"),
            Self::Topology => t!("page-tabs-topology"),
            Self::Settings => t!("page-tabs-settings"),
        }
    }
}

/// The page tabs saved as `saved`, in order; unknown names are dropped.
pub fn parse(saved: &str) -> Vec<PageTab> {
    let mut out = Vec::new();
    for t in saved.split(',').map(str::trim) {
        let tab = [PageTab::Setup, PageTab::Topology, PageTab::Settings].into_iter().find(|p| p.id() == t);
        if let Some(tab) = tab.filter(|t| !out.contains(t)) {
            out.push(tab);
        }
    }
    out
}

impl CrowApp {
    /// Opening one of Crow's pages opens its tab, once.
    pub(super) fn open_page_tab(&mut self, screen: Screen) {
        if let Some(tab) = PageTab::of(screen).filter(|t| !self.page_tabs.contains(t)) {
            self.page_tabs.push(tab);
            self.remember_page_tabs();
        }
    }

    /// Closes a page tab; on it, back to the fleet.
    pub fn close_page_tab(&mut self, tab: PageTab, cx: &mut Context<Self>) {
        self.page_tabs.retain(|t| *t != tab);
        self.remember_page_tabs();
        if self.screen == tab.screen() {
            if tab == PageTab::Settings {
                self.close_settings_dropdown(cx);
            }
            self.set_screen(Screen::Fleet, cx);
        }
        cx.notify();
    }

    fn remember_page_tabs(&self) {
        let ids = self.page_tabs.iter().map(|t| t.id()).collect::<Vec<_>>().join(",");
        if let Ok(db) = self.vault.db().lock() {
            let _ = db.set_flag(OPEN_PAGES, &ids);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{parse, PageTab};

    #[test]
    fn saved_page_tabs_come_back_in_order() {
        assert_eq!(parse("settings,topology"), [PageTab::Settings, PageTab::Topology]);
        assert_eq!(parse(" setup, nope ,setup,"), [PageTab::Setup]);
        assert!(parse("").is_empty());
    }
}
