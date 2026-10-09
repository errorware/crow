//! The command palette's search (ERR-135): Crow's own things (servers,
//! places, the current server's config files, actions, keys), matched
//! fuzzily the way editors do, so `dbp` finds `db-prod`. Everything is in
//! memory; nothing reaches a server while you type.

/// What running an entry does.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Target {
    /// A server's page (`overview`, `terminal`, `config`, …).
    Server { id: String, view: &'static str },
    /// A page of the server that's open now.
    Page(&'static str),
    Screen(Place),
    /// A config file of the current server, by its name in the Config list.
    ConfigFile(String),
    Action(Action),
    /// The Keys page, with this key's name in mind.
    Key(String),
    /// A service of the server that's open now (its Services page, focused).
    Service(String),
    /// A process of the server that's open now, by PID.
    Process(u32),
    /// A config file Crow has read on any server (ERR-136).
    FleetConfig { server_id: String, path: String },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Place {
    Fleet,
    FleetSetup,
    FleetMap,
    /// Fleet Setup → PATCHING / ROLLOUTS / DRIFT & SEARCH.
    Patching,
    Rollouts,
    Drift,
    Audit,
    Settings,
    Keys,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    AddServer,
    LocalLab,
    CheckPosture,
    Reconnect,
    LockVault,
    About,
    /// The current server's Danger Zone, where its provider takes snapshots.
    Snapshot,
    /// The security stance (2FA) panel.
    Stance,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    /// SERVER, PAGE, PLACE, CONFIG, ACTION, KEY.
    pub category: &'static str,
    pub label: String,
    /// Shown dimmer after the label (an address, a path, a fingerprint).
    pub hint: String,
    /// Also matched, but worth less than the label (env, group, tags…).
    pub keywords: String,
    pub target: Target,
}

impl Entry {
    /// Identifies the entry across rebuilds, for "recently used".
    pub fn key(&self) -> String {
        format!("{}:{:?}", self.category, self.target)
    }
}

fn is_boundary(prev: Option<char>, c: char) -> bool {
    match prev {
        None => true,
        Some(p) => matches!(p, ' ' | '-' | '_' | '/' | '.' | ':' | '@') || (p.is_lowercase() && c.is_uppercase()),
    }
}

/// How well `query` matches `text` as a subsequence (case-insensitive), or
/// `None`. Consecutive runs, word starts and an early first hit score
/// higher; skipped characters cost a little.
pub fn fuzzy_score(query: &str, text: &str) -> Option<i32> {
    let q: Vec<char> = query.chars().filter(|c| !c.is_whitespace()).flat_map(char::to_lowercase).collect();
    if q.is_empty() {
        return Some(0);
    }
    let t: Vec<char> = text.chars().collect();
    let lower: Vec<char> = t.iter().map(|c| c.to_lowercase().next().unwrap_or(*c)).collect();
    // score[i]: best score with the current query char matched at text[i].
    const NONE: i32 = i32::MIN / 4;
    let mut prev_row: Vec<i32> = vec![NONE; t.len()];
    for (qi, qc) in q.iter().enumerate() {
        let mut row = vec![NONE; t.len()];
        // Best previous match at j <= i-2, minus one per character skipped.
        let mut skipped = NONE;
        for i in 0..t.len() {
            if i >= 2 {
                skipped = (skipped - 1).max(prev_row[i - 2] - 1).max(NONE);
            }
            if lower[i] != *qc {
                continue;
            }
            let bonus = if is_boundary(i.checked_sub(1).map(|j| t[j]), t[i]) { 10 } else { 0 };
            row[i] = if qi == 0 {
                // An early start is better: a hit at 0 beats one far in.
                16 + bonus - (i as i32).min(12)
            } else {
                let run = if i >= 1 && prev_row[i - 1] > NONE / 2 { prev_row[i - 1] + 16 } else { NONE };
                let jump = if skipped > NONE / 2 { skipped + 4 + bonus } else { NONE };
                run.max(jump)
            };
        }
        prev_row = row;
    }
    let best = prev_row.into_iter().max().filter(|s| *s > NONE / 2)?;
    // A whole-prefix match reads as the one you meant.
    let prefix = text.to_lowercase().starts_with(&q.iter().collect::<String>());
    Some(best + if prefix { 20 } else { 0 })
}

/// Where `query` matches in `text` (char indices), the same alignment
/// `fuzzy_score` rates best, for highlighting. Empty when it doesn't match.
pub fn match_positions(query: &str, text: &str) -> Vec<usize> {
    let q: Vec<char> = query.chars().filter(|c| !c.is_whitespace()).flat_map(char::to_lowercase).collect();
    let t: Vec<char> = text.chars().collect();
    if q.is_empty() || t.is_empty() {
        return Vec::new();
    }
    let lower: Vec<char> = t.iter().map(|c| c.to_lowercase().next().unwrap_or(*c)).collect();
    const NONE: i32 = i32::MIN / 4;
    // score[qi][i] and, for qi > 0, the text index of the previous match.
    let mut score = vec![vec![NONE; t.len()]; q.len()];
    let mut from = vec![vec![usize::MAX; t.len()]; q.len()];
    for (qi, qc) in q.iter().enumerate() {
        let (mut skipped, mut skipped_at) = (NONE, usize::MAX);
        for i in 0..t.len() {
            if qi > 0 && i >= 2 {
                let cand = score[qi - 1][i - 2] - 1;
                if cand >= skipped - 1 {
                    (skipped, skipped_at) = (cand, i - 2);
                } else {
                    skipped -= 1;
                }
            }
            if lower[i] != *qc {
                continue;
            }
            let bonus = if is_boundary(i.checked_sub(1).map(|j| t[j]), t[i]) { 10 } else { 0 };
            if qi == 0 {
                score[0][i] = 16 + bonus - (i as i32).min(12);
                continue;
            }
            let run = if i >= 1 && score[qi - 1][i - 1] > NONE / 2 { score[qi - 1][i - 1] + 16 } else { NONE };
            let jump = if skipped > NONE / 2 { skipped + 4 + bonus } else { NONE };
            if run >= jump && run > NONE / 2 {
                (score[qi][i], from[qi][i]) = (run, i - 1);
            } else if jump > NONE / 2 {
                (score[qi][i], from[qi][i]) = (jump, skipped_at);
            }
        }
    }
    let last = q.len() - 1;
    let Some(mut i) = (0..t.len()).filter(|&i| score[last][i] > NONE / 2).max_by_key(|&i| score[last][i]) else { return Vec::new() };
    let mut out = vec![i];
    for qi in (1..q.len()).rev() {
        i = from[qi][i];
        if i == usize::MAX {
            return Vec::new();
        }
        out.push(i);
    }
    out.reverse();
    out
}

/// `match_positions` as byte ranges of `text`, adjacent ones merged.
pub fn match_ranges(query: &str, text: &str) -> Vec<std::ops::Range<usize>> {
    let starts: Vec<(usize, char)> = text.char_indices().collect();
    let mut out: Vec<std::ops::Range<usize>> = Vec::new();
    for ci in match_positions(query, text) {
        let Some(&(b, c)) = starts.get(ci) else { continue };
        let r = b..b + c.len_utf8();
        match out.last_mut() {
            Some(last) if last.end == r.start => last.end = r.end,
            _ => out.push(r),
        }
    }
    out
}

/// The entries matching `query`, best first. With an empty query: recently
/// used first (most recent first), then the rest in their given order.
pub fn rank<'a>(entries: &'a [Entry], query: &str, recent: &[String]) -> Vec<&'a Entry> {
    let recency = |e: &Entry| recent.iter().position(|k| *k == e.key());
    if query.trim().is_empty() {
        let mut out: Vec<(usize, &Entry)> = entries.iter().enumerate().collect();
        out.sort_by_key(|(i, e)| (recency(e).unwrap_or(usize::MAX), *i));
        return out.into_iter().map(|(_, e)| e).collect();
    }
    let mut scored: Vec<(i32, usize, &Entry)> = entries
        .iter()
        .enumerate()
        .filter_map(|(i, e)| {
            let label = fuzzy_score(query, &e.label);
            let other = fuzzy_score(query, &format!("{} {}", e.hint, e.keywords)).map(|s| s - 25);
            let s = label.max(other)?;
            // A little lift for what was used lately.
            let lift = recency(e).map_or(0, |r| 8 - (r as i32).min(8));
            Some((s + lift, i, e))
        })
        .collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    scored.into_iter().map(|(_, _, e)| e).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(category: &'static str, label: &str, keywords: &str) -> Entry {
        Entry { category, label: label.into(), hint: String::new(), keywords: keywords.into(), target: Target::Page("overview") }
    }

    #[test]
    fn fuzzy_matches_subsequences_and_prefers_word_starts() {
        assert!(fuzzy_score("dbp", "db-prod").is_some());
        assert!(fuzzy_score("xyz", "db-prod").is_none());
        assert!(fuzzy_score("hba", "pg_hba.conf").unwrap() > fuzzy_score("hba", "kubeadm-hosts").unwrap_or(i32::MIN));
        assert!(fuzzy_score("web", "web-1").unwrap() > fuzzy_score("web", "my-weird-box").unwrap(), "prefix and run beat scattered");
        assert!(fuzzy_score("WEB", "web-1").is_some(), "case doesn't matter");
        assert_eq!(fuzzy_score("", "anything"), Some(0));
    }

    #[test]
    fn ranking_puts_the_meant_thing_first() {
        let entries = vec![
            entry("SERVER", "api-staging", "STAGING"),
            entry("SERVER", "db-prod", "PROD database"),
            entry("CONFIG", "pg_hba.conf", ""),
            entry("PAGE", "Firewall", ""),
            entry("ACTION", "Lock the vault", ""),
        ];
        let first = |q: &str| rank(&entries, q, &[]).first().map(|e| e.label.clone());
        assert_eq!(first("dbp").as_deref(), Some("db-prod"));
        assert_eq!(first("hba").as_deref(), Some("pg_hba.conf"));
        assert_eq!(first("fire").as_deref(), Some("Firewall"));
        assert_eq!(first("lock").as_deref(), Some("Lock the vault"));
        assert_eq!(first("staging").as_deref(), Some("api-staging"), "keywords match too");
        assert!(rank(&entries, "qqq", &[]).is_empty());
    }

    #[test]
    fn recently_used_comes_first_when_the_box_is_empty() {
        let page = |label: &str, view: &'static str| Entry { target: Target::Page(view), ..entry("PAGE", label, "") };
        let entries = vec![page("Overview", "overview"), page("Logs", "logs"), page("Users", "users")];
        let recent = vec![entries[2].key(), entries[1].key()];
        let order: Vec<&str> = rank(&entries, "", &recent).iter().map(|e| e.label.as_str()).collect();
        assert_eq!(order, ["Users", "Logs", "Overview"]);
    }

    #[test]
    fn highlights_follow_the_best_match() {
        assert_eq!(match_ranges("dbp", "db-prod"), vec![0..2, 3..4]);
        assert_eq!(match_ranges("hba", "pg_hba.conf"), vec![3..6], "the word start, not the first h");
        assert!(match_ranges("zzz", "db-prod").is_empty());
        assert_eq!(match_ranges("é", "café"), vec![3..5], "byte ranges, not chars");
    }
}
