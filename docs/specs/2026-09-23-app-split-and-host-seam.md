# app.rs split + Host seam (pre-0.1.1 refactor)

Goal: break up `src/app.rs` (4.4k lines, ~150-field `CrowApp`) and route all
remote-capable host access through one seam, so 0.1.1 can add SSH by adding one
`Host` impl and deleting sample data. Behavior-preserving except where noted.

## Phase 1 — feature state structs (done)

- `src/app/` replaces `src/app.rs`:
  - `mod.rs` — `CrowApp` shell (screen, chrome, caret, `FleetState`, one field
    per feature state) + `new()` built from per-feature loaders
  - `poll.rs` — background poll request/worker/apply, metrics seeding, poll task
  - `keyboard.rs` — `handle_key_down`: a short-circuit chain over one
    `keys_*` method per keyboard-owning surface (same priority order as before)
  - `render.rs` — screen composition
  - `<feature>.rs` — `impl CrowApp` handlers that need `cx` or span features
- Each feature owns its state next to its view (`views/<feature>/state.rs` or
  `*_state.rs`) with GPUI-free, unit-tested methods: Firewall, Users, Files,
  DangerZone, LocalLab, Clankers, Journal, Configs, Keys, Overview, Fleet,
  Settings, plus the shared `TextCaret`.
- Pure-state actions are called directly from view closures
  (`this.users.toggle_lock(..); cx.notify()`); `CrowApp` wrappers exist only
  where a handler spans features or spawns work.
- Views take exactly the state they read (computed transitively through helper
  calls) and keep `Entity<CrowApp>` only to dispatch events. No view takes
  `&CrowApp`.

## Phase 2 — Host seam (done)

- `src/host/`: `trait Host: Send + Sync` — `exec_stdin(argv, stdin, timeout)`
  plus `exec`, `read_file`, `exists`, `list_dir`, `write_file_atomic`,
  `create_dir`, `remove`, all with exec-based defaults so a transport only has
  to implement `exec_stdin`. `HostError { Unreachable, Timeout, Failed, Io }`.
- `host_for(&ServerRecord) -> Option<Arc<dyn Host>>` is the single routing point:
  lab test nodes (by `test-node` + engine tag) → `ContainerHost`; this machine →
  `LocalHost`; everything else → `None` until `SshHost` exists.
- **Changed from the original plan: no `SampleHost`.** The sample data is typed
  (role-shaped service lists, jittered metrics), and 0.1.1 deletes it. Rewriting
  it as canned command text only to delete it was waste. Instead each collector
  has exactly one branch: `Some(host)` → real, `None` → existing sample fallback.
  Parser tests use small real-output fixtures instead.
- Collectors are pure `parse_*` functions fed over the host: services, processes,
  sockets, metrics (one batched `/proc` probe per tick, 15s df/systemctl probe,
  rate state per server), journal (`journalctl_argv`), retention, firewall
  detection, Files listing (owners resolved from the host's own passwd/group).
- Stays local (not Host): lab engine management, keys, clipboard, xdg-open,
  vault, Crow's own config, `known_hosts`, local OS detection.
- No streaming API until something tails (`exec_stream` is additive later).

### Deliberate behavior changes

- Config discovery/save go through the configs' server. Configs record the
  server they were read from, reload in the background on tab switch (unless
  there are unsaved edits), and write back atomically only to that server.
  Revisions are recorded only after a successful write; the silent
  `/tmp/crow-config/` fallback is gone and errors show on the pending-diff rail.
- `write_blocked` refuses writes for placeholders, unreadable files,
  unconnected servers, and the pg_hba / cron / journald / ufw structured
  editors, which render Crow's sample model rather than the real file.
- Lab test nodes are read from their container (they previously matched the
  localhost check and showed this machine's data).
- systemctl actions, kill, Files create/delete and firewall flush fail with
  "not connected" on servers without a transport instead of reporting
  simulated success. Power actions remain an explicit simulation everywhere.
- A reachable host whose command fails shows an empty table / error, not demo
  data.

## 0.1.1 follow-ups

1. `SshHost` (fills `host_for`'s `None`), including privileged exec (sudo) as
   an exec option, and per-server connection state for the UI.
2. Delete the sample fallbacks (`role_fallback_*`, `SimulatedCollector`,
   `SimulatedJournalReader`, `read_simulated_retention`, `simulated_directory`,
   `detect_remote_firewall`, `sample_config_content`, crawler placeholders).
3. Structured editors parse real files (crow-config-core already has pg_hba,
   hosts, sshd_config and UFW plugins) so their files become writable; wire the
   unused hosts/sshd plugins into the config screen.
4. Batch config reads for remote hosts (one round trip instead of one per file).
5. Real firewall/user actions: those screens still only mutate in-memory
   state and show "Executed: …" toasts.
6. Streaming journal tail (`exec_stream`) if polling proves too coarse.

## Verification

`cargo build` warning-free and `cargo test` green after every commit; the app
launches and runs without panics.
