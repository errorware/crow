//! Settings → Personalisation → Language (ERR-171): which language Crow's
//! screens are in, from the packs in the languages folder.

use gpui_kit::Context;

use super::CrowApp;
use crate::i18n::{self, LanguageInfo, PackReport};

pub const LANGUAGE: &str = "appearance.language";

/// A pack as the picker lists it.
#[derive(Clone, Debug)]
pub struct PackEntry {
    pub info: LanguageInfo,
    /// How complete it is, or why it can't be read.
    pub report: Result<PackReport, String>,
}

#[derive(Default)]
pub struct LanguageState {
    /// English first, then the packs found (and the pseudo-language when
    /// CROW_I18N_PSEUDO=1).
    pub entries: Vec<PackEntry>,
    pub listed: bool,
    /// What the last action did or why it failed.
    pub message: Option<(bool, String)>,
}

impl CrowApp {
    /// Reads the languages folder again.
    pub fn list_language_packs(&mut self, cx: &mut Context<Self>) {
        let mut entries = vec![PackEntry { info: LanguageInfo::english(), report: Ok(PackReport::english()) }];
        for info in i18n::pack::list(&i18n::languages_dir()) {
            let report = info.path.as_deref().ok_or_else(String::new).and_then(i18n::pack::load).map(|l| l.report);
            entries.push(PackEntry { info, report });
        }
        if i18n::pseudo::enabled() {
            entries.push(PackEntry { info: i18n::pseudo::info(), report: Ok(PackReport::english()) });
        }
        self.settings.language.entries = entries;
        self.settings.language.listed = true;
        cx.notify();
    }

    pub fn select_language(&mut self, code: &str, cx: &mut Context<Self>) {
        match i18n::select(code) {
            Ok(_) => {
                self.update_config_field(LANGUAGE, serde_json::Value::String(code.to_string()), cx);
                self.settings.language.message = None;
            }
            Err(e) => self.settings.language.message = Some((false, e)),
        }
        // Plugins' own translations follow (ERR-182).
        self.scan_user_plugins(cx);
        // Every window draws its text again in the new language.
        cx.refresh_windows();
        cx.notify();
    }

    pub fn open_languages_folder(&mut self, cx: &mut Context<Self>) {
        let dir = i18n::languages_dir();
        if let Err(e) = std::fs::create_dir_all(&dir) {
            self.settings.language.message = Some((false, t!("app-language-couldnt-create", path = dir.display(), e = e)));
            cx.notify();
            return;
        }
        #[cfg(target_os = "macos")]
        let _ = std::process::Command::new("open").arg(&dir).spawn();
        #[cfg(target_os = "linux")]
        let _ = std::process::Command::new("xdg-open").arg(&dir).spawn();
        #[cfg(target_os = "windows")]
        let _ = std::process::Command::new("explorer").arg(&dir).spawn();
    }

    /// Writes every English message, with a header to fill in, into the
    /// languages folder: the start of a new pack.
    pub fn export_language_template(&mut self, cx: &mut Context<Self>) {
        self.settings.language.message = Some(match i18n::pack::export_template(&i18n::languages_dir()) {
            Ok(path) => (true, t!("app-language-template-written", path = path.display())),
            Err(e) => (false, t!("app-language-template-not-written", e = e)),
        });
        cx.notify();
    }
}
