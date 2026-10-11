//! Language packs: `.ftl` files in the languages folder, checked against
//! English before anything from them is shown.

use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};

use fluent_bundle::concurrent::FluentBundle;
use fluent_bundle::FluentResource;
use fluent_syntax::ast;

use super::ENGLISH_FTL;

/// Packs bigger than this aren't read (English is about 300KB).
const MAX_PACK_BYTES: u64 = 4 * 1024 * 1024;

/// What a pack says about itself, in `### key: value` lines at the top:
/// `### name: Spanish`, `### native: Español`, `### author: …`,
/// `### crow: 0.9.0`, `### font: Noto Sans CJK JP`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LanguageInfo {
    /// The file's stem: `es` for `es.ftl`.
    pub code: String,
    pub name: String,
    pub native: String,
    pub author: String,
    /// The Crow version it was written for.
    pub crow: String,
    /// A font to draw it with, when Crow's own lacks its script.
    pub font: Option<String>,
    pub path: Option<PathBuf>,
}

impl LanguageInfo {
    pub fn english() -> Self {
        LanguageInfo { code: super::ENGLISH.into(), name: "English".into(), native: "English".into(), author: "Crow".into(), ..Default::default() }
    }

    /// How the picker names it: "Español (Spanish)", or the one name.
    pub fn display_name(&self) -> String {
        match (self.native.as_str(), self.name.as_str()) {
            ("", "") => self.code.clone(),
            (n, "") | ("", n) => n.to_string(),
            (native, name) if native == name => native.to_string(),
            (native, name) => format!("{native} ({name})"),
        }
    }
}

/// How complete and how sound a pack is.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PackReport {
    /// English messages the pack translates (and that passed the checks).
    pub translated: usize,
    pub total: usize,
    /// English messages it lacks.
    pub missing: usize,
    /// Messages it has that English doesn't (ignored).
    pub unknown: usize,
    /// Messages refused: syntax errors, other placeholders than English,
    /// control or direction-override characters. English shows instead.
    pub refused: Vec<(String, String)>,
    /// Syntax errors outside any message.
    pub syntax_errors: usize,
}

impl PackReport {
    pub fn english() -> Self {
        let total = super::english_ids().len();
        PackReport { translated: total, total, ..Default::default() }
    }

    pub fn percent(&self) -> usize {
        if self.total == 0 { 100 } else { self.translated * 100 / self.total }
    }
}

pub struct Loaded {
    pub info: LanguageInfo,
    pub bundle: FluentBundle<FluentResource>,
    pub report: PackReport,
}

/// The `### key: value` header of a pack.
pub fn header(text: &str, code: &str) -> LanguageInfo {
    let mut info = LanguageInfo { code: code.to_string(), ..Default::default() };
    for line in text.lines().take_while(|l| l.starts_with('#') || l.trim().is_empty()) {
        let Some(rest) = line.strip_prefix("###") else { continue };
        let Some((key, value)) = rest.split_once(':') else { continue };
        let value = clean(value.trim()).chars().take(80).collect::<String>();
        match key.trim().to_lowercase().as_str() {
            "name" => info.name = value,
            "native" => info.native = value,
            "author" => info.author = value,
            "crow" => info.crow = value,
            "font" if !value.is_empty() => info.font = Some(value),
            _ => {}
        }
    }
    info
}

/// The message ids of an `.ftl` text, in order.
pub fn message_ids(text: &str) -> Vec<String> {
    let res = FluentResource::try_new(text.to_string()).unwrap_or_else(|(r, _)| r);
    res.entries()
        .filter_map(|e| match e {
            ast::Entry::Message(m) if m.value.is_some() => Some(m.id.name.to_string()),
            _ => None,
        })
        .collect()
}

/// The packs in `dir`, sorted by name; unreadable files are left out.
pub fn list(dir: &Path) -> Vec<LanguageInfo> {
    let Ok(entries) = std::fs::read_dir(dir) else { return Vec::new() };
    let mut out: Vec<LanguageInfo> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "ftl"))
        .filter_map(|p| {
            let code = valid_code(p.file_stem()?.to_str()?)?;
            if code == super::ENGLISH || code == super::pseudo::CODE {
                return None;
            }
            let text = read_limited(&p).ok()?;
            let mut info = header(&text, &code);
            info.path = Some(p);
            Some(info)
        })
        .collect();
    out.sort_by_key(|a| a.display_name().to_lowercase());
    out
}

/// `es`, `pt-BR`, `zh-Hant`: letters, digits, `-` and `_`.
fn valid_code(stem: &str) -> Option<String> {
    (!stem.is_empty() && stem.len() <= 20 && stem.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')).then(|| stem.to_string())
}

pub fn find(dir: &Path, code: &str) -> Option<PathBuf> {
    valid_code(code)?;
    let p = dir.join(format!("{code}.ftl"));
    p.is_file().then_some(p)
}

fn read_limited(path: &Path) -> Result<String, String> {
    let meta = std::fs::metadata(path).map_err(|e| e.to_string())?;
    if meta.len() > MAX_PACK_BYTES {
        return Err(format!("{} is bigger than {} MB", path.display(), MAX_PACK_BYTES / 1024 / 1024));
    }
    std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))
}

/// Reads and checks a pack.
pub fn load(path: &Path) -> Result<Loaded, String> {
    let code = path.file_stem().and_then(|s| s.to_str()).and_then(valid_code).ok_or("a pack is named like es.ftl")?;
    let text = read_limited(path)?;
    let mut info = header(&text, &code);
    info.path = Some(path.to_path_buf());
    let (checked, report) = check(&text);
    let langid = code.replace('_', "-").parse().unwrap_or_else(|_| "en".parse::<unic_langid::LanguageIdentifier>().unwrap());
    let mut bundle = FluentBundle::new_concurrent(vec![langid]);
    bundle.set_use_isolating(false);
    let res = FluentResource::try_new(checked).unwrap_or_else(|(r, _)| r);
    let _ = bundle.add_resource(res);
    Ok(Loaded { info, bundle, report })
}

/// Keeps the messages of `text` that pass the checks; returns them as a new
/// `.ftl` text with the report.
pub fn check(text: &str) -> (String, PackReport) {
    let english = variables_by_id(ENGLISH_FTL);
    let (res, errors) = match FluentResource::try_new(text.to_string()) {
        Ok(r) => (r, vec![]),
        Err((r, e)) => (r, e),
    };
    let mut report = PackReport { total: english.len(), syntax_errors: errors.len(), ..Default::default() };
    let mut kept = String::new();
    let mut seen = BTreeSet::new();
    for entry in res.entries() {
        match entry {
            ast::Entry::Junk { .. } => {}
            ast::Entry::Term(term) => {
                // Terms (-brand = Crow) are shared wording; same checks.
                if let Some(src) = entry_source(text, &term.id.name, true) {
                    if forbidden_char(&src).is_none() {
                        kept.push_str(&src);
                        kept.push('\n');
                    }
                }
            }
            ast::Entry::Message(m) => {
                let id = m.id.name;
                let Some(want) = english.get(id) else {
                    report.unknown += 1;
                    continue;
                };
                if m.value.is_none() || !seen.insert(id.to_string()) {
                    continue;
                }
                let mut got = BTreeSet::new();
                if let Some(p) = &m.value {
                    pattern_variables(p, &mut got);
                }
                let Some(src) = entry_source(text, id, false) else { continue };
                if let Some(c) = forbidden_char(&src) {
                    report.refused.push((id.to_string(), format!("character U+{:04X}", c as u32)));
                } else if &got != want {
                    let list = |s: &BTreeSet<String>| s.iter().map(|v| format!("${v}")).collect::<Vec<_>>().join(" ");
                    report.refused.push((id.to_string(), format!("placeholders {} instead of {}", list(&got), list(want))));
                } else if references_messages(m) {
                    report.refused.push((id.to_string(), "refers to other messages".into()));
                } else {
                    kept.push_str(&src);
                    kept.push('\n');
                    report.translated += 1;
                }
            }
            _ => {}
        }
    }
    report.missing = report.total - report.translated;
    (kept, report)
}

/// The source text of one entry (`id = …` and its indented lines).
fn entry_source(text: &str, id: &str, term: bool) -> Option<String> {
    let head = if term { format!("-{id}") } else { id.to_string() };
    let mut lines = text.lines();
    let mut out = String::new();
    for line in lines.by_ref() {
        let trimmed = line.trim_end();
        if let Some(rest) = trimmed.strip_prefix(head.as_str()) {
            if rest.trim_start().starts_with('=') {
                out.push_str(line);
                out.push('\n');
                break;
            }
        }
    }
    if out.is_empty() {
        return None;
    }
    for line in lines {
        if line.starts_with(' ') || line.starts_with('}') || line.trim().is_empty() {
            out.push_str(line);
            out.push('\n');
        } else {
            break;
        }
    }
    Some(out.trim_end().to_string() + "\n")
}

/// Characters a translation may not carry: controls other than newline and
/// tab, and the bidi controls that can make text read differently from how
/// it's stored.
fn forbidden_char(s: &str) -> Option<char> {
    s.chars().find(|&c| {
        (c.is_control() && c != '\n' && c != '\t' && c != '\r')
            || matches!(c, '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}' | '\u{200E}' | '\u{200F}' | '\u{061C}')
    })
}

/// A pack's message may not pull in other messages: what a message says is
/// checked on its own.
fn references_messages(m: &ast::Message<&str>) -> bool {
    fn in_pattern(p: &ast::Pattern<&str>) -> bool {
        p.elements.iter().any(|e| match e {
            ast::PatternElement::Placeable { expression } => in_expr(expression),
            _ => false,
        })
    }
    fn in_inline(i: &ast::InlineExpression<&str>) -> bool {
        match i {
            ast::InlineExpression::MessageReference { .. } | ast::InlineExpression::FunctionReference { .. } => true,
            ast::InlineExpression::Placeable { expression } => in_expr(expression),
            _ => false,
        }
    }
    fn in_expr(e: &ast::Expression<&str>) -> bool {
        match e {
            ast::Expression::Inline(i) => in_inline(i),
            ast::Expression::Select { selector, variants } => in_inline(selector) || variants.iter().any(|v| in_pattern(&v.value)),
        }
    }
    m.value.as_ref().is_some_and(in_pattern)
}

/// Each message's variables: `{ $server }` → "server".
pub fn variables_by_id(text: &str) -> HashMap<String, BTreeSet<String>> {
    let res = FluentResource::try_new(text.to_string()).unwrap_or_else(|(r, _)| r);
    res.entries()
        .filter_map(|e| match e {
            ast::Entry::Message(m) => {
                let mut vars = BTreeSet::new();
                pattern_variables(m.value.as_ref()?, &mut vars);
                Some((m.id.name.to_string(), vars))
            }
            _ => None,
        })
        .collect()
}

fn pattern_variables(p: &ast::Pattern<&str>, out: &mut BTreeSet<String>) {
    for e in &p.elements {
        if let ast::PatternElement::Placeable { expression } = e {
            expr_variables(expression, out);
        }
    }
}

fn expr_variables(e: &ast::Expression<&str>, out: &mut BTreeSet<String>) {
    match e {
        ast::Expression::Inline(i) => inline_variables(i, out),
        ast::Expression::Select { selector, variants } => {
            inline_variables(selector, out);
            for v in variants {
                pattern_variables(&v.value, out);
            }
        }
    }
}

fn inline_variables(i: &ast::InlineExpression<&str>, out: &mut BTreeSet<String>) {
    match i {
        ast::InlineExpression::VariableReference { id } => {
            out.insert(id.name.to_string());
        }
        ast::InlineExpression::Placeable { expression } => expr_variables(expression, out),
        ast::InlineExpression::FunctionReference { arguments, .. } => {
            for a in &arguments.positional {
                inline_variables(a, out);
            }
        }
        _ => {}
    }
}

/// Removes control characters from header values.
fn clean(s: &str) -> String {
    s.chars().filter(|&c| forbidden_char(&c.to_string()).is_none()).collect()
}

/// `crow --check-language FILE`: prints the pack's coverage and what was
/// refused; exits 1 when anything was.
pub fn check_file_main(path: Option<&str>) -> i32 {
    let Some(path) = path else {
        eprintln!("usage: crow --check-language es.ftl");
        return 2;
    };
    let loaded = match load(Path::new(path)) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("{e}");
            return 2;
        }
    };
    let r = &loaded.report;
    println!("{} · {}% · {} of {} messages translated, {} missing", loaded.info.display_name(), r.percent(), r.translated, r.total, r.missing);
    if r.unknown > 0 {
        println!("{} messages English doesn't have (ignored; renamed or removed in this Crow?)", r.unknown);
    }
    if r.syntax_errors > 0 {
        println!("{} syntax errors (the lines around them are skipped)", r.syntax_errors);
    }
    for (id, why) in &r.refused {
        println!("refused {id}: {why}");
    }
    if std::env::args().any(|a| a == "--missing") {
        let have: BTreeSet<String> = message_ids(&check(&std::fs::read_to_string(path).unwrap_or_default()).0).into_iter().collect();
        for id in super::english_ids().iter().filter(|id| !have.contains(*id)) {
            println!("missing {id}");
        }
    }
    i32::from(!r.refused.is_empty() || r.syntax_errors > 0)
}

/// The English messages as a pack to translate from, with a header to fill.
pub fn template() -> String {
    let body = ENGLISH_FTL.lines().skip_while(|l| l.starts_with("###") || l.trim().is_empty()).collect::<Vec<_>>().join("\n");
    format!(
        "### name: (the language in English, e.g. Spanish)\n### native: (its own name, e.g. Español)\n### author: (you)\n### crow: {}\n### font: (optional: a font for scripts JetBrains Mono lacks, e.g. Noto Sans CJK JP)\n\n{body}\n",
        env!("CARGO_PKG_VERSION")
    )
}

/// Writes the template into the languages folder (never over a file).
pub fn export_template(dir: &Path) -> Result<PathBuf, String> {
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let mut path = dir.join("template.ftl.txt");
    let mut n = 2;
    while path.exists() {
        path = dir.join(format!("template-{n}.ftl.txt"));
        n += 1;
    }
    std::fs::write(&path, template()).map_err(|e| e.to_string())?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::{check, header, list, template};

    #[test]
    fn a_pack_is_held_to_english() {
        let pack = "### name: Spanish\n### native: Español\n\
            i18n-test-args = { $server }: { $count ->\n    [one] { $count } servidor\n   *[other] { $count } servidores\n}\n\
            i18n-test-plain = Hola \u{202E}olleh\n\
            not-in-english = ignorado\n";
        let (kept, report) = check(pack);
        assert!(kept.contains("servidores"));
        assert_eq!(super::message_ids(&kept), ["i18n-test-args"], "a select's closing brace at column 0 is kept");
        assert!(!kept.contains("Hola"), "a direction override is refused");
        assert_eq!(report.translated, 1);
        assert_eq!(report.unknown, 1);
        assert_eq!(report.refused.len(), 1);
        assert_eq!(report.missing, report.total - 1);

        let dropped = "i18n-test-args = { $count } servidores\n";
        let (_, report) = check(dropped);
        assert_eq!(report.translated, 0, "a placeholder English has can't go missing");
        assert!(report.refused[0].1.contains("$server"));

        let pulls = "i18n-test-plain = { i18n-test-args }\n";
        assert_eq!(check(pulls).1.translated, 0, "no pulling in other messages");
    }

    #[test]
    fn header_and_listing() {
        let info = header("### name: Japanese\n### native: 日本語\n### font: Noto Sans CJK JP\n\nx = y\n", "ja");
        assert_eq!(info.display_name(), "日本語 (Japanese)");
        assert_eq!(info.font.as_deref(), Some("Noto Sans CJK JP"));

        let dir = std::env::temp_dir().join(format!("crow-i18n-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("ja.ftl"), "### name: Japanese\n").unwrap();
        std::fs::write(dir.join("bad name.ftl"), "").unwrap();
        std::fs::write(dir.join("notes.txt"), "").unwrap();
        let codes: Vec<String> = list(&dir).into_iter().map(|l| l.code).collect();
        assert_eq!(codes, ["ja"]);
        std::fs::remove_dir_all(&dir).unwrap();

        assert!(template().starts_with("### name:"));
    }
}
