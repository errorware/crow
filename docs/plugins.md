# Writing a Crow plugin

> **Draft.** The plugin system is being built in stages ([ERR-173](https://linear.app/errorware/issue/ERR-173)). This page describes the format as it is designed. Until the stages that load and run plugins land, Crow reads none of this.

A Crow plugin is a folder: a `plugin.toml` that says what the plugin can do, plus the scripts it uses. There is no compiled code. Crow runs the plugin's commands over its own SSH connections, asks before anything changes, records every run, and shows the results with the same tables, badges and consoles as its built-in pages.

```
~/.config/crow/plugins/
└── restic-check/
    ├── plugin.toml
    ├── status.sh
    └── i18n/          (optional: the plugin's own translations)
        └── es.ftl
```

The folder name is the plugin's `id`.

## How Crow keeps a plugin honest

A plugin can run commands as root on every server you have, so Crow never runs one you haven't seen:

- **Review before use.** A new plugin shows every action it declares: the exact command, where it runs, whether it needs root, and whether it reads, changes or is dangerous. Nothing runs until you accept it.
- **Pinned.** Accepting records a hash of the whole folder. If any file changes, the plugin is switched off until you review it again.
- **No shell text.** A command is a list of arguments, not a line for a shell. A value you type into an input becomes part of one argument; it can't add another argument or a second command.
- **Secrets stay out of sight.** A secret setting reaches a command only as an environment variable, never as an argument (where `ps` can read it), and never appears in run history or logs.
- **Everything is recorded.** Each run goes into the plugin's run history and Crow's audit log.

`crow --check-plugin <folder>` runs the same checks Crow does, so you can test a plugin before sharing it.

## plugin.toml

```toml
format = 1

[plugin]
id = "restic-check"
name = "Restic backups"
version = "0.1.0"
author = "Your name"
about = "Lists restic snapshots on each server and flags old backups."
crow = ">=0.8.3"
license = "MIT"                # optional
homepage = "https://…"         # optional
category = "backup"            # optional
capabilities = ["backup.list"] # optional
```

| Key | |
|---|---|
| `format` | Always `1` for now. A Crow that reads an older format says so instead of guessing. |
| `id` | 2–40 characters of `a-z`, `0-9` and `-`. Must match the folder name. |
| `version` | Semantic version: `1.2.0`. |
| `crow` | Which Crow versions it works with, as a requirement: `>=0.8.3`, `^0.8`. |

### requires

```toml
[requires]
local = ["python3"]          # commands on this machine
server = ["restic"]          # commands on each server
os = ["debian", "ubuntu"]    # distro families; leave out for any
```

Crow checks these before running anything and names what's missing.

### setup

A one-time install on this machine, such as a Python venv. It's shown as INSTALL on the plugin's row, streamed in a console. The plugin's actions stay unavailable until it succeeds. It may write only inside the plugin's data folder, `{data_dir}`.

```toml
[setup]
script = "setup.sh"
interpreter = "sh"
env = { ANSIBLE_CORE = "2.17.4", VENV = "{data_dir}/venv" }
timeout = "15m"
```

### settings

Shown as a form under the plugin in Settings → Plugins.

```toml
[[setting]]
key = "repository"
label = "Repository"
type = "text"
charset = "path"
required = true
help = "As restic takes it in RESTIC_REPOSITORY."

[[setting]]
key = "password"
label = "Repository password"
type = "secret"
```

| Key | |
|---|---|
| `key` | `a-z`, `0-9` and `_`. |
| `type` | `text`, `number`, `bool`, `choice` (one of `choices`), `choices` (several of them), or `secret` (kept in Crow's vault). |
| `charset` | For text: `name` (letters, digits, `.` `_` `-`; the default), `path` (a name plus `/ : @ ~ + = ,`) or `any` (any printable text). Anything else is refused before it reaches a command. |
| `default`, `choices`, `min`, `max`, `required`, `help` | As you'd expect. A secret can't have a default. |

Plain settings are kept in Crow's vault as you enter them; secrets are sealed with the vault's key, the same as provider tokens, and are never shown again (an empty box keeps the saved one). Whatever a run prints is cleared of secret values (shown as •••) before it reaches the console, the run history or the audit log.

### actions

An action is one thing the plugin can do.

```toml
[[action]]
id = "backup"
label = "Back up now"
help = "Runs restic backup on the path you give."
on = "server"          # or "local"
effect = "change"      # "read", "change" or "danger"
root = true            # server only: through sudo -n
command = ["restic", "backup", "{input.path}"]
env = { RESTIC_REPOSITORY = "{setting.repository}", RESTIC_PASSWORD = "{secret.password}" }
timeout = "2h"         # 90s, 30m, 6h; at most 6h; default 5m

[[action.input]]
key = "path"
label = "Path to back up"
type = "text"
charset = "path"
default = "/etc"

[action.output]
format = "lines"
summary_prefix = "snapshot "
```

**Where it runs (`on`)**

- `server`: over SSH on each server it's run for, as Crow's own login user, through `sudo -n` when `root = true`.
- `local`: on this machine, once for all the servers it's run for. Crow writes those servers into `{inventory}` (an Ansible-style inventory) and `{ssh_config}`, with Crow's key, login user, port, host-key checking and bastions. Your program gets the paths, never the key itself. Local actions never run as root.

**What it does (`effect`)**

- `read` changes nothing and runs without asking.
- `change` asks first.
- `danger` (anything that could cut off access or lose data) needs a typed keyword, `keyword = "HARDEN"` (default `CONFIRM`). The English is shown under a translation.

For `change` and `danger`:

- `dry_run = "<a read action>"` runs that action first and shows its output before you confirm.
- `login_check = true` makes Crow prove a fresh key login before and after. If the login after fails, the run stops and tells you how to get back in.

**How it runs (`command` or `script`)**

- `command` is an argument list, run directly with no shell in between.
- `script = "status.sh"` with `interpreter = "sh" | "bash" | "python3"` runs a file from the plugin's folder. Its arguments go in `args = [...]`. On a server the script is sent on stdin, so nothing is left behind.

If you need a pipe, put it in a script and pass values as arguments or environment variables. `sh -c` with a placeholder inside its text is refused.

**Placeholders** fill in part of one argument or env value. Write `{{` and `}}` for literal braces.

| Placeholder | |
|---|---|
| `{input.KEY}` | An input of this action, typed when it's run. |
| `{setting.KEY}` | A setting. `choices` become a comma-separated list. |
| `{secret.KEY}` | A secret setting. In `env` only. |
| `{server.name}` `{server.host}` `{server.user}` `{server.port}` | Server actions only. |
| `{plugin_dir}` `{data_dir}` | Local actions and setup: the plugin's folder (read-only), and its own data folder. |
| `{inventory}` `{ssh_config}` | Local actions: the servers it's run for. |
| `{results}` | Local actions with `output.results = true`: a file to write per-server results to. |

**Reading the output (`[action.output]`)**

| Key | |
|---|---|
| `format` | `lines` (shown as a console; the default), `json`, `table` (whitespace columns, first line the header) or `exit` (only the exit code matters). |
| `rows` | For json: the dotted key holding the list of rows, when it isn't the top level. |
| `columns` | For json and table: `[{ key = "time", label = "Time" }, …]`, in order. |
| `summary_prefix` | The first line starting with this becomes the run's one-line summary. |
| `results` | Local actions: the program writes one JSON line per server to `{results}`: `{"host": "web-1", "ok": true, "changed": 3, "summary": "…", "state": "ok", "label": "…"}`. Stdout stays a console for people. |

A **status** (for a status block or the badge) is JSON with `state` (`ok`, `warn`, `bad` or `unknown`) and a short `label`. A server action prints it. A local action writes it per server to `{results}`.

### pages

Where the plugin shows up: in a server's sidebar (`where = "server"`) or in Fleet configuration's side menu (`where = "fleet"`). A page is a list of blocks, drawn with Crow's own components.

```toml
[[page]]
id = "backups"
label = "Backups"
icon = "database"
where = "server"

[[page.block]]
kind = "status"
action = "status"

[[page.block]]
kind = "table"
action = "snapshots"
title = "Snapshots"

[[page.block]]
kind = "actions"
actions = ["check", "backup"]

[[page.block]]
kind = "runs"
```

| Block | Shows |
|---|---|
| `status` | ok / warn / bad and a label, from a read action. Server pages only. |
| `kv` | Key and value pairs from a read action's JSON object. Server pages only. |
| `table` | Rows from a read action. Server pages only. |
| `actions` | Buttons for the listed actions. On a fleet page they run as a fleet run: pick servers or a group, a canary first, then batches, stopping at the first failure. |
| `runs` | The plugin's run history: when, where, the result and the summary. |

### badge

A status shown on the server's identity bar, in the fleet table and on the fleet map.

```toml
[badge]
action = "status"      # a read action giving a status
every = "1h"           # how often; at least 1m
last_run = "apply"     # optional: adds "· Oct 12", that action's last success
```

## Translations

A plugin can ship `i18n/<language>.ftl` for its own labels. Without it, the plugin shows its English. The details come with the stage that adds them ([ERR-182](https://linear.app/errorware/issue/ERR-182)).

## Examples

- `src/plugin_host/fixtures/restic-check/`: server actions, a secret, an input, a table, a badge.
- `src/plugin_host/fixtures/devsec-hardening/`: a local Ansible action with setup, a dry run, a danger action with a login check, and per-server results ([ERR-170](https://linear.app/errorware/issue/ERR-170)).
