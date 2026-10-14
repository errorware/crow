//! A plugin's own translations (ERR-182): `i18n/<language>.ftl` in its
//! folder, for its labels. Messages are plain text: no placeholders, no
//! references, so a translation can't run or pull in anything. The ids:
//!
//! | id | translates |
//! |---|---|
//! | `plugin-name`, `plugin-about` | the plugin's name and about |
//! | `action-<id>`, `action-<id>-help` | an action's label and help |
//! | `input-<action>-<key>`, `input-<action>-<key>-help` | an input |
//! | `setting-<key>`, `setting-<key>-help` | a setting |
//! | `page-<id>` | a page's label |
//! | `page-<id>-block-<n>` | the title of a page's block n (from 1) |
//! | `column-<action>-<key>` | a table column's label |
//!
//! Anything missing stays in the plugin's English. The review screen always
//! shows the English, since that's what you decide to trust.

use std::collections::HashMap;
use std::path::Path;

use fluent_syntax::ast::{Entry, PatternElement};

use super::manifest::Manifest;

/// The plain-text messages of `ftl`, and the ids it refused (not plain).
pub fn messages(ftl: &str) -> (HashMap<String, String>, Vec<String>) {
    let res = match fluent_syntax::parser::parse(ftl) {
        Ok(r) => r,
        Err((r, _)) => r,
    };
    let (mut out, mut refused) = (HashMap::new(), Vec::new());
    for e in res.body {
        let Entry::Message(m) = e else { continue };
        let id = m.id.name.to_string();
        let plain = m.attributes.is_empty()
            && m.value.as_ref().is_some_and(|p| p.elements.iter().all(|el| matches!(el, PatternElement::TextElement { .. })));
        if !plain {
            refused.push(id);
            continue;
        }
        let text: String = m.value.map(|p| p.elements.into_iter().map(|el| if let PatternElement::TextElement { value } = el { value.to_string() } else { String::new() }).collect()).unwrap_or_default();
        let text = text.trim().to_string();
        if !text.is_empty() && !text.chars().any(|c| c.is_control() && c != '\n') {
            out.insert(id, text);
        }
    }
    (out, refused)
}

/// `m` with its labels from `msgs`; what isn't there stays as it was.
pub fn apply(m: &Manifest, msgs: &HashMap<String, String>) -> Manifest {
    let mut m = m.clone();
    let tr = |id: String, s: &mut String| {
        if let Some(t) = msgs.get(&id) {
            *s = t.clone();
        }
    };
    let tr_opt = |id: String, s: &mut Option<String>| {
        if let Some(t) = msgs.get(&id) {
            *s = Some(t.clone());
        }
    };
    tr("plugin-name".into(), &mut m.plugin.name);
    tr("plugin-about".into(), &mut m.plugin.about);
    for a in &mut m.actions {
        tr(format!("action-{}", a.id), &mut a.label);
        tr_opt(format!("action-{}-help", a.id), &mut a.help);
        for f in &mut a.inputs {
            tr(format!("input-{}-{}", a.id, f.key), &mut f.label);
            tr_opt(format!("input-{}-{}-help", a.id, f.key), &mut f.help);
        }
        for c in &mut a.output.columns {
            tr(format!("column-{}-{}", a.id, c.key), &mut c.label);
        }
    }
    for f in &mut m.settings {
        tr(format!("setting-{}", f.key), &mut f.label);
        tr_opt(format!("setting-{}-help", f.key), &mut f.help);
    }
    for p in &mut m.pages {
        tr(format!("page-{}", p.id), &mut p.label);
        for (i, b) in p.blocks.iter_mut().enumerate() {
            tr_opt(format!("page-{}-block-{}", p.id, i + 1), &mut b.title);
        }
    }
    m
}

/// `m` in language `code` when the plugin has it, else `None`.
pub fn localized(dir: &Path, m: &Manifest, code: &str) -> Option<Manifest> {
    if code.is_empty() || code == "en" || !code.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        return None;
    }
    let ftl = std::fs::read_to_string(dir.join("i18n").join(format!("{code}.ftl"))).ok()?;
    let (msgs, _) = messages(&ftl);
    (!msgs.is_empty()).then(|| apply(m, &msgs))
}

#[cfg(test)]
mod tests {
    use super::{apply, messages};
    use crate::plugin_host::manifest;

    #[test]
    fn plain_messages_only() {
        let (msgs, refused) = messages("plugin-name = Notas de inicio\naction-write = Escribir { $x }\naction-current = Nota actual\nsetting-a = { -term }\n");
        assert_eq!(msgs.get("plugin-name").map(String::as_str), Some("Notas de inicio"));
        assert_eq!(msgs.get("action-current").map(String::as_str), Some("Nota actual"));
        assert!(refused.contains(&"action-write".to_string()) && refused.contains(&"setting-a".to_string()));
    }

    #[test]
    fn labels_are_replaced_and_the_rest_stays() {
        let m = manifest::parse(include_str!("fixtures/motd-note/plugin.toml")).unwrap();
        let (msgs, _) = messages("plugin-name = ログインメモ\naction-write = メモを書く\ninput-write-note = メモ\npage-note = ログインメモ\n");
        let t = apply(&m, &msgs);
        assert_eq!(t.plugin.name, "ログインメモ");
        let write = t.actions.iter().find(|a| a.id == "write").unwrap();
        assert_eq!((write.label.as_str(), write.inputs[0].label.as_str()), ("メモを書く", "メモ"));
        assert_eq!(t.actions.iter().find(|a| a.id == "count").unwrap().label, "Count slowly", "untranslated stays English");
        assert_eq!(t.pages[0].label, "ログインメモ");
        // Nothing but labels changes: commands, effects and keywords are the plugin's.
        assert_eq!(write.command, m.actions.iter().find(|a| a.id == "write").unwrap().command);
        assert_eq!(write.keyword, Some("NOTE".to_string()));
    }
}
