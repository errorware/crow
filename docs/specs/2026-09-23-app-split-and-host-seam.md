# app.rs split + Host seam (pre-0.1.1 refactor)

Goal: break up `src/app.rs` (4.4k lines, ~150-field `CrowApp`) and route all
remote-capable host access through one seam, so 0.1.1 can add SSH by adding one
`Host` impl and deleting sample data. Behavior-preserving except where noted.

## Phase 1 — feature state structs (no behavior change)

- `src/app/` replaces `src/app.rs`:
  - `mod.rs` — slim `CrowApp` shell + `new()`
  - `poll.rs` — `BackgroundPollRequest/Result`, `run_background_poll`, apply
  - `keys.rs` — keyboard dispatcher (same priority order as today)
  - `render.rs` — screen composition
  - `<feature>.rs` — thin `impl CrowApp` handlers that need `cx`
- Each feature owns `views/<feature>/state.rs`: a struct, `load(..)`, and
  GPUI-free methods. Features: lock, setup, onboard, fleet, overview, journal,
  configs, files, danger, keys, users, firewall, lab, clankers, settings.
- Shared: `FleetState` (servers, tabs, active tab, metrics, local distro) with
  a single `active_server()`; `TextCaret` (blink, cursor, selection, anchor).
- Views take the feature state(s) they read, not `&CrowApp`.
- Cross-feature links become explicit calls (service actions →
  `journal.push_marker`; firewall reads `&configs`; keys/onboard read-only refs).
- Each feature state gets `on_key(&mut self, ev, caret) -> bool`.

## Phase 2 — Host seam

- `src/host/`: `trait Host: Send + Sync` with `exec(argv, timeout)`,
  `read_file`, `list_dir`, `write_file_atomic`, `create_dir`, `remove`;
  `HostError { Unreachable, Timeout, Failed{status, stderr}, Io }`.
- `host_for(&ServerRecord) -> Arc<dyn Host>` replaces every `is_localhost` copy.
- Impls: `LocalHost` (Command/std::fs), `ContainerHost{engine,name}`
  (podman/docker exec), `SampleHost` (canned raw command output).
- Collectors split into pure `parse_*(text)` + `collect_*(host)`.
  SampleHost's canned outputs double as parser test fixtures and survive 0.1.1.
- Stays local (not Host): lab engine management, keys, clipboard, xdg-open,
  vault, local OS detection.
- No streaming API until something tails (`exec_stream` is additive later).

### Deliberate behavior change

Config discovery and save go through `host_for(active_server)`. Today they
always target this machine (crawled once at startup, `std::fs::write`), with a
silent `/tmp/crow-config/` fallback that reports success. Phase 2 removes the
fallback (write failures surface) and sample hosts refuse writes.

## Deferred to 0.1.1

`SshHost`, privileged exec (sudo), streaming journal tail, wiring unused
crow-config parsers (hosts, sshd, ufw), deleting `SampleHost`.

## Verification

`cargo build` warning-free and `cargo test` green after every feature move.
