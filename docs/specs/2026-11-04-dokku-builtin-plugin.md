# Dokku Built-in Plugin Specification (`ERR-203` .. `ERR-210`)

> [!IMPORTANT]
> **Product Thesis Alignment:** A developer attaches a fresh Ubuntu/Debian box to Crow and wants to take a full app (e.g., a Rails app with `web` + `worker` processes, a primary PostgreSQL database, a Redis/Valkey queue datastore, `RAILS_MASTER_KEY` and env vars, custom domains, Let's Encrypt SSL, and `git push` deploy keys) from zero to production without guessing CLI flags or risking silent downtime.
>
> Consistent with `CROW.md`: **make irreversible things reversible, and make opaque things legible.** Why is Dokku a **built-in plugin** (`src/plugins.rs`, like `containers`) rather than a folder recipe plugin? Because a multi-tab app inspector, staged masked environment variable diffing, live deployment streaming, Key Hub deploy-key bridging, Containers screen cross-linking, and Topology graph edges require rich, stateful GPUI surfaces that static `kv`/`table` blocks cannot express.

---

## 1. End-to-End Operator Journey ("Fresh Box to Production Rails App")

1. **Enable Dokku Plugin** (`Settings → Plugins` → switch on `Dokku`).
2. **Bootstrap Server** (`Server → Dokku` → 1-click install Dokku + official service plugins + authorize SSH deploy keys from Crow's Key Hub).
3. **Provision Backing Stores** (Create `postgres` primary DB + `redis`/`valkey` queue store).
4. **Create & Wire App** (Create app, link DB & queue stores with explicit blast-radius preview of injected `DATABASE_URL` / `REDIS_URL`).
5. **Stage Environment** (Set `RAILS_MASTER_KEY`, `SECRET_KEY_BASE`, or paste `.env` block; apply batched with `--no-restart` or single restart).
6. **Expose & Secure** (Configure vhost domains with inline DNS verification, proxy port mappings, and 1-click Let's Encrypt TLS + auto-renew cron).
7. **Deploy & Operate** (Copy `git remote add dokku ...` or deploy via `git:from-image`, scale `web=2 worker=1`, toggle maintenance mode, tail logs, or launch an interactive `dokku enter` console).

---

## 2. Built-in Plugin Registration & Architecture

### 2.1 `src/plugins.rs` & `src/components/sidebar.rs`
- **Built-in ID:** `"dokku"`
- **Category:** `"provider.containers"`
- **Capabilities:** `&["dokku.apps", "dokku.services", "dokku.config", "dokku.domains", "dokku.certs", "dokku.keys", "dokku.deploy"]`
- **Sidebar Integration:**
  - Adds `NavItemDef { icon: TablerIcon::Rocket, label: t!("sidebar-dokku"), view_id: Some("dokku"), hint: "" }` to `src/components/sidebar.rs`.
  - Registered in `PLUGIN_PAGES`: `("dokku", "dokku")`. Hidden completely when the plugin is switched off.
  - **Live Nav Badge:** Shows `<running>/<total>` apps (colored `OK` when all healthy, `WARN` if any app is in maintenance or stopped, `CRIT` if any app has crashed/unhealthy processes or expired SSL).

### 2.2 Module Layout
- `src/dokku/mod.rs` — Data model (`Scan`, `DokkuApp`, `BackingService`, `ServicePlugin`, `EnvVar`, `DomainReport`, `CertStatus`, `DeployKey`), batched single-round-trip `SCAN_SCRIPT` + parser, and command builders.
- `src/app/dokku.rs` — `DokkuState`, background polling (`dokku_tick`), drawer/modal state, staged env diff state, streamed install/deploy tasks, and `run_host_action` mutations.
- `src/views/dokku/mod.rs` — Main screen (Server status bar, Backing Services strip, Apps table, App Inspector drawer with tabs: **Overview & Scale**, **Env & Secrets**, **Domains & TLS**, **Services & Storage**, **Logs**, **Deploy & Git**).

---

## 3. Single-Round-Trip Probe (`dokku::SCAN_SCRIPT`)

Like `containers::SCAN_SCRIPT`, Crow reads the entire Dokku state of a server in **one SSH round trip** (excluding secret env values, which are fetched only when the operator opens an app's **Env & Secrets** tab):

1. **Installation & Global State:**
   - Checks `command -v dokku` and `dokku version`.
   - If missing: checks OS (`debian`/`ubuntu`), total RAM, and Docker status so Crow can offer a 1-click **BOOTSTRAP DOKKU** flow.
   - Reads global domain (`dokku domains:report --global`), Let's Encrypt global email + cron status (`dokku letsencrypt:list`), and authorized deploy keys (`dokku ssh-keys:list`).
2. **Installed Service Plugins & Instances:**
   - Parses `dokku plugin:list` to detect official plugins (`postgres`, `redis`, `valkey`, `mysql`, `mariadb`, `mongo`, `rabbitmq`, `elasticsearch`, `meilisearch`, `clickhouse`, `letsencrypt`, `maintenance`).
   - For each installed service plugin `<svc>`, runs `dokku <svc>:list` and `dokku <svc>:links <name>` to know every backing store, its status, container state, exposed ports, and which apps it is linked to.
3. **Apps, Procfile Scale, Domains, Ports, Maintenance & Git:**
   - Lists apps via `dokku apps:list`.
   - Reads per-app reports (`ps:report`, `ps:scale`, `domains:report`, `ports:report`, `certs:report`, `git:report`, `maintenance:report`, `storage:list`) in a single shell loop framed with `@@app <name>` markers.
   - Correlates with `containers::Scan` by matching container names (`<app>.<proc>.<n>`) or labels (`com.dokku.app-name`, `org.label-schema.nyae.dokku.app-name`) to attach live CPU/Memory/Network `Usage` (`ERR-200`) to each Dokku app and process type.

---

## 4. Ticket Breakdown (`ERR-203` – `ERR-210`)

### Epic `ERR-203`: Tight Dokku Integration (Built-in Plugin)
**Summary:** Track the end-to-end built-in Dokku PaaS integration across 7 focused stages (`ERR-204`..`ERR-210`), taking a server from bare OS to a full production app with backing databases, queues, secrets, domains, TLS, and topology visibility.

---

### Stage 1 — `ERR-204`: Built-in `dokku` Plugin, Single-Round-Trip Scanner & Server Page
**Goal:** Add the `dokku` built-in plugin, probe servers in one SSH round trip, and render the read-only Server → Dokku page.

**Scope:**
- **Plugin Registration (`src/plugins.rs`, `src/views/settings/plugins.rs`, `src/components/sidebar.rs`):**
  - Add `BuiltinPlugin { id: "dokku", name: "Dokku", category: "provider.containers", capabilities: &["dokku.apps", "dokku.services", "dokku.config", "dokku.domains", "dokku.certs", "dokku.deploy"] }`.
  - Add `Dokku` (`TablerIcon::Rocket`) to the Server sidebar when enabled, with live app health badge.
- **Collector (`src/dokku/mod.rs`):**
  - Implement `SCAN_SCRIPT` and pure `parse_scan(stdout: &str) -> Scan` unit-tested against real Dokku output fixtures.
  - Capture:
    - Dokku version, global vhost domain, installed plugins (`plugin:list`).
    - Apps (`DokkuApp`): deploy source (git rev, branch, last deploy timestamp, or docker image), running state, process types & scale (`web: 2/2`, `worker: 1/1`), domains, port mappings (`http:80:5000`), SSL/Let's Encrypt expiry, maintenance mode state, mounted host storage paths, and linked services.
    - Backing Services (`BackingService`): kind (`postgres`, `redis`, `valkey`, `mysql`, `mongo`, `rabbitmq`), name, version/image, status, linked apps.
- **UI (`src/views/dokku/mod.rs`, `src/app/dokku.rs`):**
  - Top strip: Dokku version, global domain, installed plugin chips, and `REFRESH`.
  - **Backing Services section:** Cards/rows for every Postgres, Redis, etc. instance showing status, linked apps, and resource usage.
  - **Apps table:** Name, State/Processes (`web 2 · worker 1`), CPU/Mem summed from container stats, Domains, SSL badge, Linked Services (`pg:main`, `redis:queue`), Last Deploy, and row click to open the App Inspector drawer.
  - **Inspector Drawer (Logs & Overview):** Live log tailing (`dokku logs <app> --num 200`, polled every 3s when followed, with process filter e.g. `web` vs `worker`) + AI Wand (`explain_dokku_app` in `src/ai.rs` using app processes, linked services, domains, and recent logs).

---

### Stage 2 — `ERR-205`: Dokku Server Bootstrap, SSH Deploy Keys & Global Setup
**Goal:** When a user opens the Dokku page on a fresh server (or wants to configure global Dokku settings), let them bootstrap Dokku, install official service plugins, set the global domain, and authorize SSH deploy keys from Crow's Key Hub in one click.

---

### Stage 3 — `ERR-206`: App Lifecycle, Process Scaling, Maintenance Mode & Deployments
**Goal:** Create/destroy apps, scale Procfile process types (`web`, `worker`, `release`), toggle maintenance mode, and deploy/rebuild/rollback.

---

### Stage 4 — `ERR-207`: Backing Services Lifecycle (Databases, Queues, Links & Backups)
**Goal:** Provision and manage PostgreSQL, Redis, Valkey, MySQL, Mongo, and RabbitMQ services, link/unlink them to apps (showing what env vars get injected), and run one-click service exports/dumps.

---

### Stage 5 — `ERR-208`: Staged Environment & Secrets Manager (`dokku config`)
**Goal:** Give operators a safe, legible, secret-aware environment variable editor for each app, with batch staging, `.env` paste/import, and explicit restart control.

---

### Stage 6 — `ERR-209`: Domains, Proxy Ports & Let's Encrypt TLS
**Goal:** Manage virtual host domains, proxy port mappings (`http:80:5000`), and Let's Encrypt certificates per app.

---

### Stage 7 — `ERR-210`: Cross-Surface Integration (Topology Map, Containers Cross-Linking & Checks)
**Goal:** Make Dokku apps first-class citizens across Crow's Containers page, Topology & DR map, and External Checks.

**Scope:**
- **Containers Page Cross-Linking (`src/containers/mod.rs`, `src/views/containers.rs`):**
  - Detect Dokku containers via `com.dokku.app-name` / `org.label-schema.nyae.dokku.app-name` labels or `dokku.<service>.<name>` naming.
  - Group Dokku app containers and backing services cleanly on the Containers page (just like Compose stacks are grouped), with a `DOKKU` badge and an `OPEN IN DOKKU →` button that jumps straight to that app or service in the Dokku screen.
- **Topology & DR Map (`src/topology/mod.rs`):**
  - Represent Dokku apps and backing services on the Fleet Map with `EdgeKind::Runs` from the server and service-link edges between a Dokku app and its linked Postgres/Redis workloads.
  - **Posture & Workload Risks:** Flag Dokku-specific risks on the map:
    - Production app serving public domain over HTTP only (no TLS cert).
    - Let's Encrypt certificate expiring in < 14 days or auto-renew cron missing.
    - App in Maintenance Mode or with crashed/0-scaled `web` processes.
    - Unlinked backing service running and consuming memory with no attached apps.
- **1-Click External Uptime Check (`src/checks.rs`):**
  - On any Dokku app domain in **Domains & TLS**, a `WATCH IN CHECKS` button creates an HTTP(S) check (`ERR-148`) attached to this server for `https://<domain>` so Crow monitors its external reachability and TLS expiry automatically.
- **Translations & Polish:**
  - Full Fluent keys in `assets/i18n/en.ftl`, `languages/es.ftl`, and `languages/ja.ftl` verified via `crow --check-language`.

**Verification:**
- Unit tests for Dokku container label detection, topology graph node/edge/risk generation, and `crow --check-language` passing for `es.ftl` and `ja.ftl`.
