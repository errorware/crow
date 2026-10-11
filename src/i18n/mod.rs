//! Translations (ERR-171). Every string Crow shows comes from a Fluent
//! message: `t!("fleet-enroll")`, `t!("fleet-servers", count = n)`.
//!
//! English is built in (`assets/i18n/en.ftl`) and is both the reference and
//! the fallback. Other languages are files the user owns: `.ftl` packs in
//! `~/.config/crow/languages/`, picked in Settings → Personalisation. A pack
//! is someone else's words shown as Crow's own, so it is held to English:
//! a message that doesn't parse, uses other placeholders than English, or
//! carries control or direction-override characters is ignored and English
//! shows instead.

#[cfg(test)]
mod guard;
pub mod pack;
pub mod pseudo;

use std::borrow::Cow;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock, RwLock};

use fluent_bundle::concurrent::FluentBundle;
use fluent_bundle::{FluentArgs, FluentResource, FluentValue};

pub use pack::{LanguageInfo, PackReport};

/// The English messages, the reference every pack is checked against.
pub const ENGLISH_FTL: &str = include_str!("../../assets/i18n/en.ftl");

/// The code of the built-in language.
pub const ENGLISH: &str = "en";

/// A loaded language: its bundle, plus the English one for what it lacks.
pub struct Language {
    pub info: LanguageInfo,
    bundle: Option<FluentBundle<FluentResource>>,
    /// Messages without placeables, formatted once and kept for the life
    /// of the program (`t!("id")` is a `&'static str`, like the literal it
    /// replaced). Bounded: one copy per message per language loaded.
    plain: RwLock<HashMap<&'static str, &'static str>>,
    /// Translated text → message id, for `english_of`.
    reverse: OnceLock<HashMap<String, &'static str>>,
}

impl Language {
    fn english() -> Self {
        Language { info: LanguageInfo::english(), bundle: None, plain: RwLock::new(HashMap::new()), reverse: OnceLock::new() }
    }

    fn with_bundle(info: LanguageInfo, bundle: FluentBundle<FluentResource>) -> Self {
        Language { info, bundle: Some(bundle), plain: RwLock::new(HashMap::new()), reverse: OnceLock::new() }
    }

    pub fn code(&self) -> &str {
        &self.info.code
    }
}

fn english_bundle() -> &'static FluentBundle<FluentResource> {
    static EN: OnceLock<FluentBundle<FluentResource>> = OnceLock::new();
    EN.get_or_init(|| {
        let res = FluentResource::try_new(ENGLISH_FTL.to_string()).unwrap_or_else(|(res, errors)| {
            debug_assert!(errors.is_empty(), "en.ftl doesn't parse: {errors:?}");
            res
        });
        let mut b = FluentBundle::new_concurrent(vec!["en".parse::<unic_langid::LanguageIdentifier>().unwrap()]);
        b.set_use_isolating(false);
        let _ = b.add_resource(res);
        b
    })
}

/// Every English message id, in file order.
pub fn english_ids() -> &'static [String] {
    static IDS: OnceLock<Vec<String>> = OnceLock::new();
    IDS.get_or_init(|| pack::message_ids(ENGLISH_FTL))
}

/// English, for `in_english` (its own cache of `plain` texts).
fn english_language() -> Arc<Language> {
    static EN: OnceLock<Arc<Language>> = OnceLock::new();
    EN.get_or_init(|| Arc::new(Language::english())).clone()
}

fn current_slot() -> &'static RwLock<Arc<Language>> {
    static CURRENT: OnceLock<RwLock<Arc<Language>>> = OnceLock::new();
    CURRENT.get_or_init(|| RwLock::new(Arc::new(Language::english())))
}

/// The language in use.
pub fn current() -> Arc<Language> {
    current_slot().read().map(|l| l.clone()).unwrap_or_else(|e| e.into_inner().clone())
}

fn set_current(lang: Language) {
    match current_slot().write() {
        Ok(mut slot) => *slot = Arc::new(lang),
        Err(e) => *e.into_inner() = Arc::new(lang),
    }
}

/// Where language packs live.
pub fn languages_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_default().join(".config").join("crow").join("languages")
}

/// Switches to `code` ("en", a pack's code, or the pseudo-locale). Unknown or
/// unreadable packs leave English in place; the report says why.
pub fn select(code: &str) -> Result<PackReport, String> {
    if code.is_empty() || code == ENGLISH {
        set_current(Language::english());
        return Ok(PackReport::english());
    }
    if code == pseudo::CODE {
        let mut b = FluentBundle::new_concurrent(vec!["en".parse::<unic_langid::LanguageIdentifier>().unwrap()]);
        b.set_use_isolating(false);
        b.set_transform(Some(pseudo::transform));
        let _ = b.add_resource(FluentResource::try_new(ENGLISH_FTL.to_string()).unwrap_or_else(|(r, _)| r));
        set_current(Language::with_bundle(pseudo::info(), b));
        return Ok(PackReport::english());
    }
    let path = pack::find(&languages_dir(), code).ok_or_else(|| format!("no {code}.ftl in {}", languages_dir().display()))?;
    let loaded = pack::load(&path)?;
    let report = loaded.report.clone();
    set_current(Language::with_bundle(loaded.info, loaded.bundle));
    Ok(report)
}

thread_local! {
    static IN_ENGLISH: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Runs `f` with every message in English on this thread: for documents
/// Crow writes out (reports, exports), which are shared with people who may
/// not read the language Crow is in.
pub fn in_english<R>(f: impl FnOnce() -> R) -> R {
    let was = IN_ENGLISH.with(|c| c.replace(true));
    let r = f();
    IN_ENGLISH.with(|c| c.set(was));
    r
}

fn forced_english() -> bool {
    IN_ENGLISH.with(|c| c.get())
}

/// The text of `id` with `args` in the current language, English when the
/// pack lacks it, the id itself when English does too (a bug: tests catch it).
pub fn format(id: &str, args: Option<&FluentArgs>) -> String {
    if forced_english() {
        return english(id, args);
    }
    let lang = current();
    lang.bundle
        .as_ref()
        .and_then(|b| format_in(b, id, args))
        .or_else(|| format_in(english_bundle(), id, args))
        .unwrap_or_else(|| id.to_string())
}

/// `t!("id")`: a message without arguments, as a `&'static str`.
pub fn plain(id: &'static str) -> &'static str {
    let lang = if forced_english() { english_language() } else { current() };
    if let Some(s) = lang.plain.read().ok().and_then(|m| m.get(id).copied()) {
        return s;
    }
    let text: &'static str = Box::leak(format(id, None).into_boxed_str());
    if let Ok(mut m) = lang.plain.write() {
        return m.entry(id).or_insert(text);
    }
    text
}

/// The English text of `id`, whatever the language: danger prompts show it
/// under the translation.
pub fn english(id: &str, args: Option<&FluentArgs>) -> String {
    format_in(english_bundle(), id, args).unwrap_or_else(|| id.to_string())
}

/// The English text under a translated danger prompt, or None in English.
pub fn english_beside(id: &str, args: Option<&FluentArgs>) -> Option<String> {
    (current().code() != ENGLISH).then(|| english(id, args)).filter(|en| *en != format(id, args))
}

/// The English of a text shown in the current language (a message without
/// arguments), when that's another language: what a translated danger
/// prompt also shows.
pub fn english_of(text: &str) -> Option<String> {
    let lang = current();
    if lang.code() == ENGLISH {
        return None;
    }
    let map = lang.reverse.get_or_init(|| {
        let b = lang.bundle.as_ref();
        english_ids().iter().filter_map(|id| b.and_then(|b| format_in(b, id, None)).map(|t| (t, id.as_str()))).collect()
    });
    map.get(text).map(|id| english(id, None)).filter(|en| en != text)
}

fn format_in(bundle: &FluentBundle<FluentResource>, id: &str, args: Option<&FluentArgs>) -> Option<String> {
    let msg = bundle.get_message(id)?;
    let pattern = msg.value()?;
    let mut errors = vec![];
    let text = bundle.format_pattern(pattern, args, &mut errors);
    errors.is_empty().then(|| text.into_owned())
}

/// Translates a string that is still English in the source (a static table,
/// a label from a manifest): the English message with that exact text, or
/// the text itself when there is none.
pub fn tx(english_text: &str) -> &str {
    match english_by_text().get(english_text) {
        Some(id) if current().code() != ENGLISH => plain(id),
        _ => english_text,
    }
}

fn english_by_text() -> &'static HashMap<String, &'static str> {
    static BY_TEXT: OnceLock<HashMap<String, &'static str>> = OnceLock::new();
    BY_TEXT.get_or_init(|| english_ids().iter().filter_map(|id| format_in(english_bundle(), id, None).map(|text| (text, id.as_str()))).collect())
}

/// Whether `tx` can translate this English text (a message has it).
pub fn has_english_text(text: &str) -> bool {
    english_by_text().contains_key(text)
}

/// An argument to a message: numbers stay numbers (plural rules select on
/// them), everything else is its text.
pub struct ArgWrap<'a, T: ?Sized>(pub &'a T);

pub trait ArgNumber {
    fn fluent_arg(&self) -> FluentValue<'static>;
}

pub trait ArgDisplay {
    fn fluent_arg(&self) -> FluentValue<'static>;
}

impl<T: std::fmt::Display + ?Sized> ArgDisplay for ArgWrap<'_, T> {
    fn fluent_arg(&self) -> FluentValue<'static> {
        FluentValue::String(Cow::Owned(self.0.to_string()))
    }
}

macro_rules! numeric_args {
    ($($t:ty),*) => {$(
        impl ArgNumber for &ArgWrap<'_, $t> {
            fn fluent_arg(&self) -> FluentValue<'static> {
                FluentValue::from(*self.0)
            }
        }
        impl ArgNumber for &ArgWrap<'_, &$t> {
            fn fluent_arg(&self) -> FluentValue<'static> {
                FluentValue::from(**self.0)
            }
        }
    )*};
}
numeric_args!(u8, u16, u32, u64, usize, i8, i16, i32, i64, isize, f32, f64);

/// The text of a message: `t!("id")` (a `&'static str`) or
/// `t!("id", name = value, …)` (a `String`).
#[macro_export]
macro_rules! t {
    ($id:literal) => {
        $crate::i18n::plain($id)
    };
    ($id:literal, $($name:ident = $value:expr),+ $(,)?) => {{
        #[allow(unused_imports)]
        use $crate::i18n::{ArgDisplay as _, ArgNumber as _};
        let mut args = fluent_bundle::FluentArgs::new();
        $( args.set(stringify!($name), (&&$crate::i18n::ArgWrap(&$value)).fluent_arg()); )+
        $crate::i18n::format($id, Some(&args))
    }};
}

/// Like `t!`, but the English text, whatever the language.
#[macro_export]
macro_rules! t_en {
    ($id:literal) => {
        $crate::i18n::english($id, None)
    };
    ($id:literal, $($name:ident = $value:expr),+ $(,)?) => {{
        #[allow(unused_imports)]
        use $crate::i18n::{ArgDisplay as _, ArgNumber as _};
        let mut args = fluent_bundle::FluentArgs::new();
        $( args.set(stringify!($name), (&&$crate::i18n::ArgWrap(&$value)).fluent_arg()); )+
        $crate::i18n::english($id, Some(&args))
    }};
}

/// For a danger prompt: the English text when another language is in use
/// (shown under the translation), else None.
#[macro_export]
macro_rules! t_beside {
    ($id:literal) => {
        $crate::i18n::english_beside($id, None)
    };
    ($id:literal, $($name:ident = $value:expr),+ $(,)?) => {{
        #[allow(unused_imports)]
        use $crate::i18n::{ArgDisplay as _, ArgNumber as _};
        let mut args = fluent_bundle::FluentArgs::new();
        $( args.set(stringify!($name), (&&$crate::i18n::ArgWrap(&$value)).fluent_arg()); )+
        $crate::i18n::english_beside($id, Some(&args))
    }};
}

/// `(text, English beside it)` for a danger prompt's title: the text in the
/// current language, and the English when that's another language.
#[macro_export]
macro_rules! t_pair {
    ($id:literal $(, $name:ident = $value:expr)* $(,)?) => {
        ($crate::t!($id $(, $name = $value)*).to_string(), $crate::t_beside!($id $(, $name = $value)*))
    };
}

#[cfg(test)]
mod tests {
    use super::{english_ids, format, ENGLISH_FTL};
    use fluent_bundle::FluentResource;

    #[test]
    fn english_parses_and_every_message_formats() {
        if let Err((_, errors)) = FluentResource::try_new(ENGLISH_FTL.to_string()) {
            panic!("en.ftl: {errors:?}");
        }
        let ids = english_ids();
        let mut seen = std::collections::HashSet::new();
        for id in ids {
            assert!(seen.insert(id), "{id} is defined twice");
        }
    }

    #[test]
    fn every_setting_shown_can_be_translated() {
        let manifest = &*crate::config::CROW_CONFIG_MANIFEST;
        for key in crate::config::WIRED_SETTINGS {
            let f = manifest.fields.iter().find(|f| f.name == *key).unwrap();
            let leaf = key.split('.').nth(1).unwrap();
            let label = crate::views::settings::config_table::format_field_label(leaf);
            assert!(super::has_english_text(&label), "en.ftl has no message for {key}'s label {label:?}");
            if let Some(help) = &f.help {
                assert!(super::has_english_text(help), "en.ftl has no message for {key}'s help");
            }
            for o in f.options.iter().flatten() {
                assert!(super::has_english_text(&o.label), "en.ftl has no message for {key}'s option {:?}", o.label);
            }
        }
    }

    #[test]
    fn every_plugin_card_can_be_translated() {
        for p in crate::plugins::BUILTIN {
            assert!(super::has_english_text(p.name) && super::has_english_text(p.about), "en.ftl has no message for plugin {}", p.id);
        }
    }

    #[test]
    fn numbers_stay_numbers_and_text_is_text() {
        let n = 3usize;
        let name = String::from("web-01");
        let a = t!("i18n-test-args", count = n, server = name);
        assert_eq!(a, "web-01: 3 servers");
        assert_eq!(t!("i18n-test-args", count = 1, server = "db"), "db: 1 server");
        assert_eq!(format("no-such-message", None), "no-such-message");
    }
}

