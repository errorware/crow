//! `plugin.toml`: what a recipe plugin declares (ERR-175). Parsing only;
//! nothing here reads a plugins folder or runs anything. The format is
//! described for plugin authors in `docs/plugins.md`.
//!
//! Commands are argv lists, never shell text. A `{placeholder}` is filled in
//! per argument, so an input becomes part of one argument and can't add
//! another. Templates and durations are checked while the TOML is read (their
//! errors carry a line); references between parts are checked after.

use std::collections::{BTreeMap, HashSet};
use std::fmt;

use serde::{Deserialize, Deserializer};

/// The manifest format this Crow reads.
pub const FORMAT: u32 = 1;

/// The longest an action may run: six hours.
pub const MAX_TIMEOUT_SECS: u64 = 6 * 3600;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub format: u32,
    pub plugin: PluginInfo,
    #[serde(default)]
    pub requires: Requires,
    pub setup: Option<Setup>,
    #[serde(default, rename = "setting")]
    pub settings: Vec<Field>,
    #[serde(default, rename = "action")]
    pub actions: Vec<Action>,
    #[serde(default, rename = "page")]
    pub pages: Vec<Page>,
    pub badge: Option<Badge>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub about: String,
    /// The Crow versions it works with, as a semver requirement (">=0.8.3").
    pub crow: String,
    pub license: Option<String>,
    pub homepage: Option<String>,
    pub category: Option<String>,
    #[serde(default)]
    pub capabilities: Vec<String>,
}

/// What must exist before anything runs.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Requires {
    /// Commands on this machine.
    #[serde(default)]
    pub local: Vec<String>,
    /// Commands on each server.
    #[serde(default)]
    pub server: Vec<String>,
    /// Distro families it supports; empty means any.
    #[serde(default)]
    pub os: Vec<String>,
}

/// A one-time install on this machine (a venv, a download), run before any
/// action is offered. It writes only inside the plugin's data folder.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Setup {
    pub command: Option<Vec<Template>>,
    pub script: Option<String>,
    pub interpreter: Option<Interpreter>,
    #[serde(default)]
    pub args: Vec<Template>,
    #[serde(default)]
    pub env: BTreeMap<String, Template>,
    #[serde(default = "Duration::default_timeout")]
    pub timeout: Duration,
}

/// A setting (per plugin) or an input (per run of an action).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Field {
    pub key: String,
    pub label: String,
    #[serde(rename = "type")]
    pub kind: FieldKind,
    pub default: Option<toml::Value>,
    #[serde(default)]
    pub choices: Vec<String>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    /// Which characters text may hold; anything else is refused before it
    /// reaches a command.
    #[serde(default)]
    pub charset: Charset,
    #[serde(default)]
    pub required: bool,
    pub help: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FieldKind {
    Text,
    Number,
    Bool,
    Choice,
    Choices,
    /// Kept in the vault; reaches an action only as an environment variable.
    Secret,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Charset {
    /// Letters, digits, `.`, `_`, `-`.
    #[default]
    Name,
    /// A name plus `/`, `:`, `@`, `~`, `+`, `=`, `,`.
    Path,
    /// Any printable text, no control characters.
    Any,
}

impl Charset {
    pub fn allows(self, c: char) -> bool {
        match self {
            Charset::Name => c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'),
            Charset::Path => Charset::Name.allows(c) || matches!(c, '/' | ':' | '@' | '~' | '+' | '=' | ','),
            Charset::Any => !c.is_control(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Action {
    pub id: String,
    pub label: String,
    pub help: Option<String>,
    pub on: Runs,
    pub effect: Effect,
    /// Server actions only: run through `sudo -n`.
    #[serde(default)]
    pub root: bool,
    pub command: Option<Vec<Template>>,
    /// A file in the plugin's folder. On a server it's sent on stdin to the
    /// interpreter, so nothing is left behind.
    pub script: Option<String>,
    pub interpreter: Option<Interpreter>,
    #[serde(default)]
    pub args: Vec<Template>,
    #[serde(default)]
    pub env: BTreeMap<String, Template>,
    #[serde(default = "Duration::default_timeout")]
    pub timeout: Duration,
    /// A read action run first, and shown before this one is confirmed.
    pub dry_run: Option<String>,
    /// Crow proves a fresh key login before and after; a failure after stops
    /// the run and says how to get back in.
    #[serde(default)]
    pub login_check: bool,
    /// Typed to confirm a danger action. Default: CONFIRM.
    pub keyword: Option<String>,
    #[serde(default, rename = "input")]
    pub inputs: Vec<Field>,
    #[serde(default)]
    pub output: Output,
}

impl Action {
    /// Every template the action fills in, with whether it's an env value.
    pub fn templates(&self) -> impl Iterator<Item = (&Template, bool)> {
        self.command.iter().flatten().chain(self.args.iter()).map(|t| (t, false)).chain(self.env.values().map(|t| (t, true)))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Runs {
    /// Over SSH, on each server it's run for.
    Server,
    /// On this machine, given an inventory of the servers it's run for.
    Local,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Effect {
    /// Changes nothing; runs without asking.
    Read,
    /// Changes something; asks first.
    Change,
    /// Could cut access or lose data; needs the typed keyword.
    Danger,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Interpreter {
    Sh,
    Bash,
    Python3,
}

impl Interpreter {
    pub fn program(self) -> &'static str {
        match self {
            Interpreter::Sh => "sh",
            Interpreter::Bash => "bash",
            Interpreter::Python3 => "python3",
        }
    }
}

/// How Crow reads what an action prints.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Output {
    #[serde(default)]
    pub format: OutputFormat,
    /// json: the key holding the rows (dotted), when the top level isn't the array.
    pub rows: Option<String>,
    /// json and table: which columns to show, in order.
    #[serde(default)]
    pub columns: Vec<Column>,
    /// The first line starting with this is the run's one-line summary.
    pub summary_prefix: Option<String>,
    /// Local actions only: the program writes one JSON line per server to
    /// `{results}`, and stdout stays a console for people.
    #[serde(default)]
    pub results: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    /// Shown as a console.
    #[default]
    Lines,
    Json,
    /// Columns split on whitespace, first line the header.
    Table,
    /// Only the exit code matters.
    Exit,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Column {
    pub key: String,
    pub label: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Page {
    pub id: String,
    pub label: String,
    pub icon: Option<String>,
    #[serde(rename = "where")]
    pub place: Place,
    #[serde(default, rename = "block")]
    pub blocks: Vec<Block>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Place {
    /// In a server's sidebar.
    Server,
    /// In Fleet configuration's side menu.
    Fleet,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Block {
    pub kind: BlockKind,
    pub action: Option<String>,
    #[serde(default)]
    pub actions: Vec<String>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BlockKind {
    /// ok / warn / bad and a label, from a read action's JSON.
    Status,
    /// Key and value pairs from a read action's JSON object.
    Kv,
    /// Rows from a read action.
    Table,
    /// Buttons for actions.
    Actions,
    /// The plugin's run history.
    Runs,
}

/// A status shown on the server's identity bar, the fleet table and map.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Badge {
    pub action: String,
    pub every: Duration,
    /// Adds "· <date>" of this action's last successful run.
    pub last_run: Option<String>,
}

// ---------------------------------------------------------------------------
// Templates

/// One argument or env value, with `{placeholders}`; `{{` and `}}` are braces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Template(pub Vec<Part>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Part {
    Text(String),
    Var(Var),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Var {
    Input(String),
    Setting(String),
    /// Env values only.
    Secret(String),
    /// Server actions only.
    Server(ServerField),
    /// Local actions and setup only.
    PluginDir,
    DataDir,
    Inventory,
    SshConfig,
    Results,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerField {
    Name,
    Host,
    User,
    Port,
}

impl Var {
    fn parse(name: &str) -> Result<Var, String> {
        let key = |k: &str| {
            if is_key(k) {
                Ok(k.to_string())
            } else {
                Err(format!("\"{k}\" isn't a key: use a-z, 0-9 and _"))
            }
        };
        Ok(match name.split_once('.') {
            Some(("input", k)) => Var::Input(key(k)?),
            Some(("setting", k)) => Var::Setting(key(k)?),
            Some(("secret", k)) => Var::Secret(key(k)?),
            Some(("server", "name")) => Var::Server(ServerField::Name),
            Some(("server", "host")) => Var::Server(ServerField::Host),
            Some(("server", "user")) => Var::Server(ServerField::User),
            Some(("server", "port")) => Var::Server(ServerField::Port),
            None if name == "plugin_dir" => Var::PluginDir,
            None if name == "data_dir" => Var::DataDir,
            None if name == "inventory" => Var::Inventory,
            None if name == "ssh_config" => Var::SshConfig,
            None if name == "results" => Var::Results,
            _ => return Err(format!("unknown placeholder {{{name}}}")),
        })
    }

    /// Only means something when the action runs on this machine.
    pub fn is_local_only(&self) -> bool {
        matches!(self, Var::PluginDir | Var::DataDir | Var::Inventory | Var::SshConfig | Var::Results)
    }
}

impl fmt::Display for Var {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Var::Input(k) => write!(f, "{{input.{k}}}"),
            Var::Setting(k) => write!(f, "{{setting.{k}}}"),
            Var::Secret(k) => write!(f, "{{secret.{k}}}"),
            Var::Server(s) => write!(f, "{{server.{}}}", match s {
                ServerField::Name => "name",
                ServerField::Host => "host",
                ServerField::User => "user",
                ServerField::Port => "port",
            }),
            Var::PluginDir => f.write_str("{plugin_dir}"),
            Var::DataDir => f.write_str("{data_dir}"),
            Var::Inventory => f.write_str("{inventory}"),
            Var::SshConfig => f.write_str("{ssh_config}"),
            Var::Results => f.write_str("{results}"),
        }
    }
}

impl Template {
    pub fn parse(s: &str) -> Result<Template, String> {
        let mut parts = Vec::new();
        let mut text = String::new();
        let mut chars = s.chars().peekable();
        while let Some(c) = chars.next() {
            match c {
                '{' if chars.peek() == Some(&'{') => {
                    chars.next();
                    text.push('{');
                }
                '}' if chars.peek() == Some(&'}') => {
                    chars.next();
                    text.push('}');
                }
                '{' => {
                    let mut name = String::new();
                    loop {
                        match chars.next() {
                            Some('}') => break,
                            Some(c) => name.push(c),
                            None => return Err(format!("\"{s}\": a {{ isn't closed")),
                        }
                    }
                    if !text.is_empty() {
                        parts.push(Part::Text(std::mem::take(&mut text)));
                    }
                    parts.push(Part::Var(Var::parse(&name).map_err(|e| format!("\"{s}\": {e}"))?));
                }
                '}' => return Err(format!("\"{s}\": a }} without a {{ (write }}}} for a brace)")),
                c => text.push(c),
            }
        }
        if !text.is_empty() {
            parts.push(Part::Text(text));
        }
        Ok(Template(parts))
    }

    pub fn vars(&self) -> impl Iterator<Item = &Var> {
        self.0.iter().filter_map(|p| match p {
            Part::Var(v) => Some(v),
            Part::Text(_) => None,
        })
    }
}

/// As it was written, for the review screen.
impl fmt::Display for Template {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for part in &self.0 {
            match part {
                Part::Text(t) => f.write_str(&t.replace('{', "{{").replace('}', "}}"))?,
                Part::Var(v) => write!(f, "{v}")?,
            }
        }
        Ok(())
    }
}

impl<'de> Deserialize<'de> for Template {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Template::parse(&String::deserialize(d)?).map_err(serde::de::Error::custom)
    }
}

// ---------------------------------------------------------------------------
// Durations: "90s", "30m", "6h".

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Duration(pub u64);

impl Duration {
    fn default_timeout() -> Duration {
        Duration(300)
    }

    pub fn parse(s: &str) -> Result<Duration, String> {
        let (n, unit) = s.split_at(s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len()));
        let n: u64 = n.parse().map_err(|_| format!("\"{s}\" isn't a duration: write 90s, 30m or 6h"))?;
        let secs = match unit {
            "s" => n,
            "m" => n * 60,
            "h" => n * 3600,
            _ => return Err(format!("\"{s}\" isn't a duration: write 90s, 30m or 6h")),
        };
        Ok(Duration(secs))
    }
}

impl<'de> Deserialize<'de> for Duration {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Duration::parse(&String::deserialize(d)?).map_err(serde::de::Error::custom)
    }
}

// ---------------------------------------------------------------------------
// Parsing and the checks between parts

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestError(pub Vec<String>);

impl fmt::Display for ManifestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0.join("\n"))
    }
}

impl std::error::Error for ManifestError {}

pub fn parse(text: &str) -> Result<Manifest, ManifestError> {
    // The format number first, so a newer manifest says so instead of
    // failing on a key this Crow doesn't know.
    if let Ok(v) = text.parse::<toml::Table>() {
        if let Some(n) = v.get("format").and_then(|f| f.as_integer()) {
            if n != FORMAT as i64 {
                return Err(ManifestError(vec![format!("format = {n}: this Crow reads format {FORMAT}")]));
            }
        }
    }
    let m: Manifest = toml::from_str(text).map_err(|e| ManifestError(vec![e.to_string().trim_end().to_string()]))?;
    let problems = check(&m);
    if problems.is_empty() {
        Ok(m)
    } else {
        Err(ManifestError(problems))
    }
}

fn is_id(s: &str) -> bool {
    (2..=40).contains(&s.len()) && s.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') && !s.starts_with('-') && !s.ends_with('-')
}

fn is_key(s: &str) -> bool {
    (1..=40).contains(&s.len()) && s.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_') && !s.starts_with(|c: char| c.is_ascii_digit())
}

fn is_env_name(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_') && !s.starts_with(|c: char| c.is_ascii_digit())
}

fn check(m: &Manifest) -> Vec<String> {
    let mut out = Vec::new();
    let mut bad = |at: &str, msg: String| out.push(format!("{at}: {msg}"));

    if m.format != FORMAT {
        bad("format", format!("this Crow reads format {FORMAT}"));
    }
    let p = &m.plugin;
    if !is_id(&p.id) {
        bad("plugin.id", format!("\"{}\": use 2–40 of a-z, 0-9 and -", p.id));
    }
    if semver::Version::parse(&p.version).is_err() {
        bad("plugin.version", format!("\"{}\" isn't a version like 1.2.0", p.version));
    }
    if semver::VersionReq::parse(&p.crow).is_err() {
        bad("plugin.crow", format!("\"{}\" isn't a version requirement like >=0.8.3", p.crow));
    }
    for (at, s) in [("plugin.name", &p.name), ("plugin.author", &p.author), ("plugin.about", &p.about)] {
        if s.trim().is_empty() {
            bad(at, "can't be empty".into());
        }
    }

    // Settings
    let mut keys = HashSet::new();
    for (i, s) in m.settings.iter().enumerate() {
        let at = format!("setting[{i}] ({})", s.key);
        if !keys.insert(s.key.as_str()) {
            bad(&at, "the key is used twice".into());
        }
        check_field(&at, s, true, &mut bad);
    }
    let setting = |k: &str| m.settings.iter().find(|s| s.key == k);

    // Setup
    if let Some(s) = &m.setup {
        check_runnable("setup", s.command.as_deref(), s.script.as_deref(), s.interpreter, &mut bad);
        let templates = s.command.iter().flatten().chain(s.args.iter()).map(|t| (t, false)).chain(s.env.values().map(|t| (t, true)));
        for (t, in_env) in templates {
            for v in t.vars() {
                match v {
                    Var::Input(_) | Var::Server(_) => bad("setup", format!("{v} isn't known during setup")),
                    Var::Inventory | Var::SshConfig | Var::Results => bad("setup", format!("{v}: setup runs for no servers")),
                    Var::Setting(k) => check_setting_ref("setup", k, setting(k), &mut bad),
                    Var::Secret(k) => check_secret_ref("setup", k, in_env, setting(k), &mut bad),
                    Var::PluginDir | Var::DataDir => {}
                }
            }
        }
        for k in s.env.keys() {
            if !is_env_name(k) {
                bad("setup.env", format!("\"{k}\" isn't an environment variable name"));
            }
        }
        if s.timeout.0 > MAX_TIMEOUT_SECS {
            bad("setup.timeout", "at most 6h".into());
        }
    }

    // Actions
    let mut ids = HashSet::new();
    for (i, a) in m.actions.iter().enumerate() {
        let at = format!("action[{i}] ({})", a.id);
        if !is_id(&a.id) {
            bad(&at, "the id: use 2–40 of a-z, 0-9 and -".into());
        }
        if !ids.insert(a.id.as_str()) {
            bad(&at, "the id is used twice".into());
        }
        check_runnable(&at, a.command.as_deref(), a.script.as_deref(), a.interpreter, &mut bad);
        if a.root && a.on == Runs::Local {
            bad(&at, "root: only for actions on a server; Crow never runs plugins as root here".into());
        }
        if a.login_check && a.effect == Effect::Read {
            bad(&at, "login_check: only for actions that change something".into());
        }
        if a.keyword.is_some() && a.effect != Effect::Danger {
            bad(&at, "keyword: only for effect = \"danger\"".into());
        }
        if let Some(k) = &a.keyword {
            if !(3..=20).contains(&k.len()) || !k.chars().all(|c| c.is_ascii_uppercase()) {
                bad(&at, format!("keyword \"{k}\": 3–20 capital letters A–Z"));
            }
        }
        if a.timeout.0 == 0 || a.timeout.0 > MAX_TIMEOUT_SECS {
            bad(&at, "timeout: between 1s and 6h".into());
        }
        let mut input_keys = HashSet::new();
        for (j, f) in a.inputs.iter().enumerate() {
            let at = format!("{at} input[{j}] ({})", f.key);
            if !input_keys.insert(f.key.as_str()) {
                bad(&at, "the key is used twice".into());
            }
            if f.kind == FieldKind::Secret {
                bad(&at, "an input can't be a secret: make it a setting".into());
            }
            check_field(&at, f, false, &mut bad);
        }
        for k in a.env.keys() {
            if !is_env_name(k) {
                bad(&at, format!("env \"{k}\" isn't an environment variable name"));
            }
        }
        for (t, in_env) in a.templates() {
            for v in t.vars() {
                match v {
                    Var::Input(k) if !input_keys.contains(k.as_str()) => bad(&at, format!("{v}: no input with that key")),
                    Var::Setting(k) => check_setting_ref(&at, k, setting(k), &mut bad),
                    Var::Secret(k) => check_secret_ref(&at, k, in_env, setting(k), &mut bad),
                    Var::Server(_) if a.on == Runs::Local => bad(&at, format!("{v}: a local action runs for many servers; read them from {{inventory}}")),
                    v if v.is_local_only() && a.on == Runs::Server => bad(&at, format!("{v}: only on this machine (on = \"local\")")),
                    Var::Results if !a.output.results => bad(&at, "{results} needs output.results = true".into()),
                    _ => {}
                }
            }
        }
        if a.output.results && a.on == Runs::Server {
            bad(&at, "output.results: only for local actions".into());
        }
        if !a.output.columns.is_empty() && !matches!(a.output.format, OutputFormat::Json | OutputFormat::Table) {
            bad(&at, "output.columns: only for json or table output".into());
        }
        if a.output.rows.is_some() && a.output.format != OutputFormat::Json {
            bad(&at, "output.rows: only for json output".into());
        }
    }
    let action = |id: &str| m.actions.iter().find(|a| a.id == id);
    for (i, a) in m.actions.iter().enumerate() {
        let at = format!("action[{i}] ({})", a.id);
        if let Some(d) = &a.dry_run {
            match action(d) {
                None => bad(&at, format!("dry_run \"{d}\": no action with that id")),
                Some(_) if d == &a.id => bad(&at, "dry_run can't be the action itself".into()),
                Some(dr) if dr.effect != Effect::Read => bad(&at, format!("dry_run \"{d}\" must be a read action")),
                Some(dr) if dr.on != a.on => bad(&at, format!("dry_run \"{d}\" must run in the same place")),
                Some(_) => {}
            }
        }
    }

    // Pages
    let mut page_ids = HashSet::new();
    for (i, pg) in m.pages.iter().enumerate() {
        let at = format!("page[{i}] ({})", pg.id);
        if !is_id(&pg.id) {
            bad(&at, "the id: use 2–40 of a-z, 0-9 and -".into());
        }
        if !page_ids.insert(pg.id.as_str()) {
            bad(&at, "the id is used twice".into());
        }
        if pg.blocks.is_empty() {
            bad(&at, "a page needs at least one block".into());
        }
        for (j, b) in pg.blocks.iter().enumerate() {
            let at = format!("{at} block[{j}]");
            let reads = matches!(b.kind, BlockKind::Status | BlockKind::Kv | BlockKind::Table);
            if reads {
                if pg.place == Place::Fleet {
                    bad(&at, "status, kv and table show one server: put them on a server page".into());
                }
                match b.action.as_deref().map(|id| (id, action(id))) {
                    None => bad(&at, "needs action = \"<a read action>\"".into()),
                    Some((id, None)) => bad(&at, format!("action \"{id}\": no action with that id")),
                    Some((id, Some(a))) if a.effect != Effect::Read => bad(&at, format!("action \"{id}\" changes something; a page only shows read actions")),
                    Some((id, Some(a))) if b.kind == BlockKind::Status && a.output.format != OutputFormat::Json && !a.output.results => {
                        bad(&at, format!("action \"{id}\": a status needs json output (or results)"))
                    }
                    Some(_) => {}
                }
            } else if b.action.is_some() {
                bad(&at, "action: only for status, kv and table (use actions = [...] for buttons)".into());
            }
            if b.kind == BlockKind::Actions {
                if b.actions.is_empty() {
                    bad(&at, "actions = [...] lists the buttons".into());
                }
                for id in &b.actions {
                    if action(id).is_none() {
                        bad(&at, format!("actions: no action \"{id}\""));
                    }
                }
            } else if !b.actions.is_empty() {
                bad(&at, "actions = [...]: only for kind = \"actions\"".into());
            }
        }
    }

    // Badge
    if let Some(b) = &m.badge {
        match action(&b.action) {
            None => bad("badge", format!("action \"{}\": no action with that id", b.action)),
            Some(a) if a.effect != Effect::Read => bad("badge", format!("action \"{}\" must be a read action", b.action)),
            Some(_) => {}
        }
        if b.every.0 < 60 {
            bad("badge.every", "at least 1m".into());
        }
        if let Some(l) = &b.last_run {
            match action(l) {
                None => bad("badge.last_run", format!("\"{l}\": no action with that id")),
                Some(a) if a.effect == Effect::Read => bad("badge.last_run", format!("\"{l}\" is a read action; name the one that applies")),
                Some(_) => {}
            }
        }
    }
    out
}

fn check_runnable(at: &str, command: Option<&[Template]>, script: Option<&str>, interpreter: Option<Interpreter>, bad: &mut impl FnMut(&str, String)) {
    match (command, script) {
        (None, None) => bad(at, "needs command = [...] or script = \"...\"".into()),
        (Some(_), Some(_)) => bad(at, "command and script: pick one".into()),
        (Some(c), None) => {
            if c.is_empty() {
                bad(at, "command is empty".into());
            }
            if interpreter.is_some() {
                bad(at, "interpreter: only with script".into());
            }
        }
        (None, Some(s)) => {
            if interpreter.is_none() {
                bad(at, "script needs interpreter = \"sh\", \"bash\" or \"python3\"".into());
            }
            if s.is_empty() || s.starts_with('/') || s.split('/').any(|c| c == ".." || c.is_empty()) {
                bad(at, format!("script \"{s}\": a path inside the plugin's folder"));
            }
        }
    }
}

fn check_field(at: &str, f: &Field, is_setting: bool, bad: &mut impl FnMut(&str, String)) {
    if !is_key(&f.key) {
        bad(at, "the key: use a-z, 0-9 and _".into());
    }
    if f.label.trim().is_empty() {
        bad(at, "label can't be empty".into());
    }
    let choosing = matches!(f.kind, FieldKind::Choice | FieldKind::Choices);
    if choosing && f.choices.is_empty() {
        bad(at, "choices = [...] lists what can be picked".into());
    }
    if !choosing && !f.choices.is_empty() {
        bad(at, "choices: only for choice or choices".into());
    }
    if (f.min.is_some() || f.max.is_some()) && f.kind != FieldKind::Number {
        bad(at, "min and max: only for numbers".into());
    }
    if let (Some(lo), Some(hi)) = (f.min, f.max) {
        if lo > hi {
            bad(at, "min is above max".into());
        }
    }
    if f.kind == FieldKind::Secret && f.default.is_some() {
        bad(at, "a secret can't have a default".into());
    }
    if f.kind == FieldKind::Secret && !is_setting {
        return;
    }
    if let Some(d) = &f.default {
        let ok = match f.kind {
            FieldKind::Text => d.as_str().is_some_and(|s| s.chars().all(|c| f.charset.allows(c))),
            FieldKind::Number => d.as_float().or(d.as_integer().map(|i| i as f64)).is_some_and(|n| f.min.is_none_or(|lo| n >= lo) && f.max.is_none_or(|hi| n <= hi)),
            FieldKind::Bool => d.as_bool().is_some(),
            FieldKind::Choice => d.as_str().is_some_and(|s| f.choices.iter().any(|c| c == s)),
            FieldKind::Choices => d.as_array().is_some_and(|a| a.iter().all(|v| v.as_str().is_some_and(|s| f.choices.iter().any(|c| c == s)))),
            FieldKind::Secret => true,
        };
        if !ok {
            bad(at, "the default doesn't fit the type, charset, choices or range".into());
        }
    }
}

fn check_setting_ref(at: &str, k: &str, s: Option<&Field>, bad: &mut impl FnMut(&str, String)) {
    match s {
        None => bad(at, format!("{{setting.{k}}}: no setting with that key")),
        Some(s) if s.kind == FieldKind::Secret => bad(at, format!("{{setting.{k}}} is a secret: use {{secret.{k}}} in env")),
        Some(_) => {}
    }
}

fn check_secret_ref(at: &str, k: &str, in_env: bool, s: Option<&Field>, bad: &mut impl FnMut(&str, String)) {
    match s {
        None => bad(at, format!("{{secret.{k}}}: no setting with that key")),
        Some(s) if s.kind != FieldKind::Secret => bad(at, format!("{{secret.{k}}} isn't a secret setting: use {{setting.{k}}}")),
        Some(_) if !in_env => bad(at, format!("{{secret.{k}}}: secrets go only in env, never on the command line (where ps can read them)")),
        Some(_) => {}
    }
}

#[cfg(test)]
mod tests {
    use super::{parse, Duration, Effect, Part, Runs, Template, Var, FORMAT};

    const DEVSEC: &str = include_str!("fixtures/devsec-hardening/plugin.toml");
    const RESTIC: &str = include_str!("fixtures/restic-check/plugin.toml");

    /// A valid manifest with one server action, to break one thing at a time.
    const BASE: &str = r#"
format = 1
[plugin]
id = "demo"
name = "Demo"
version = "0.1.0"
author = "me"
about = "A demo."
crow = ">=0.8.3"

[[setting]]
key = "token"
label = "Token"
type = "secret"

[[action]]
id = "list"
label = "List"
on = "server"
effect = "read"
command = ["ls", "-l"]
"#;

    fn errors(text: &str) -> String {
        parse(text).expect_err("should be refused").to_string()
    }

    #[test]
    fn the_examples_parse() {
        let d = parse(DEVSEC).unwrap_or_else(|e| panic!("devsec: {e}"));
        assert_eq!(d.plugin.id, "devsec-hardening");
        let apply = d.actions.iter().find(|a| a.id == "apply").unwrap();
        assert_eq!((apply.on, apply.effect, apply.dry_run.as_deref(), apply.login_check), (Runs::Local, Effect::Danger, Some("plan"), true));
        assert!(d.setup.is_some() && d.badge.is_some());

        let r = parse(RESTIC).unwrap_or_else(|e| panic!("restic: {e}"));
        let backup = r.actions.iter().find(|a| a.id == "backup").unwrap();
        assert_eq!(backup.inputs[0].key, "path");
        assert!(r.actions.iter().all(|a| a.root));
    }

    #[test]
    fn the_base_parses() {
        parse(BASE).unwrap();
    }

    #[test]
    fn templates_are_one_argument_with_placeholders() {
        let t = Template::parse("--path={input.path}/x").unwrap();
        assert_eq!(t.0, vec![Part::Text("--path=".into()), Part::Var(Var::Input("path".into())), Part::Text("/x".into())]);
        assert_eq!(t.to_string(), "--path={input.path}/x");
        assert_eq!(Template::parse("a {{b}} c").unwrap().0, vec![Part::Text("a {b} c".into())]);
        assert_eq!(Template::parse("a {{b}} c").unwrap().to_string(), "a {{b}} c");
        assert!(Template::parse("{input.path").is_err());
        assert!(Template::parse("x}").is_err());
        assert!(Template::parse("{shell}").is_err());
        assert!(Template::parse("{input.Bad}").is_err());
    }

    #[test]
    fn durations() {
        assert_eq!(Duration::parse("90s"), Ok(Duration(90)));
        assert_eq!(Duration::parse("30m"), Ok(Duration(1800)));
        assert_eq!(Duration::parse("6h"), Ok(Duration(21600)));
        for bad in ["", "m", "5", "5d", "-5m", "1.5h"] {
            assert!(Duration::parse(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn toml_errors_name_the_line() {
        let e = errors(&BASE.replace("effect = \"read\"", "effect = \"read\"\nshell = true"));
        assert!(e.contains("line") && e.contains("shell"), "{e}");
        let e = errors(&BASE.replace("[\"ls\", \"-l\"]", "[\"ls\", \"{nope}\"]"));
        assert!(e.contains("line") && e.contains("unknown placeholder {nope}"), "{e}");
        let e = errors(&BASE.replace("effect = \"read\"", "effect = \"read\"\ntimeout = \"5 minutes\""));
        assert!(e.contains("line") && e.contains("isn't a duration"), "{e}");
    }

    #[test]
    fn a_newer_format_says_so() {
        let e = errors(&BASE.replace("format = 1", "format = 2\nsomething_new = true"));
        assert!(e.contains(&format!("this Crow reads format {FORMAT}")), "{e}");
    }

    #[test]
    fn refusals_name_the_part() {
        let cases: &[(&str, &str, &str)] = &[
            ("command = [\"ls\", \"-l\"]", "command = [\"ls\", \"{secret.token}\"]", "secrets go only in env"),
            ("command = [\"ls\", \"-l\"]", "command = [\"ls\", \"{input.path}\"]", "no input with that key"),
            ("command = [\"ls\", \"-l\"]", "command = [\"ls\", \"{inventory}\"]", "only on this machine"),
            ("command = [\"ls\", \"-l\"]", "command = [\"ls\", \"{setting.token}\"]", "is a secret"),
            ("command = [\"ls\", \"-l\"]", "script = \"../x.sh\"\ninterpreter = \"sh\"", "inside the plugin's folder"),
            ("command = [\"ls\", \"-l\"]", "script = \"x.sh\"", "needs interpreter"),
            ("command = [\"ls\", \"-l\"]", "", "needs command"),
            ("command = [\"ls\", \"-l\"]", "command = []", "command is empty"),
            ("effect = \"read\"", "effect = \"read\"\nkeyword = \"GO\"", "only for effect = \"danger\""),
            ("effect = \"read\"", "effect = \"danger\"\nkeyword = \"go\"", "capital letters"),
            ("effect = \"read\"", "effect = \"read\"\nlogin_check = true", "only for actions that change"),
            ("on = \"server\"", "on = \"local\"\nroot = true", "never runs plugins as root"),
            ("effect = \"read\"", "effect = \"change\"\ndry_run = \"list\"", "dry_run can't be the action itself"),
            ("effect = \"read\"", "effect = \"read\"\ntimeout = \"7h\"", "between 1s and 6h"),
            ("id = \"list\"", "id = \"List!\"", "use 2–40"),
            ("id = \"demo\"", "id = \"Demo\"", "plugin.id"),
            ("version = \"0.1.0\"", "version = \"one\"", "plugin.version"),
            ("crow = \">=0.8.3\"", "crow = \"newest\"", "plugin.crow"),
            ("type = \"secret\"", "type = \"secret\"\ndefault = \"x\"", "can't have a default"),
        ];
        for (from, to, want) in cases {
            assert!(BASE.contains(from), "base lacks {from}");
            let e = errors(&BASE.replacen(from, to, 1));
            assert!(e.contains(want), "{to:?}: wanted {want:?}, got {e}");
        }
    }

    #[test]
    fn references_between_parts_are_checked() {
        let two = format!("{BASE}\n[[action]]\nid = \"wipe\"\nlabel = \"Wipe\"\non = \"server\"\neffect = \"danger\"\ncommand = [\"true\"]\n");
        parse(&two).unwrap();
        let e = errors(&two.replace("effect = \"danger\"", "effect = \"danger\"\ndry_run = \"wipe2\""));
        assert!(e.contains("no action with that id"), "{e}");
        let e = errors(&format!("{two}\n[badge]\naction = \"wipe\"\nevery = \"1h\"\n"));
        assert!(e.contains("must be a read action"), "{e}");
        let e = errors(&format!("{two}\n[badge]\naction = \"list\"\nevery = \"10s\"\n"));
        assert!(e.contains("at least 1m"), "{e}");
        let e = errors(&format!("{two}\n[[page]]\nid = \"p1\"\nlabel = \"P\"\nwhere = \"server\"\n[[page.block]]\nkind = \"table\"\naction = \"wipe\"\n"));
        assert!(e.contains("a page only shows read actions"), "{e}");
        let e = errors(&format!("{two}\n[[page]]\nid = \"p1\"\nlabel = \"P\"\nwhere = \"fleet\"\n[[page.block]]\nkind = \"table\"\naction = \"list\"\n"));
        assert!(e.contains("put them on a server page"), "{e}");
        let e = errors(&format!("{two}\n[[page]]\nid = \"p1\"\nlabel = \"P\"\nwhere = \"server\"\n[[page.block]]\nkind = \"actions\"\nactions = [\"nope\"]\n"));
        assert!(e.contains("no action \"nope\""), "{e}");
        let e = errors(&format!("{two}\n[[action]]\nid = \"list\"\nlabel = \"Again\"\non = \"server\"\neffect = \"read\"\ncommand = [\"true\"]\n"));
        assert!(e.contains("the id is used twice"), "{e}");
    }

    #[test]
    fn inputs_and_defaults_fit_their_type() {
        let with_input = |input: &str| format!("{}\n[[action.input]]\n{input}\n", BASE.replace("[\"ls\", \"-l\"]", "[\"ls\", \"{input.path}\"]"));
        parse(&with_input("key = \"path\"\nlabel = \"Path\"\ntype = \"text\"\ncharset = \"path\"\ndefault = \"/etc\"")).unwrap();
        let e = errors(&with_input("key = \"path\"\nlabel = \"Path\"\ntype = \"text\"\ndefault = \"/etc\""));
        assert!(e.contains("doesn't fit"), "a name can't hold / : {e}");
        let e = errors(&with_input("key = \"path\"\nlabel = \"Path\"\ntype = \"choice\""));
        assert!(e.contains("lists what can be picked"), "{e}");
        let e = errors(&with_input("key = \"path\"\nlabel = \"Path\"\ntype = \"number\"\nmin = 5\nmax = 1"));
        assert!(e.contains("min is above max"), "{e}");
        let e = errors(&with_input("key = \"path\"\nlabel = \"Path\"\ntype = \"secret\""));
        assert!(e.contains("can't be a secret"), "{e}");
    }
}
