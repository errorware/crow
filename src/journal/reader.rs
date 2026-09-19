use std::process::Command;
use crate::vault::ServerRecord;
use super::{parse_journal_json, JournalEntry, JournalPriority, JournalQuery};

pub struct LocalJournalReader;

impl LocalJournalReader {
    /// Runs a full journal lookup against the local host using `journalctl -o json`.
    /// Returns `None` only on a hard failure (journalctl missing / failed to spawn) —
    /// a query that legitimately matches nothing still returns `Some(vec![])`, so
    /// callers must not treat "no matches" as a reason to fall back to demo data.
    pub fn read_query(query: &JournalQuery) -> Option<Vec<JournalEntry>> {
        let mut cmd = Command::new("journalctl");
        cmd.args(["-o", "json", "-n", &query.limit.to_string(), "--no-pager"]);
        cmd.args(["-b", &query.boot.offset().to_string()]);

        if let Some(ref u) = query.unit {
            if !u.is_empty() && u != "ALL" && u != "ALL UNITS" {
                cmd.args(["-u", u]);
            }
        }

        if let Some(p) = query.priority {
            cmd.args(["-p", &(p as u8).to_string()]);
        }

        if let Some(pid) = query.pid {
            cmd.arg(format!("_PID={}", pid));
        }

        if let Some(since) = query.time_range.since_str() {
            cmd.args(["--since", since]);
        }

        if let Some(ref pattern) = query.grep {
            if !pattern.trim().is_empty() {
                cmd.args(["--grep", pattern]);
            }
        }

        let output = cmd.output().ok()?;

        let mut entries = Vec::new();
        let stdout_str = String::from_utf8_lossy(&output.stdout);

        for line in stdout_str.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Some(entry) = parse_journal_json(trimmed) {
                entries.push(entry);
            }
        }

        Some(entries)
    }
}

pub struct SimulatedJournalReader;

impl SimulatedJournalReader {
    /// Generates realistic systemd journal streams tailored to the server's role and name,
    /// then applies the same unit/priority/pid/grep narrowing a real journalctl query would.
    /// Time range and boot scope are not meaningful against a canned recent-events template
    /// and are ignored here.
    pub fn generate(server: &ServerRecord, query: &JournalQuery) -> Vec<JournalEntry> {
        let events = match server.role.to_lowercase().as_str() {
            r if r.contains("db") || r.contains("postgres") => postgres_events(&server.name),
            r if r.contains("redis") || r.contains("cache") => redis_events(&server.name),
            r if r.contains("web") || r.contains("nginx") => web_events(&server.name, &server.host),
            _ => generic_server_events(&server.name, &server.host),
        };

        let events: Vec<_> = events
            .into_iter()
            .filter(|(prio, unit, _ident, pid, msg, _cmdline, _exe)| {
                if let Some(ref u) = query.unit {
                    if !u.is_empty() && u != "ALL" && u != "ALL UNITS" && !unit.eq_ignore_ascii_case(u) {
                        return false;
                    }
                }
                if let Some(min_prio) = query.priority {
                    if *prio > min_prio {
                        return false;
                    }
                }
                if let Some(want_pid) = query.pid {
                    if *pid != want_pid {
                        return false;
                    }
                }
                if let Some(ref pattern) = query.grep {
                    if !pattern.trim().is_empty() && !msg.to_lowercase().contains(&pattern.to_lowercase()) {
                        return false;
                    }
                }
                true
            })
            .collect();

        let now_usec = chrono::Local::now().timestamp_micros() as u64;
        let mut entries = Vec::new();

        for (idx, (prio, unit, ident, pid, msg, cmdline, exe)) in events.into_iter().take(query.limit).enumerate() {
            let offset_usec = (idx as u64) * 3_500_000 + 400_000;
            let ts = now_usec.saturating_sub(offset_usec);
            let (ts_fmt, rel_fmt) = JournalEntry::format_time(ts);

            let mut fields = vec![
                ("_SYSTEMD_UNIT".to_string(), unit.to_string()),
                ("_COMM".to_string(), ident.to_string()),
                ("_PID".to_string(), pid.to_string()),
                ("_HOSTNAME".to_string(), server.name.clone()),
                ("_CMDLINE".to_string(), cmdline.to_string()),
                ("_EXE".to_string(), exe.to_string()),
                ("_UID".to_string(), "0".to_string()),
                ("_GID".to_string(), "0".to_string()),
                ("_SYSTEMD_CGROUP".to_string(), format!("/system.slice/{}", unit)),
                ("_SYSTEMD_SLICE".to_string(), "system.slice".to_string()),
                ("_BOOT_ID".to_string(), "8c5bafa6afa547d6ace60f67973a3d58".to_string()),
                ("_TRANSPORT".to_string(), "journal".to_string()),
            ];
            if prio.is_error() {
                fields.push(("ERROR_CODE".to_string(), "ERR_SUBPROCESS_FAULT".to_string()));
            }

            entries.push(JournalEntry {
                id: format!("sim_{}_{}", server.id, idx),
                cursor: Some(format!("s=sim;i={:x};b=8c5bafa6;m={:x}", idx, ts)),
                timestamp_usec: ts,
                timestamp_formatted: ts_fmt,
                time_relative: rel_fmt,
                priority: prio,
                unit: unit.to_string(),
                syslog_identifier: ident.to_string(),
                pid: Some(pid),
                message: msg.to_string(),
                fields,
                is_expanded: false,
            });
        }

        entries
    }
}

type EventTemplate = (JournalPriority, &'static str, &'static str, u32, &'static str, &'static str, &'static str);

fn web_events(_host: &str, _ip: &str) -> Vec<EventTemplate> {
    vec![
        (JournalPriority::Info, "nginx.service", "nginx", 1412, "159.223.84.17 GET /api/v2/orders 200 14ms - Mozilla/5.0", "/usr/sbin/nginx -g 'daemon on;'", "/usr/sbin/nginx"),
        (JournalPriority::Info, "crow-agent.service", "crow-agent", 20441, "metrics flush 312 series -> coordinator:443 in 12ms", "/usr/local/bin/crow-agent --config /etc/crow.toml", "/usr/local/bin/crow-agent"),
        (JournalPriority::Warning, "fail2ban.service", "fail2ban-server", 1077, "Ban 45.134.26.7 (sshd: 5 failed authentications in 60s)", "/usr/bin/fail2ban-server -xf start", "/usr/bin/fail2ban-server"),
        (JournalPriority::Info, "sshd.service", "sshd", 764, "Accepted publickey for root from 10.0.4.19 port 51244 ssh2: ED25519 SHA256:8b4f1c", "/usr/sbin/sshd -D", "/usr/sbin/sshd"),
        (JournalPriority::Info, "nginx.service", "nginx", 1412, "10.0.4.22 POST /webhooks/stripe 204 8ms TLSv1.3", "/usr/sbin/nginx -g 'daemon on;'", "/usr/sbin/nginx"),
        (JournalPriority::Warning, "ufw.service", "kernel", 0, "[UFW BLOCK] IN=eth0 OUT= MAC=52:54:00:12:34:56 SRC=185.220.101.4 DST=159.223.84.17 PROTO=TCP SPT=49122 DPT=23 [SYN]", "kernel", "/boot/vmlinuz"),
        (JournalPriority::Info, "chrony.service", "chronyd", 722, "System clock synchronized: offset -0.000184s, stratum 2, peer time.cloudflare.com", "/usr/sbin/chronyd -F 1", "/usr/sbin/chronyd"),
        (JournalPriority::Err, "clamav-freshclam.service", "freshclam", 3110, "signature mirror timeout after 30s (mirror.clamav.net) - retrying in 600s", "/usr/bin/freshclam -d --foreground=true", "/usr/bin/freshclam"),
        (JournalPriority::Info, "systemd-journald.service", "systemd-journald", 411, "Journal stopped. Rotated system.journal (128M) to /var/log/journal/system@...journal", "/lib/systemd/systemd-journald", "/lib/systemd/systemd-journald"),
        (JournalPriority::Info, "nginx.service", "nginx", 1412, "159.223.84.17 GET /healthz 200 1ms HTTP/2.0", "/usr/sbin/nginx -g 'daemon on;'", "/usr/sbin/nginx"),
        (JournalPriority::Warning, "systemd.service", "systemd", 1, "unattended-upgrades.service holding dpkg transaction lock (3 packages pending)", "/sbin/init", "/sbin/init"),
        (JournalPriority::Info, "docker.service", "dockerd", 993, "Container web.2 healthcheck succeeded (HTTP 200 after 4ms)", "/usr/bin/dockerd -H fd://", "/usr/bin/dockerd"),
    ]
}

fn postgres_events(_host: &str) -> Vec<EventTemplate> {
    vec![
        (JournalPriority::Info, "postgresql@16-main.service", "postgres", 1189, "checkpoint complete: wrote 1428 buffers (8.7%); 0 WAL file(s) added, 0 removed, 1 recycled", "/usr/lib/postgresql/16/bin/postgres -D /var/lib/postgresql/16/main", "/usr/lib/postgresql/16/bin/postgres"),
        (JournalPriority::Info, "postgresql@16-main.service", "postgres", 1189, "autovacuum: completed vacuuming \"public.events\": 482 pages, 1420 tuples", "/usr/lib/postgresql/16/bin/postgres -D /var/lib/postgresql/16/main", "/usr/lib/postgresql/16/bin/postgres"),
        (JournalPriority::Warning, "postgresql@16-main.service", "postgres", 1189, "duration: 481.120 ms  statement: SELECT count(*) FROM audit_logs WHERE created_at > NOW() - INTERVAL '7 days'", "/usr/lib/postgresql/16/bin/postgres -D /var/lib/postgresql/16/main", "/usr/lib/postgresql/16/bin/postgres"),
        (JournalPriority::Info, "postgresql@16-main.service", "postgres", 1189, "connection authorized: user=app_prod database=crow_db application_name=sidekiq", "/usr/lib/postgresql/16/bin/postgres -D /var/lib/postgresql/16/main", "/usr/lib/postgresql/16/bin/postgres"),
        (JournalPriority::Info, "crow-agent.service", "crow-agent", 20441, "schema pack postgres-16 verified: pg_hba.conf matches baseline checksum", "/usr/local/bin/crow-agent", "/usr/local/bin/crow-agent"),
        (JournalPriority::Warning, "kernel", "kernel", 0, "TCP: request_sock_TCP: Possible SYN flooding on port 5432. Sending cookies.", "kernel", "/boot/vmlinuz"),
        (JournalPriority::Info, "sshd.service", "sshd", 764, "Accepted publickey for postgres from 10.0.4.12 port 42890 (replication streaming)", "/usr/sbin/sshd -D", "/usr/sbin/sshd"),
        (JournalPriority::Err, "systemd.service", "systemd", 1, "wal-g-backup.timer: Job wal-g-backup.service/start failed with result 'exit-code'", "/sbin/init", "/sbin/init"),
        (JournalPriority::Info, "chrony.service", "chronyd", 722, "System clock synchronized: offset +0.000041s", "/usr/sbin/chronyd -F 1", "/usr/sbin/chronyd"),
    ]
}

fn redis_events(_host: &str) -> Vec<EventTemplate> {
    vec![
        (JournalPriority::Info, "redis-server.service", "redis-server", 1902, "DB 0: 428,912 keys (0 volatile) in 524288 slots. 0 stats.", "/usr/bin/redis-server 127.0.0.1:6379", "/usr/bin/redis-server"),
        (JournalPriority::Warning, "redis-server.service", "redis-server", 1902, "latency spike: 84ms on BLPOP queue:default from client 10.0.4.31:49182", "/usr/bin/redis-server 127.0.0.1:6379", "/usr/bin/redis-server"),
        (JournalPriority::Err, "redis-server.service", "redis-server", 1902, "MISCONF Redis is configured to save RDB snapshots, but is currently not able to persist to disk (errno 28)", "/usr/bin/redis-server 127.0.0.1:6379", "/usr/bin/redis-server"),
        (JournalPriority::Info, "crow-agent.service", "crow-agent", 20441, "alert triggered: redis_disk_persistence_failure on /var/lib/redis", "/usr/local/bin/crow-agent", "/usr/local/bin/crow-agent"),
        (JournalPriority::Info, "redis-server.service", "redis-server", 1902, "Accepted 10.0.4.32:51294 (client id 91841)", "/usr/bin/redis-server 127.0.0.1:6379", "/usr/bin/redis-server"),
    ]
}

fn generic_server_events(_host: &str, _ip: &str) -> Vec<EventTemplate> {
    vec![
        (JournalPriority::Info, "systemd.service", "systemd", 1, "Started Crow Node Management Agent.", "/sbin/init", "/sbin/init"),
        (JournalPriority::Info, "crow-agent.service", "crow-agent", 20441, "telemetry stream connected -> tls://coordinator.crow.internal:443", "/usr/local/bin/crow-agent", "/usr/local/bin/crow-agent"),
        (JournalPriority::Info, "sshd.service", "sshd", 764, "Server listening on 0.0.0.0 port 22.", "/usr/sbin/sshd -D", "/usr/sbin/sshd"),
        (JournalPriority::Info, "chrony.service", "chronyd", 722, "Selected source 162.159.200.123 (time.cloudflare.com)", "/usr/sbin/chronyd -F 1", "/usr/sbin/chronyd"),
        (JournalPriority::Warning, "kernel", "kernel", 0, "device eth0 entered promiscuous mode", "kernel", "/boot/vmlinuz"),
        (JournalPriority::Info, "cron.service", "cron", 812, "(root) CMD (/usr/local/bin/crow-telemetry-sync >/dev/null 2>&1)", "/usr/sbin/cron -f", "/usr/sbin/cron"),
    ]
}

/// Dispatches journal collection depending on whether the server is local or simulated.
/// A local host that genuinely has zero matches for the query returns an empty list —
/// it does NOT fall back to simulated data, which would misrepresent a real "no results"
/// as fabricated demo activity.
pub fn read_journal_for_server(server: &ServerRecord, query: &JournalQuery) -> Vec<JournalEntry> {
    let is_localhost = server.host == "127.0.0.1"
        || server.host == "localhost"
        || server.host == "::1"
        || server.name.to_lowercase() == "localhost"
        || server.tags.iter().any(|t| t == "localhost" || t == "local");

    if is_localhost {
        if let Some(entries) = LocalJournalReader::read_query(query) {
            return entries;
        }
    }

    SimulatedJournalReader::generate(server, query)
}
