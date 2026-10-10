### Crow's English messages: the reference every language pack is checked
### against, and what shows when a pack lacks a message.
###
### Each message is `id = text`. `{ $name }` is a value Crow fills in;
### a translation must use the same ones. Plurals:
###   { $count -> [one] one server *[other] { $count } servers }

## Tests (ERR-171)
i18n-test-args = { $server }: { $count ->
    [one] { $count } server
   *[other] { $count } servers
}
i18n-test-plain = Hello


## appearance
appearance-choose-a-picture-for-the = Choose a picture for the Fleet page

## app-archive
app-archive-stored-data-purged = stored data purged
app-archive-data-purged-in-days = data purged in { $days } days
app-archive-data-purged-in-1-day = data purged in 1 day
app-archive-data-purged-today = data purged today
app-archive-purge-due = purge due
app-archive-purging-is-off-data-kept = purging is off · data kept
app-archive-never-purging-is-switched-off = never — purging is switched off
app-archive-is-the-bastion-for-move = { $name } is the bastion for { $users }: move them first, then archive it
app-archive-could-not-archive = Could not archive { $name }: { $e }
app-archive-could-not-archive-the-vault = Could not archive: the vault is unavailable
app-archive-archived-its-data-is-purged = { $name } archived · its data is purged { $purge_due_text }
app-archive-could-not-restore = Could not restore { $name }: { $e }
app-archive-restored-to-the-fleet = { $name } restored to the fleet
app-archive-could-not-purge-s-data = Could not purge { $name }'s data: { $e }

## app-bastions
app-bastions-is-still-the-bastion-for = { $name } is still the bastion for { $users }: move them first
app-bastions-is-a-bastion-servers-can = { $name } is a bastion: servers can be added through it
app-bastions-is-no-longer-a-bastion = { $name } is no longer a bastion

## app-certificates
app-certificates-isnt-managed-by-certbot-or = { $cert_path } isn't managed by certbot or acme.sh on { $name }: renew it where it came from.
app-certificates-certbot-dry-run-renew-reload = certbot: dry run, renew { $name }, reload{ $inv }{ $inv2 }, confirm
app-certificates-acme-sh-renew-reload-confirm = acme.sh: renew { $domain }, reload, confirm
app-certificates-renew = RENEW { $label }

## app-checks
app-checks-http-a-word-the-page = HTTP: a word the page must contain · DNS: the address it must resolve to
app-checks-name-optional = name (optional)
app-checks-type-what-to-check-first = Type what to check first.
app-checks-an-http-check-needs-a = An HTTP check needs a URL starting with http:// or https://.
app-checks-a-tcp-check-is-host = A TCP check is host:port, like db.lan:5432.
app-checks-added-checked-every-s-while = Added "{ $name }": checked every { $every_secs } s while Crow is open.

## app-clankers
app-clankers-set-leave-blank-to-keep = set · leave blank to keep it, or paste a new key
app-clankers-e-g-sk-proj = e.g. sk-proj-...
app-clankers-e-g-gpt-4o-mini = e.g. gpt-4o-mini
app-clankers-model-cannot-be-empty = Model cannot be empty
app-clankers-the-vault-is-busy-try = the vault is busy; try again
app-clankers-unknown-provider = unknown provider
app-clankers-no-encryption-key = no encryption key
app-clankers-not-saved = Not saved: { $e }
app-clankers-is-the-primary-clanker = { $name } is the primary clanker
app-clankers-is-the-backup-clanker = { $display_name } is the backup clanker
app-clankers-no-backup-clanker = No backup clanker
app-clankers-provider-call-stats-reset = Provider call stats reset
app-clankers-no-ai-provider-has-an = No AI provider has an API key yet. Add one above with ⚙ Edit Key.
app-clankers-via-the-backup-the-primary =
    via the backup, { $provider_label } (the primary failed: { $why })
    {""}
    { $text }
app-clankers-via =
    via { $provider_label }
    {""}
    { $text }

## app-configs
app-configs-the-fleet = the fleet
app-configs-group = group { $scope }
app-configs-type-a-key-or-pick = type a key, or pick one from the list
app-configs-a-directive-needs-its-arguments = a directive needs its arguments
app-configs-could-not-be-parsed = { $selected } could not be parsed: { $e }
app-configs-no-config-file-selected = No config file selected.
app-configs-isnt-one-of-the-files = { $path } isn't one of the files Crow manages on this server
app-configs-this-servers-configs-are-still = this server's configs are still loading; nothing was written
app-configs-is-not-loaded = { $file } is not loaded
app-configs-couldnt-re-read-it = { $file }: couldn't re-read it: { $e }
app-configs-not-saved = { $file }: not saved: { $e }
app-configs-filter-configs = Filter configs...
app-configs-reverted-firewall-rules-to-baseline = Reverted firewall rules to baseline
app-configs-the-vault-is-busy = the vault is busy
app-configs-no-snapshot-nothing-applied = No snapshot, nothing applied: { $e }
app-configs-the-snapshot-failed-so-nothing = The snapshot failed, so nothing was applied: { $e }
app-configs-type-exactly-to-stage-this = Type { $risk_confirm_keyword } exactly to stage this change.
app-configs-audit-commit-created = Audit commit created: { $description }
app-configs-rolled-back-firewall-to-revision = Rolled back firewall to revision v{ $version }
app-configs-this-machine = this machine
app-configs-was-written-but-its-history = { $file } was written, but its history wasn't recorded: { $e }
app-configs-baseline-not-pinned = baseline not pinned: { $e }
app-configs-baseline-not-unpinned = baseline not unpinned: { $e }
app-configs-couldnt-be-read-or-recorded = couldn't be read or recorded: { $e }

## app-containers
app-containers-done = Done: { $summary }
app-containers-failed = { $summary } failed: { $e }
app-containers-no-ai-provider-has-a = No AI provider has a key yet. Add one in Settings → Clankers (AI).
app-containers-every-network = every network
app-containers-this-machine-only = this machine only

## app-danger
app-danger-the-vault-is-busy = the vault is busy
app-danger-asking-to = asking { $label } to { $verb }…
app-danger-type-exactly-to-confirm = Type { $keyword } exactly to confirm.
app-danger-no-active-server = No active server
app-danger-stopped-lab-container-s-failed = Stopped { $ok } lab container(s), { $failed } failed
app-danger-unknown-action = Unknown action

## app-drift
app-drift-a-file-sshd-config-or = a file (sshd_config) or a setting (PermitRootLogin) on any server…
app-drift-couldnt-read-on-a-server = couldn't read { $failed } on a server: { $failed2 }
app-drift-the-baselines-content-wasnt-kept = the baseline's content wasn't kept (only its hash), so there's nothing to push
app-drift-not-reachable-right-now = not reachable right now
app-drift-put-back-to-the-baseline = put { $path } back to the { $e } baseline
app-drift-bring-back = BRING BACK { $path } · { $n } { $n ->
        [one] SERVER
       *[other] SERVERS
    }
app-drift-the-vault-is-busy = the vault is busy
app-drift-crow-has-no-copy-of = Crow has no copy of it from that server
app-drift-only-the-hash-of-that = only the hash of that copy was kept, so it couldn't be pushed anywhere: open the vault and read it again
app-drift-s-is-now-the-baseline = { $name }'s { $path } is now the baseline for { $app }. Other servers are compared with it from now on.
app-drift-not-accepted = Not accepted: { $e }

## app-dr
app-dr-plan-name-e-g-orders = plan name, e.g. orders database
app-dr-rto-minutes = RTO, minutes
app-dr-rpo-minutes = RPO, minutes
app-dr-drill-notes-e-g-restored = drill notes, e.g. restored to a Multipass VM from last night's dump
app-dr-what-matters-one-per-line =
    what matters, one per line:
    /var/lib/postgresql
    /etc/nginx
    orders database
app-dr-backups-one-per-line-path =
    backups, one per line (path or glob | timer):
    /var/backups/db/*.sql.gz | pg-dump.timer
app-dr-rebuild-1-launch-a-fresh =
    ## Rebuild
    1. Launch a fresh server (Add Server)
    2. Restore: zcat /var/backups/db/latest.sql.gz | psql orders
    3. Push the baselines (Drift → bring back)
    4. Check the site (Checks)
app-dr-give-the-plan-a-name = Give the plan a name and pick what it covers.
app-dr-saved = Saved.
app-dr-save-the-plan-first = Save the plan first.
app-dr-not-reachable-right-now-nothing = not reachable right now: nothing could be checked
app-dr-drill-started-follow-the-runbook = Drill started: follow the runbook (restore to a Multipass VM or a scratch server), then FINISH DRILL.
app-dr-drill-recorded-min-the-rto = Drill recorded: { $minutes } min, { $met } the RTO of { $rto_minutes } min.
app-dr-group = group { $g }
app-dr-plan-written-to-keep-a = Plan written to { $path }: keep a copy where you can read it when Crow is down.
app-dr-couldnt-write = Couldn't write { $path }: { $e }

## app-files
app-files-chmod = chmod { $after } { $path }: { $e }
app-files-that-name-has-control-characters = That name has control characters; open it from the terminal yourself.

## app-firewall
app-firewall-8080-or-a-range-8000 = 8080, or a range 8000:8100
app-firewall-anywhere-or-an-address-cidr = anywhere, or an address / CIDR
app-firewall-why-this-rule-exists = why this rule exists
app-firewall-reading-s-firewall = Reading { $name }'s firewall…
app-firewall-ran = Ran: { $summary }
app-firewall-failed = Failed: { $summary } — { $e }
app-firewall-type-exactly-to-apply-this = Type { $risk_confirm_keyword } exactly to apply this change.
app-firewall-not-applied-no-firewall-crow = Not applied: no firewall Crow can change was found on this host.
app-firewall-not-applied-crow-reads-but = Not applied: Crow reads { $label } but never writes it.
app-firewall-not-applied = Not applied: { $e }

## app-fleet-run
app-fleet-run-the-baselines-content-wasnt-kept = { $path }: the baseline's content wasn't kept (hash only), so there's nothing to push
app-fleet-run-push-baseline = push baseline { $path }
app-fleet-run-push-baseline-nothing-has-drifted = PUSH BASELINE · nothing has drifted
app-fleet-run-push-baseline-to-all = PUSH BASELINE TO ALL
app-fleet-run-rolling-reboot = ROLLING REBOOT · { $n } { $n ->
        [one] SERVER
       *[other] SERVERS
    }
app-fleet-run-the-machine-crow-runs-on = the machine Crow runs on: reboot it yourself
app-fleet-run-not-reachable-right-now-so = not reachable right now, so Crow couldn't watch it come back
app-fleet-run-reboot-via = reboot via { $label }
app-fleet-run-restart-the-lab-container = restart the lab container
app-fleet-run-the-machine-crow-runs-on-2 = the machine Crow runs on
app-fleet-run-a-lab-container-reached-without = a lab container, reached without SSH
app-fleet-run-not-reachable-right-now-the = not reachable right now: the new keys are fetched over the current connection
app-fleet-run-new-host-keys-re-pin = new host keys, re-pin, verify a fresh login
app-fleet-run-not-pinned = not pinned
app-fleet-run-other-machines = other machines
app-fleet-run-anyone-else-who-sshes-to = anyone else who SSHes to these servers will see a changed host key and must re-verify it
app-fleet-run-rotate-host-keys = ROTATE HOST KEYS · { $n } { $n ->
        [one] SERVER
       *[other] SERVERS
    }
app-fleet-run-the-machine-crow-runs-on-3 = the machine Crow runs on (your own logins)
app-fleet-run-not-reachable-right-now = not reachable right now
app-fleet-run-end-every-ssh-login-but = end every SSH login but Crow's
app-fleet-run-no-key-is-removed-anyone = no key is removed: anyone whose key still works can log back in (rotate or remove keys for that)
app-fleet-run-revoke-sessions = REVOKE SESSIONS · { $n } { $n ->
        [one] SERVER
       *[other] SERVERS
    }
app-fleet-run-crow-has-only-the-public = Crow has only the public half of this key, so it can't log in with it
app-fleet-run-switch-to-key = SWITCH TO KEY { $name }
app-fleet-run-no-server-logs-in-with = No server logs in with { $name }; nothing to rotate.
app-fleet-run-couldnt-generate-the-new-key = Couldn't generate the new key: { $e }
app-fleet-run-new-key-written-to-if = new key written to { $dir }; if you cancel it stays there, unused
app-fleet-run-rotate = ROTATE { $name } → { $name2 }
app-fleet-run-install-new-key-verify-switch = install new key, verify, switch, remove old key
app-fleet-run-install-key-verify-switch = install key, verify, switch
app-fleet-run-switched-old-key-removed = switched; old key removed
app-fleet-run-type-exactly-to-start = Type { $keyword } exactly to start.
app-fleet-run-approved-by = Approved by { $by }.
app-fleet-run-crow = crow: { $title } — { $summary }
app-fleet-run-pinned-was = pinned { $r } (was { $before })

## app-groups
app-groups-group-name-e-g-workers = group name (e.g. workers, edge, data)

## app-hardening
app-hardening-why-this-risk-is-acceptable = why this risk is acceptable (needed to accept one)
app-hardening-write-why-the-risk-is = Write why the risk is acceptable first: the reason goes in the report and the audit log.
app-hardening-accepted-its-no-longer-flagged = Accepted: it's no longer flagged on the map or here, and the report lists it with your reason.
app-hardening-accepted-as-a-risk = accepted as a risk
app-hardening-not-an-ssh-server-crow = not an SSH server (Crow proves every fix with a fresh SSH login)
app-hardening-not-reachable-right-now = not reachable right now
app-hardening-neither-ufw-nor-firewalld-is = neither ufw nor firewalld is installed: install one (apt install ufw, dnf install firewalld), then fix it
app-hardening-crow-logs-in-as-root = Crow logs in as root with a key: prohibit-password is already the safest setting it can keep
app-hardening-harden = HARDEN · { $code } · { $n } { $n ->
        [one] SERVER
       *[other] SERVERS
    }
app-hardening-report-written-to = Report written to { $path }
app-hardening-couldnt-write-the-report-to = Couldn't write the report to { $path }: { $e }

## app-history-view
app-history-view-disk = DISK { $mount }

## app-incidents
app-incidents-on = { $detail } on { $name }
app-incidents-days = { $secs } days
app-incidents-hours = { $secs } hours
app-incidents-no-ai-provider-has-a = No AI provider has a key yet. Add one in Settings → Clankers (AI).
app-incidents-timeline-written-to = Timeline written to { $path }
app-incidents-couldnt-write = Couldn't write { $path }: { $e }
app-incidents-the-last = { $name }, the last { $secs }

## app-journal
app-journal-no-ai-provider-has-an = No AI provider has an API key. Add one in Settings → Clankers.
app-journal-there-are-no-log-lines = There are no log lines on screen to explain.
app-journal-and-worse = { $label } and worse
app-journal-answered-by-the-backup-the =
    Answered by the backup, { $provider_label }: the primary failed ({ $why }).
    {""}
    { $text }
app-journal-nothing-to-export = Nothing to export
app-journal-copied = Copied { $lines } { $lines ->
        [one] line
       *[other] lines
    }

## app-keys
app-keys-enrolled-into-group = Enrolled '{ $name }' into group '{ $target_group }'
app-keys-removed-key-from-memory-file = Removed key from memory (file preserved)
app-keys-key-name-cannot-be-empty = Key name cannot be empty
app-keys-group-name-cannot-be-empty = Group name cannot be empty
app-keys-path-cannot-be-empty = Path cannot be empty
app-keys-directory-does-not-exist = Directory does not exist: { $expanded }
app-keys-e-g-id-ed25519-bastion = e.g. id_ed25519_bastion
app-keys-e-g-nelson-crow = e.g. nelson@crow
app-keys-e-g-staging-fleet-edge = e.g. Staging Fleet, Edge Bastions
app-keys-e-g-work-keys-or = e.g. ~/work-keys or /etc/ssh
app-keys-enter-key-name = Enter key name…

## app-lab
app-lab-crows-key = Crow's key: { $e }
app-lab-multipass = Multipass: { $e }
app-lab-couldnt-save-in-the-vault = couldn't save { $name } in the vault: { $e }
app-lab-is-in-the-fleet = { $name } is in the fleet ({ $host })
app-lab-launching = Launching { $name }
app-lab-importing = Importing { $name }
app-lab-deleted = { $vm } deleted
app-lab-done = { $vm }: { $label } done

## app-logsearch
app-logsearch-text-or-regex-e-g = text or regex, e.g. Out of memory|segfault
app-logsearch-unit-optional-e-g-nginx = unit (optional), e.g. nginx.service
app-logsearch-a-name-to-save-it = a name to save it as
app-logsearch-no-reachable-server-in = No reachable server in { $label }.
app-logsearch-couldnt-read-its-journal-no = couldn't read its journal (no journalctl, or unreachable)
app-logsearch-name-the-search-to-save = Name the search to save it.
app-logsearch-a-saved-search-needs-text = A saved search needs text, a unit or a priority: one matching everything would always alert.
app-logsearch-saved-switch-alert-on-to = Saved "{ $name }". Switch ALERT on to be told when it matches.
app-logsearch-no-ai-provider-has-a = No AI provider has a key yet. Add one in Settings → Clankers (AI).

## app
app-no-server-is-enrolled-yet = No server is enrolled yet.
app-detecting = Detecting...

## app-notify
app-notify-blank-if-the-server-needs = blank if the server needs no login
app-notify-set-leave-blank-to-keep = set · leave blank to keep it
app-notify-the-smtp-password-or-app = the SMTP password or app password
app-notify-you-example-com-oncall-example = you@example.com, oncall@example.com
app-notify-e-g-22-07-or = e.g. 22-07, or blank
app-notify-the-form-isnt-open = the form isn't open
app-notify-the-port-is-a-number = the port is a number, like 465 or 587
app-notify-the-digest-interval-is-a = the digest interval is a number of minutes
app-notify-theres-a-username-but-no = there's a username but no password
app-notify-the-vault-is-busy-try = the vault is busy; try again
app-notify-the-vault-is-locked = the vault is locked
app-notify-saved-alerts-are-mailed-while = Saved. Alerts are mailed while Crow is running.
app-notify-not-saved = Not saved: { $e }
app-notify-the-saved-password-cant-be = the saved password can't be read while the vault is locked
app-notify-sent-to-check-the-inbox = Sent to { $to }. Check the inbox (and spam).
app-notify-nothing-was-sent = nothing was sent
app-notify-the-smtp-password-cant-be = the SMTP password can't be read (is the vault locked?)

## app-onboard
app-onboard-e-g-10-0-4 = e.g. 10.0.4.32 or prod-db.internal
app-onboard-e-g-prod-db-01 = e.g. prod-db-01
app-onboard-e-g-postgres-primary = e.g. postgres, primary
app-onboard-test-node = test-node · { $distro }
app-onboard-the-servers-address-is-missing = The server's address is missing.
app-onboard-that-isnt-an-address-a = That isn't an address: a host name or IP, without spaces.
app-onboard-the-user-to-log-in = The user to log in as is missing.
app-onboard-pick-the-key-crow-logs = Pick the key Crow logs in with, or use a password once.
app-onboard-type-the-servers-password-crow = Type the server's password: Crow uses it once to install its key.
app-onboard-trust-the-servers-host-key = Trust the server's host key first (or check again).
app-onboard-give-it-a-name = Give it a name.
app-onboard-is-already-the-name-of = { $name } is already the name of another server.
app-onboard-no-home-directory = no home directory
app-onboard-no-free-crow-ed25519-file = no free crow_ed25519 file name in ~/.ssh
app-onboard-saving-crows-key-in-the = saving Crow's key in the vault: { $e }
app-onboard-enter-the-servers-password-on = enter the server's password on step 2 (Credentials)
app-onboard-the-bastion = the bastion
app-onboard-cant-go-through-the-bastion = can't go through the bastion: { $e }
app-onboard-the-host-key-changed-remove = The host key changed. Remove the old entry from ~/.ssh/known_hosts yourself if the change is expected.
app-onboard-no-host-key-was-fetched = No host key was fetched from the server; probe it again.
app-onboard-couldnt-write-ssh-known-hosts = Couldn't write ~/.ssh/known_hosts: { $e }
app-onboard-server-enrolled-into-fleet = Server '{ $name }' enrolled into fleet
app-onboard-password-login = password login: { $why }
app-onboard-logged-in-with-the-password = logged in with the password once; installed Crow's key ({ $fingerprint })
app-onboard-key-only-from-now-on = key-only from now on
app-onboard-password-login-2 = password login: { $e }

## app-overview
app-overview-search-services = search services…  ( / )
app-overview-search-command-user-pid = search command, user, pid…  ( / )
app-overview-search-name-group-shell = search name, group, shell…  ( / )
app-overview-search-port-address-comment = search port, address, comment…  ( / )
app-overview-couldnt-read-updates = couldn't read updates: { $e }
app-overview-failed = { $failed } FAILED
app-overview-on = ON
app-overview-off = OFF
app-overview-established-listening = { $established } established, { $listening } listening
app-overview-no-ai-provider-has-a = No AI provider has a key yet. Add one in Settings → Clankers (AI).

## app-palette
app-palette-page = PAGE
app-palette-this-server = this server
app-palette-danger-zone = Danger Zone
app-palette-offline-unreachable-down = offline unreachable down
app-palette-server = SERVER
app-palette-terminal = TERMINAL
app-palette-terminal-on = Terminal on { $name }
app-palette-place = PLACE
app-palette-fleet = Fleet
app-palette-fleet-map = Fleet Map
app-palette-fleet-setup-policies = Fleet Setup & Policies
app-palette-patching = Patching
app-palette-rollouts = Rollouts
app-palette-drift-config-search = Drift & config search
app-palette-audit-log = Audit Log
app-palette-settings = Settings
app-palette-ssh-keys = SSH Keys
app-palette-fleet-config = FLEET CONFIG
app-palette-action = ACTION
app-palette-add-a-server = Add a server
app-palette-check-posture-now = Check posture now
app-palette-reconnect-to-this-server = Reconnect to this server
app-palette-lock-the-vault = Lock the vault
app-palette-take-a-snapshot = Take a snapshot
app-palette-security-stance-2fa = Security stance (2FA)
app-palette-about-crow = About Crow
app-palette-key = KEY
app-palette-config = CONFIG
app-palette-service = SERVICE
app-palette-process = PROCESS
app-palette-pid = pid { $pid } · { $user }
app-palette-search-servers-pages-config-files = Search servers, pages, config files, actions, keys…

## app-password-login
app-password-login-crow-doesnt-connect-to-this = Crow doesn't connect to this server with its own key yet. Set up key login first (Add Server → install Crow's key), then turn passwords off.
app-password-login-password-login-is-already-off = ✓ Password login is already off on this server.
app-password-login-this-sshd-config-doesnt-include = This sshd_config doesn't include sshd_config.d, so Crow's drop-in wouldn't apply. Set PasswordAuthentication to no in the sshd_config sheet, then reload sshd.
app-password-login-couldnt-read-sshds-settings = Couldn't read sshd's settings: { $e }

## app-patching
app-patching-every-server = every server
app-patching-group = group { $g }
app-patching-any-time-e-g-sun = any time · e.g. Sun 02-05
app-patching-the-machine-crow-runs-on = the machine Crow runs on: patch it yourself
app-patching-not-reachable-right-now = not reachable right now
app-patching-its-updates-havent-been-read = its updates haven't been read yet: REFRESH first
app-patching-couldnt-read-its-updates = couldn't read its updates: { $e }
app-patching-no-pending = no { $label } pending
app-patching-cant-be-rebooted-from-here = can't be rebooted from here ({ $why }), so it's left out of a run that reboots
app-patching-then-reboot-if-needed = {" "}· then reboot if needed
app-patching-rebooted = { $done }; rebooted: { $m }
app-patching-apply = APPLY { $label } · { $label2 } · { $n } { $n ->
        [one] SERVER
       *[other] SERVERS
    }
app-patching-reboot-where-needed = REBOOT WHERE NEEDED · { $label } · { $n } { $n ->
        [one] SERVER
       *[other] SERVERS
    }

## app-people
app-people-failed-exit = `{ $cloned }` failed (exit { $status }): { $stderr }
app-people-a-keys-fingerprint-sha256-or = a key's fingerprint (SHA256:…) or comment
app-people-username-e-g-alice = username, e.g. alice
app-people-their-public-key-ssh-ed25519 = their public key: ssh-ed25519 AAAA… alice@laptop
app-people-revoke-keys-lock-and-expire = revoke keys, lock and expire
app-people-revoke-keys-delete-the-account = revoke keys, delete the account (home kept)
app-people-offboard = OFFBOARD { $user } · { $n } { $n ->
        [one] SERVER
       *[other] SERVERS
    }
app-people-text = { $user }: { $verb } ({ $count } { $count ->
        [one] step
       *[other] steps
    })
app-people-revoke-key = REVOKE KEY { $label } · { $n } { $n ->
        [one] SERVER
       *[other] SERVERS
    }
app-people-remove-the-key-from = remove the key from { $count } { $count ->
        [one] account
       *[other] accounts
    }
app-people-fill-in-the-username-and = Fill in the username and their public key first.
app-people-no-server-in-has-been = No server in { $label } has been read yet: READ AGAIN first.
app-people-onboard = ONBOARD { $user } · { $label } · { $n } { $n ->
        [one] SERVER
       *[other] SERVERS
    }
app-people-create-add-their-key = create { $user }{ $sudo }, add their key
app-people-add-the-key-to = add the key to { $user }

## app-plugins
app-plugins-local-vms-containers = Local VMs & Containers
app-plugins-multipass-vms = Multipass VMs
app-plugins-local-lab-containers = Local Lab Containers
app-plugins-is-on = is on
app-plugins-is-off = is off
app-plugins-mails = mails { $s }
app-plugins-not-set-up-yet = not set up yet

## app-providers
app-providers-set-type-to-replace = set · type to replace
app-providers-not-set = not set
app-providers-default = default: { $d }
app-providers-the-vault-is-busy-try = the vault is busy; try again
app-providers-the-vault-is-locked-or = the vault is locked or has no password, so the provider's secrets can't be read
app-providers-no-provider-accounts-yet-set = No provider accounts yet. Set one up in Settings → Providers.
app-providers-no-encryption-key = no encryption key
app-providers-this-provider-cant-list-instances = this provider can't list instances
app-providers-no-longer-listed-by = { $m } (no longer listed by { $name })
app-providers-now-lists-for-it-crow = { $m }: { $name } now lists { $addrs } for it; Crow still connects to the old address

## app-recovery
app-recovery-checking-the-host = checking the host…
app-recovery-connecting = connecting…
app-recovery-is-reachable-again = { $name } is reachable again
app-recovery-type-exactly-to-trust-the = Type { $name } exactly to trust the new key.
app-recovery-no-key-was-fetched-from = No key was fetched from the server; nothing to trust.
app-recovery-known-hosts-wasnt-changed = known_hosts wasn't changed: { $e }

## app-region
app-region-located = { $found } located
app-region-by-geoip = { $by_geoip } by GeoIP
app-region-without-cloud-metadata = { $unknown } without cloud metadata
app-region-unreachable = { $failed } unreachable
app-region-asking-server-s = asking { $count } server(s)…
app-region-geoip-unavailable = { $note } · GeoIP unavailable: { $e }

## app-render
app-render-as = as { $name } · { $name2 } · { $session_label }
app-render-no-active-server-selected = NO ACTIVE SERVER SELECTED
app-render-select-a-server-tab-or = Select a server tab or enroll a new Linux host to inspect services and config.
app-render-go-to-fleet-overview = ⬢ GO TO FLEET OVERVIEW
app-render-enroll-new-server = + ENROLL NEW SERVER
app-render-not-built-yet = { $active_view } · not built yet
app-render-fleet = Fleet
app-render-lab-container = lab container
app-render-ssh-connecting = ssh · connecting
app-render-ssh-connected = ssh · connected
app-render-ssh = ssh · { $label }

## app-rollouts
app-rollouts-the-machine-crow-runs-on = the machine Crow runs on: run it yourself
app-rollouts-not-reachable-right-now = not reachable right now
app-rollouts-the-files-new-content = the file's new content
app-rollouts-that-baselines-content-wasnt-kept = That baseline's content wasn't kept (only its hash), so it can't be rolled out.
app-rollouts-write-the-command-to-run = Write the command to run first.
app-rollouts-as-root = as root:{" "}
app-rollouts-rollout-command = ROLLOUT · COMMAND
app-rollouts-the-files-path-must-be = The file's path must be absolute, like /etc/nginx/conf.d/app.conf.
app-rollouts-write-the-files-new-content = Write the file's new content first (or start from a server's copy or a baseline).
app-rollouts-reading-each-servers-current-file = Reading each server's current file for the preview; plan again once it's read.
app-rollouts-already-identical = already identical
app-rollouts-write = write { $path } (+{ $add } −{ $del })
app-rollouts-create = create { $path }
app-rollouts-couldnt-read-its-copy = couldn't read its copy: { $e }
app-rollouts-its-copy-hasnt-been-read = its copy hasn't been read yet
app-rollouts-rollout = ROLLOUT · { $path }
app-rollouts-text = { $title } · { $n } { $n ->
        [one] SERVER
       *[other] SERVERS
    }
app-rollouts-remove-it-didnt-exist = remove { $path } (it didn't exist)
app-rollouts-put-back-as-it-was = put back { $path } as it was
app-rollouts-roll-back = ROLL BACK { $path } · { $n } { $n ->
        [one] SERVER
       *[other] SERVERS
    }

## app-secrets
app-secrets-the-key-stored-there-is = the key stored there is malformed
app-secrets-not-saved-set-up-the = not saved: set up the vault password first, then save again
app-secrets-not-saved = not saved
app-secrets-the-os-keyring-no-longer = The OS keyring no longer had Crow's encryption key, so { $lost } saved secret(s) couldn't be read and were cleared. Enter them again.
app-secrets-the-vault-is-busy = the vault is busy
app-secrets-moved-ai-api-key-s = Moved { $n } AI API key(s) out of plain text into the encrypted vault.
app-secrets-couldnt-move-ai-api-keys = Couldn't move AI API keys into the vault: { $e }
app-secrets-the-os-keyring-no-longer-2 = The OS keyring no longer has Crow's encryption key, so saved provider tokens and AI keys can't be read. Enter them again.
app-secrets-ai-api-keys-are-still = AI API keys are still stored in plain text: { $why }.
app-secrets-ai-api-keys-are-still-2 = AI API keys are still stored in plain text because no OS keyring is available. Turn on the vault password to encrypt them.

## app-server-facts
app-server-facts-reconnecting-to = Reconnecting to { $name }…
app-server-facts-is-connected-again = { $name } is connected again
app-server-facts-couldnt-reconnect-to = Couldn't reconnect to { $name }: { $e }

## app-session
app-session-enter-master-password = Enter master password…
app-session-password-cannot-be-empty = Password cannot be empty
app-session-6-digit-2fa-code-is = 6-digit 2FA code is required
app-session-this-computer = this computer
app-session-enter-master-password-min-8 = Enter master password (min 8 characters)…
app-session-re-type-password = Re-type password…
app-session-6-digit-code-e-g = 6-digit code (e.g. 123456)
app-session-password-must-be-at-least = Password must be at least 8 characters
app-session-passwords-do-not-match = Passwords do not match
app-session-please-enter-the-6-digit = Please enter the 6-digit confirmation code from your authenticator
app-session-invalid-6-digit-code-check = Invalid 6-digit code. Check your authenticator and try again

## app-settings
app-settings-not-changed = Not changed: { $message }
app-settings-not-changed-2 = Not changed: { $e }
app-settings-value = value…

## app-team
app-team-not-allowed = Not allowed: { $why }.
app-team-passphrase = passphrase
app-team-name-e-g-ana = name, e.g. ana
app-team-their-passphrase-8-characters-they = their passphrase (8+ characters; they can change it later)
app-team-a-second-members-passphrase = a second member's passphrase
app-team-signed-in-as = Signed in as { $name } ({ $name2 }).
app-team-that-passphrase-isnt-right = That passphrase isn't right.
app-team-give-the-member-a-name = Give the member a name no one else has.
app-team-team-mode-is-on-is = Team mode is on: { $name } is the first Admin and is signed in. From now on Crow asks who's at the keyboard when it starts.
app-team-added-as = Added { $name } as { $name2 }.
app-team-thats-the-last-admin-make = That's the last Admin: make someone else Admin first.
app-team-no-members-left-team-mode = No members left: team mode is off, and Crow is single-user again.
app-team-needs-a-second-members-approval = needs a second member's approval: someone else allowed fleet runs types their passphrase

## app-terminal
app-terminal-shell = shell { $i }
app-terminal-follows-the-shells-title-when = follows the shell's title when empty
app-terminal-couldnt-start-a-terminal = couldn't start a terminal: { $e }

## app-updates
app-updates-downloading-and-verifying-crow = Downloading and verifying Crow { $version }…
app-updates-couldnt-start-the-new-version = Couldn't start the new version: { $e }

## app-users
app-users-not-applied = Not applied: { $e }
app-users-ran = Ran: { $summary }
app-users-failed = Failed: { $summary } — { $e }
app-users-not-applied-the-passwords-dont = Not applied: the passwords don't match
app-users-new-password = new password
app-users-again = again
app-users-e-g-alice-deploy = e.g. alice, deploy
app-users-e-g-alice-wonderland-optional = e.g. Alice Wonderland (optional)
app-users-password = password
app-users-not-applied-crow-logs-in = Not applied: Crow logs in to this server as { $user }; deleting it would lock Crow (and you, through Crow) out. Switch the server to another account first.
app-users-not-applied-thats-the-key = Not applied: that's the key Crow logs in to this server with; revoking it would lock Crow out. Switch the server to another key first (Keys → switch), then revoke this one.

## app-vault-manage
app-vault-manage-current-password = current password
app-vault-manage-2fa-code = 2FA code
app-vault-manage-new-password-8-characters = new password (8+ characters)
app-vault-manage-new-password-again = new password again
app-vault-manage-the-new-passwords-dont-match = the new passwords don't match
app-vault-manage-password-changed-your-secrets-and = Password changed. Your secrets and 2FA are unchanged.
app-vault-manage-tick-i-understand-first = Tick "I understand" first.
app-vault-manage-crow-is-open-now-no = Crow is OPEN now: no password, and the OS keyring holds the key.
app-vault-manage-still-locked-couldnt-hand-the = Still LOCKED: couldn't hand the key to the OS keyring ({ $e }).

## about
about-crow-v-platform-profile-repository =
    Crow v{ $env }
    Platform: { $std } ({ $std2 })
    Profile: { $cfg }
    Repository: { $repo }
    Website: { $website }
    Community: { $forum }
    Engine: GPUI Kit (WGPU) + crow-config-core
    Vault: Argon2id + ChaCha20-Poly1305 + TOTP
    {""}
about-debug-build = debug build
about-release-build = release build
about-about-crow = About Crow
about-crow = Crow
about-native-linux-server-manager-over = Native Linux server manager over SSH
about-website = Website
about-source = Source
about-community = Community
about-build = BUILD
about-platform = Platform
about-profile = Profile
about-ui-toolkit = UI toolkit
about-gpui-kit-0-6-1 = GPUI Kit 0.6.1 · WGPU
about-config-engine = Config engine
about-crow-config-core-schema-ir = crow-config-core · schema IR
about-vault = Vault
about-argon2id-chacha20-poly1305-totp = Argon2id · ChaCha20-Poly1305 · TOTP
about-credits = CREDITS
about-icons = Icons
about-tabler-icons-mit = Tabler Icons · MIT
about-flags = Flags
about-flag-icons-mit = flag-icons · MIT
about-distro-logos = Distro logos
about-dashboard-icons-by-homarr-labs = Dashboard Icons by homarr-labs · Apache-2.0
about-geoip-data = GeoIP data
about-font = Font
about-jetbrains-mono-ofl-1-1 = JetBrains Mono · OFL-1.1
about-license = LICENSE
about-aeupl-1-2-free-for = AEUPL-1.2. Free for any non-commercial use. See LICENSE in the repository.
about-copied-system-info = ✓ COPIED SYSTEM INFO
about-copy-system-info = COPY SYSTEM INFO
about-ok = OK
about-pre-release = {" "}(PRE-RELEASE)
about-release-notes = RELEASE NOTES
about-skip-this-version = SKIP THIS VERSION
about-verify-install = VERIFY & INSTALL
about-restart-now = RESTART NOW
about-crow-doesnt-download-anything-unless = Crow doesn't download anything unless you turn on installing in Settings → General.
about-crow-is-available = CROW { $version }{ $r } IS AVAILABLE

## danger-zone
danger-zone-simulated-signal-sent-to-crow = Simulated { $action } signal sent to { $name } — Crow does not execute real power actions yet
danger-zone-this-host-isnt-running-ufw = This host isn't running ufw — nothing to flush.
danger-zone-ufw-rules-flushed-on = ufw rules flushed on { $label }
danger-zone-power = Power
danger-zone-through-works-even-when-ssh = Through { $name }: works even when SSH is down.
danger-zone-power-off = POWER OFF
danger-zone-simulated-crow-doesnt-send-real = Simulated: Crow doesn't send real power signals to hosts yet. Link a provider for real power control.
danger-zone-reboot-45s-downtime = REBOOT · 45s DOWNTIME
danger-zone-snapshots = SNAPSHOTS…
danger-zone-snapshots-2 = Snapshots
danger-zone-taken-and-listed-at = Taken and listed at { $name }.
danger-zone-flush-firewall = Flush firewall
danger-zone-resets-ufw-every-rule-is = Resets ufw: every rule is removed and the firewall is disabled.
danger-zone-flush-firewall-ufw = FLUSH FIREWALL (UFW)
danger-zone-rotate-host-keys = Rotate host keys
danger-zone-not-built-yet = Not built yet.
danger-zone-not-yet-built = NOT YET BUILT
danger-zone-lab-containers = Lab containers
danger-zone-stops-every-lab-node-crow = Stops every lab node Crow created on this machine, and nothing else.
danger-zone-kill-all-lab-containers = KILL ALL LAB CONTAINERS
danger-zone-archive-server = Archive server
danger-zone-removes-it-from-the-fleet = Removes it from the fleet; its data is kept until the purge date, and it can be restored.
danger-zone-archive-server-2 = ARCHIVE SERVER
danger-zone-danger-zone = DANGER ZONE
danger-zone-type-the-actions-keyword-to = type the action's keyword to confirm · no single-click destructive actions
danger-zone-type-to-confirm = Type { $keyword } to confirm:
danger-zone-execute = EXECUTE ⏎
danger-zone-cancel = cancel

## distro-icon
distro-icon-alma-linux = alma linux
distro-icon-red-hat = red hat
distro-icon-arch-linux = arch linux
distro-icon-azure-linux = azure linux
distro-icon-mac-os = mac os
distro-icon-rhel = RHEL

## identity-bar
identity-bar-local = LOCAL
identity-bar-lab-container = LAB CONTAINER
identity-bar-connecting = CONNECTING…
identity-bar-connected = CONNECTED
identity-bar-via-unreachable = via { $name } (unreachable)
identity-bar-via = via { $name }
identity-bar-unverified = unverified
identity-bar-group = group
identity-bar-none = none ▾
identity-bar-bastion = bastion ✓
identity-bar-bastion-no = bastion: no
identity-bar-search = Search
identity-bar-reconnect = RECONNECT
identity-bar-start-vm = START VM
identity-bar-stop-vm = STOP VM
identity-bar-restart-vm = RESTART VM
identity-bar-crow-reaches-through-which-is = Crow reaches { $name } through { $name2 }, which is unreachable itself: fix { $name2 } first. { $detail }
identity-bar-crow-cant-read-right-now = Crow can't read { $name } right now — pages below stay empty until the connection works. { $detail }
identity-bar-what-happened = what happened →
identity-bar-set-up-key-login = set up key login →
identity-bar-region-unknown = region: unknown ▾
identity-bar-region-unknown-2 = { $region_provider } · region unknown ▾
identity-bar-set-by-hand = { $place } (set by hand) ▾
identity-bar-by-ip = { $place } (by IP) ▾

## palette
palette-recent-first = { $scope_label } · recent first
palette-text = { $scope_label } · { $count } { $count ->
        [one] result
       *[other] results
    }
palette-nothing-matches-try-a-server = Nothing matches. Try a server name, a page (logs, firewall), a config file, or an action.
palette-navigate = ↑↓ navigate
palette-open = ⏎ open
palette-into-a-server = ⇥ into a server
palette-back-out = ⌫ back out
palette-esc-close = esc close

## password-login
password-login-reading-sshds-settings = Reading sshd's settings…
password-login-working-checking-a-fresh-key = Working: checking a fresh key login, writing the drop-in, validating, reloading, checking again…
password-login-done-keep-a-key-for = Done. Keep a key for this server somewhere safe: passwords won't get you in any more.
password-login-password-login-is-still-on = Password login is still on.
password-login-password-login-is-on-anyone = Password login is ON. Anyone who guesses or steals a password can get in, and bots try constantly. With it off, only SSH keys work.
password-login-what-crow-will-do-stopping = What Crow will do, stopping and undoing at the first problem:
password-login-1-check-that-a-fresh = 1. Check that a fresh key login works (a new connection, not the one Crow holds).
password-login-2-write-passwordauthentication-no-kbdinteractiveauthentication = 2. Write { $dropin } (PasswordAuthentication no, KbdInteractiveAuthentication no); it's read before other drop-ins, so a cloud image's own setting can't override it.
password-login-3-validate-with-sshd-t = 3. Validate with sshd -t, and confirm with sshd -T that passwords are really off.
password-login-4-reload-sshd-open-sessions = 4. Reload sshd (open sessions stay up), then check a fresh key login again; if that fails, remove the file and reload.
password-login-these-accounts-have-a-password = These accounts have a password and will lose SSH login with it: { $pre }. Make sure each one has a key first.
password-login-i-understand = I understand
password-login-turn-off-ssh-password-login = Turn off SSH password login · { $server_name }
password-login-turn-off-password-login = TURN OFF PASSWORD LOGIN
password-login-cancel = CANCEL
password-login-close = CLOSE

## recovery-panel
recovery-panel-checking = CHECKING
recovery-panel-what-to-do = WHAT TO DO
recovery-panel-trust-new-key = TRUST NEW KEY
recovery-panel-install-key-with-password = INSTALL KEY WITH PASSWORD
recovery-panel-try-again = TRY AGAIN
recovery-panel-close = CLOSE

## sidebar
sidebar-overview = Overview
sidebar-history = History
sidebar-services = Services
sidebar-processes = Processes
sidebar-sockets = Sockets
sidebar-containers = Containers
sidebar-config = Config
sidebar-logs = Logs
sidebar-files = Files
sidebar-cron = Cron
sidebar-users = Users
sidebar-firewall = Firewall
sidebar-terminal = Terminal
sidebar-collapse = Collapse
sidebar-danger-zone = Danger Zone

## snapshot-offer
snapshot-offer-snapshot-the-whole-server-at = Snapshot the whole server at { $provider } first?
snapshot-offer-youre-about-to-apply-a = You're about to apply { $file } ({ $description }). A mistake here can lock you out of SSH, and then Crow's file backup can't reach the server to undo it. A { $provider } snapshot restores the whole machine from outside. The change waits until { $provider } accepts the snapshot; if it fails, nothing is applied.
snapshot-offer-remember-my-choice-for-these = Remember my choice for these changes
snapshot-offer-taking-snapshot = TAKING SNAPSHOT…
snapshot-offer-snapshot-first-then-apply = SNAPSHOT FIRST, THEN APPLY
snapshot-offer-apply-without = APPLY WITHOUT
snapshot-offer-cancel = CANCEL

## snapshots-panel
snapshots-panel-asking = asking { $provider }…
snapshots-panel-has-no-snapshots-of-yet = { $provider } has no snapshots of { $server } yet.
snapshots-panel-automatic = (automatic)
snapshots-panel-snapshots = SNAPSHOTS · { $server }
snapshots-panel-at = at { $provider }
snapshots-panel-restore-a-snapshot-from-the = Restore a snapshot from the provider's own console: restoring replaces the server's disks, and Crow doesn't do that for you.

## stance
stance-2fa = 2FA
stance-text = · { $weak } { $weak ->
        [one] issue
       *[other] issues
    }
stance-security-stance = SECURITY STANCE
stance-no-vault-password-key-held = no vault password · key held by your OS keyring
stance-vault-password-2fa-key-locked = vault password + 2FA · key locked by your password
stance-weakening-it-right-now = WEAKENING IT RIGHT NOW
stance-what-it-protects = WHAT IT PROTECTS
stance-what-youre-exposed-to = WHAT YOU'RE EXPOSED TO
stance-what-it-costs = WHAT IT COSTS
stance-nothing-is-usable-without-your = nothing is usable without your password and a 2FA code, at the price of unlocking every time and losing the secrets if you lose either.
stance-no-unlocking-at-the-price = no unlocking, at the price of anything running as you being able to read every secret.
stance-switch-to-locked = SWITCH TO LOCKED →
stance-lock-now = LOCK NOW
stance-vault-security-settings = VAULT & SECURITY SETTINGS
stance-no-password = no password
stance-password-2fa = password + 2FA
stance-protects = PROTECTS
stance-exposed-to = EXPOSED TO
stance-costs = COSTS
stance-stay-open-and-save = STAY OPEN AND SAVE
stance-i-understand-while-im-logged = I understand: while I'm logged in, anything running as me can read these secrets.
stance-go-locked-set-a-password = GO LOCKED: SET A PASSWORD →
stance-where-should-this-secret-stand = Where should this secret stand?
stance-crow-is-about-to-store = Crow is about to store its first secret. Both stances encrypt it; they differ in who can get at it and what you give up. Pick one on purpose. You can switch to Locked later from the stance badge.
stance-cancel-dont-save = CANCEL (DON'T SAVE)
stance-the-other-stance = The other stance, { $label }: { $other }

## stat-strip
stat-strip-vcpu-peak = { $vcpu_count } vCPU · peak { $m }%
stat-strip-inodes-iowait = inodes { $pct_or_dash } · iowait { $pct_or_dash2 }
stat-strip-mb-s = ↓ { $rx_mbps } · ↑ { $tx_mbps } Mb/s
stat-strip-turbo-buffer = TURBO BUFFER
stat-strip-s-lag-local-memory-cache = · -{ $lag_secs }s lag · local memory cache
stat-strip-ahead-in-s = AHEAD · { $description } · in { $lead_seconds }s
stat-strip-next-s-nothing-unusual = next { $lag_secs }s: nothing unusual
stat-strip-cpu-load = CPU LOAD
stat-strip-memory = MEMORY
stat-strip-disk-nvme = DISK { $disk_mount } (NVME)
stat-strip-load-avg = LOAD AVG
stat-strip-uptime = UPTIME
stat-strip-sampled = sampled { $last_sample_ts }
stat-strip-network = NETWORK
stat-strip-mb-s-2 = Mb/s

## status-bar
status-bar-no-open-alerts = no open alerts
status-bar-1-open-alert = 1 open alert
status-bar-open-alerts = { $n } open alerts
status-bar-agentless-ssh = agentless · ssh
status-bar-v-available = v{ $v } available ↗

## titlebar
titlebar-crow = CROW
titlebar-fleet = FLEET
titlebar-agentless-ssh = { $server_count } { $server_count ->
        [one] host
       *[other] hosts
    } · agentless ssh
titlebar-srv = SRV
titlebar-utc = UTC
titlebar-you = YOU
titlebar-overview = { $name } · Overview
titlebar-server-overview = Server Overview
titlebar-fleet-overview = Fleet Overview
titlebar-enroll-new-server = Enroll New Server…
titlebar-fleet-setup-topology-policy = Fleet Setup — Topology & Policy
titlebar-this-server = THIS SERVER
titlebar-managed-configs = Managed Configs
titlebar-systemd-journal-logs = Systemd Journal Logs
titlebar-application = APPLICATION
titlebar-command-palette = Command Palette…
titlebar-settings = Settings
titlebar-about-crow = About Crow…
titlebar-session = SESSION
titlebar-lock-vault-disconnect = Lock Vault & Disconnect
titlebar-quit-crow = Quit Crow

## audit-model
audit-model-by = { $text } · by { $a }
audit-model-enrolled = enrolled { $login_user }@{ $host }:{ $port }
audit-model-saved = saved { $target }
audit-model-restored-an-earlier-version-of = restored an earlier version of { $target }
audit-model-baseline-for = baseline for { $target }: { $before_state }
audit-model-killed-process = killed process { $target }
audit-model-purged-stored-data-for = purged stored data for { $target }
audit-model-turned-off-ssh-password-login = turned off SSH password login
audit-model-trusted-a-new-host-key = trusted a new host key for { $target }
audit-model-opened-a-terminal = opened a terminal
audit-model-at-the-provider = { $k } at the provider
audit-model-rolling-reboot = rolling reboot: { $before_state }
audit-model-ufw = ufw { $target }
audit-model-user = user { $target }

## audit
audit-audit-log = AUDIT LOG
audit-failed-every-server = { $count } { $count ->
        [one] change
       *[other] changes
    } · { $failed } failed · every server
audit-server = SERVER
audit-kind = KIND
audit-nothing-recorded-yet = Nothing recorded yet.

## config-cron-editor
config-cron-editor-every-minute = Every minute (* * * * *)
config-cron-editor-every-minutes = Every { $step } minutes
config-cron-editor-every-hour-on-the-hour = Every hour on the hour (0 * * * *)
config-cron-editor-every-hour-at-minute = Every hour at minute { $m }
config-cron-editor-every-day-at = Every day at { $h }:{ $m }
config-cron-editor-sunday = Sunday
config-cron-editor-monday = Monday
config-cron-editor-tuesday = Tuesday
config-cron-editor-wednesday = Wednesday
config-cron-editor-thursday = Thursday
config-cron-editor-friday = Friday
config-cron-editor-saturday = Saturday
config-cron-editor-weekday-mon-fri = Weekday (Mon-Fri)
config-cron-editor-weekend-sat-sun = Weekend (Sat-Sun)
config-cron-editor-weekly-on-at = Weekly on { $day_name } at { $h }:{ $m }
config-cron-editor-monthly-on-day-at = Monthly on day { $dom } at { $h }:{ $m }
config-cron-editor-schedule = Schedule: { $m } { $h } { $dom } { $mon } { $dow }
config-cron-editor-active-schedules = { $active_count } ACTIVE SCHEDULES
config-cron-editor-unsaved-edits = UNSAVED EDITS
config-cron-editor-in-sync = IN SYNC
config-cron-editor-new-cron-job = + NEW CRON JOB
config-cron-editor-revert = REVERT
config-cron-editor-stage-apply = STAGE & APPLY
config-cron-editor-history = HISTORY · { $rev_count }
config-cron-editor-each-job-runs-as-its = Each job runs as its USER · cron rereads /etc/crontab by itself after a save
config-cron-editor-no-jobs-in-etc-crontab = No jobs in /etc/crontab. + NEW CRON JOB adds one.
config-cron-editor-on = ON
config-cron-editor-schedule-2 = SCHEDULE
config-cron-editor-user = USER
config-cron-editor-command = COMMAND
config-cron-editor-every-15m = Every 15m
config-cron-editor-hourly = Hourly
config-cron-editor-daily-3-am = Daily (3 AM)
config-cron-editor-daily-midnight = Daily (Midnight)
config-cron-editor-weekly-sun = Weekly (Sun)
config-cron-editor-monthly-1st = Monthly (1st)
config-cron-editor-workdays-9-am = Workdays (9 AM)
config-cron-editor-quick-schedule-presets = QUICK SCHEDULE PRESETS
config-cron-editor-5-field-schedule-expression = 5-FIELD SCHEDULE EXPRESSION
config-cron-editor-minute-0-59 = Minute (0-59)
config-cron-editor-hour-0-23 = Hour (0-23)
config-cron-editor-day-of-month-1-31 = Day of Month (1-31)
config-cron-editor-month-1-12 = Month (1-12)
config-cron-editor-day-of-week-0-7 = Day of Week (0-7)
config-cron-editor-execution-user = EXECUTION USER
config-cron-editor-shell-command = SHELL COMMAND
config-cron-editor-any = {"*"} (any)

## config-journald-editor
config-journald-editor-crash-safe = CRASH-SAFE
config-journald-editor-volatile-risk = VOLATILE RISK
config-journald-editor-unsaved-edits = UNSAVED EDITS
config-journald-editor-in-sync = IN SYNC
config-journald-editor-usage = Usage: { $disk_usage_display } / { $max_use_display }
config-journald-editor-revert = REVERT
config-journald-editor-stage-apply-config = STAGE & APPLY CONFIG
config-journald-editor-history = HISTORY · { $rev_count }
config-journald-editor-why-default-linux-journal-configs = WHY DEFAULT LINUX JOURNAL CONFIGS FAIL IN PRODUCTION
config-journald-editor-by-default-systemd-journal-uses = By default, systemd journal uses silent compile-time limits and Storage=auto. On busy servers or post-incident reboots, logs rotate out in 30 minutes or vanish completely. Crow-Config enforces explicit boundaries so you never lose evidence of a crash.
config-journald-editor-storage = Storage
config-journald-editor-controls-where-journal-files-are = Controls where journal files are stored. Volatile keeps logs in RAM (/run), which are permanently wiped on reboot. Persistent guarantees writes to /var/log/journal/.
config-journald-editor-persistent-rec = PERSISTENT (Rec)
config-journald-editor-survives-reboots = Survives reboots
config-journald-editor-volatile = VOLATILE
config-journald-editor-ram-only-wiped-on-reboot = RAM only · Wiped on reboot
config-journald-editor-auto = AUTO
config-journald-editor-default-fallback = Default fallback
config-journald-editor-systemmaxuse = SystemMaxUse
config-journald-editor-the-maximum-disk-space-journals = The maximum disk space journals may occupy on persistent storage. When exceeded, systemd removes the oldest archive files.
config-journald-editor-4g-rec = 4G (Rec)
config-journald-editor-custom = Custom
config-journald-editor-maxretentionsec = MaxRetentionSec
config-journald-editor-the-maximum-time-to-store = The maximum time to store journal entries. Prevents older log archives from lingering indefinitely once log volume is low.
config-journald-editor-30day-rec = 30day (Rec)
config-journald-editor-0-unlimited = 0 (Unlimited)
config-journald-editor-systemkeepfree = SystemKeepFree
config-journald-editor-guarantees-that-at-least-this = Guarantees that at least this much disk space is left free on the root filesystem, preventing log-flood out-of-disk crashes.
config-journald-editor-1-0-gb = 1.0 GB
config-journald-editor-2-0-gb-rec = 2.0 GB (Rec)
config-journald-editor-5-0-gb = 5.0 GB
config-journald-editor-ratelimitburst-ratelimitintervalsec = RateLimitBurst / RateLimitIntervalSec
config-journald-editor-suppresses-runaway-spam-loops-from = Suppresses runaway spam loops from flooding journals and choking the system (10,000 entries per 30 seconds default).
config-journald-editor-entries-s = { $rate_limit_burst } entries / { $rate_limit_interval_sec }s
config-journald-editor-anti-flood-enabled = ✓ Anti-flood enabled

## config-managed-files
config-managed-files-config-files = CONFIG FILES
config-managed-files-unverified-distro-generic-scan-only = ⚠ Unverified distro — generic scan only, some configs may be missing.
config-managed-files-crow-ui-mapped = ● CROW UI MAPPED
config-managed-files-edited = EDITED
config-managed-files-crow-ui = CROW UI
config-managed-files-system-configs-raw = ○ SYSTEM CONFIGS (RAW)
config-managed-files-ro = RO
config-managed-files-raw = RAW
config-managed-files-schema-packs-active = SCHEMA PACKS ACTIVE
config-managed-files-systemd-postgres-openssh-ufw = systemd · postgres · openssh · ufw

## config-pending-diff-rail
config-pending-diff-rail-save-failed = SAVE FAILED · { $err }
config-pending-diff-rail-read-only = READ-ONLY · { $reason }
config-pending-diff-rail-history = HISTORY · { $err }
config-pending-diff-rail-restore-v = RESTORE v{ $v }?
config-pending-diff-rail-same-as-host = same as host
config-pending-diff-rail-pending-diff = PENDING DIFF
config-pending-diff-rail-unified = unified · { $sel_file }
config-pending-diff-rail-author = Author:
config-pending-diff-rail-on-host = --- on host/{ $sel_file }
config-pending-diff-rail-back-to-pending-diff = ← back to pending diff
config-pending-diff-rail-baseline = --- baseline/{ $sel_file }
config-pending-diff-rail-staged-unsaved = +++ staged/{ $sel_file } (unsaved)
config-pending-diff-rail-no-unsaved-edits = No unsaved edits
config-pending-diff-rail-on-save = ON SAVE
config-pending-diff-rail-version-history-revisions = VERSION HISTORY ({ $count } REVISIONS)
config-pending-diff-rail-on-host-not-recorded = on host: not recorded
config-pending-diff-rail-on-host-v = On host: v{ $active_rev }
config-pending-diff-rail-hash-only = HASH ONLY
config-pending-diff-rail-restore = RESTORE
config-pending-diff-rail-current = CURRENT
config-pending-diff-rail-by = by
config-pending-diff-rail-baseline-2 = BASELINE
config-pending-diff-rail-pin-as-baseline = PIN AS BASELINE
config-pending-diff-rail-revert = REVERT
config-pending-diff-rail-stage-apply = STAGE & APPLY
config-pending-diff-rail-written-atomically-temp-file-then = Written atomically: temp file, then rename
config-pending-diff-rail-structured-edits-keep-comments-and = Structured edits keep comments and layout
config-pending-diff-rail-no-host-validator-for-this = No host validator for this format
config-pending-diff-rail-checked-on-the-host-first = Checked on the host first: { $cmds }
config-pending-diff-rail-plain-text-nothing-checks-it = Plain text: nothing checks it before it's written
config-pending-diff-rail-not-kept-in-history-crow = Not kept in history: Crow didn't read this file from the host
config-pending-diff-rail-every-version-kept-in-history = Every version kept in history, encrypted
config-pending-diff-rail-history-keeps-hashes-only-no = History keeps hashes only: no encryption key available
config-pending-diff-rail-history-not-loaded-yet = History not loaded yet
config-pending-diff-rail-none-pinned-pin-as-baseline = None pinned. PIN AS BASELINE on a revision makes it the known-good { $filename } for { $scope_label }; servers that differ show as drift.
config-pending-diff-rail-matches-the-baseline = Matches the baseline
config-pending-diff-rail-same-settings-as-the-baseline = Same settings as the baseline; only comments or layout differ
config-pending-diff-rail-drifted-from-the-baseline = Drifted from the baseline
config-pending-diff-rail-more = … { $more } more
config-pending-diff-rail-pinned-from-for-by = pinned from { $from_server } for { $scope_label } by { $set_by }
config-pending-diff-rail-load-baseline-into-editor = LOAD BASELINE INTO EDITOR
config-pending-diff-rail-unpin = UNPIN

## config-raw-editor
config-raw-editor-traditional-editor = TRADITIONAL EDITOR
config-raw-editor-edited = EDITED (+{ $add_count } -{ $del_count })
config-raw-editor-v-baseline = v{ $active_revision } (BASELINE)
config-raw-editor-author = Author:
config-raw-editor-history = HISTORY · { $count }
config-raw-editor-structured-view = STRUCTURED VIEW
config-raw-editor-revert-baseline = REVERT BASELINE
config-raw-editor-stage-version = STAGE VERSION
config-raw-editor-lines = { $line_count } lines
config-raw-editor-bytes = { $count } bytes
config-raw-editor-utf-8 = UTF-8
config-raw-editor-unix-lf = UNIX (LF)
config-raw-editor-revisions = Revisions: { $count }
config-raw-editor-read-only = READ-ONLY
config-raw-editor-unsaved-staged-changes = ● Unsaved staged changes
config-raw-editor-clean-baseline = ✓ Clean baseline

## config-sshd-sheet
config-sshd-sheet-some-files-this-config-includes = Some files this config includes couldn't be read, so values below may not be what sshd uses:
config-sshd-sheet-other-directives = OTHER DIRECTIVES
config-sshd-sheet-match-in = MATCH { $scope } · IN { $path }
config-sshd-sheet-match-applies-only-when-this = MATCH { $scope } · APPLIES ONLY WHEN THIS MATCHES
config-sshd-sheet-nothing-to-fix = nothing to fix
config-sshd-sheet-1-to-fix = 1 to fix
config-sshd-sheet-to-fix = { $n } to fix
config-sshd-sheet-security-posture = SECURITY POSTURE
config-sshd-sheet-fine = { $count } fine
config-sshd-sheet-set-in = set in { $src }
config-sshd-sheet-sshds-default = sshd's default
config-sshd-sheet-set = SET { $v }
config-sshd-sheet-turn-off-safely = TURN OFF SAFELY…
config-sshd-sheet-not-set = not set
config-sshd-sheet-line = line { $l }
config-sshd-sheet-ignored-sets-it-first = { $at } · ignored: { $by } sets it first
config-sshd-sheet-ignored-set-earlier = { $at } · ignored, set earlier
config-sshd-sheet-back-to-sshds-default-removes = Back to sshd's default (removes the line)
config-sshd-sheet-remove-the-line = Remove the line
config-sshd-sheet-default = default

## config-state
config-state-differs-from-the-baseline-only = differs from the baseline (only its hash was kept)
config-state-was-not-found-on-this = { $file } was not found on this server
config-state-new-job = New job

## config-structured-editor
config-structured-editor-recommended = RECOMMENDED
config-structured-editor-caution = CAUTION
config-structured-editor-weak = WEAK
config-structured-editor-never-on-prod = NEVER ON PROD
config-structured-editor-deny = DENY
config-structured-editor-lets-run = lets { $field } run { $risk }
config-structured-editor-crow-ui = CROW UI · { $plugin_name } · { $kind }
config-structured-editor-edited = EDITED (+{ $add_count } -{ $del_count })
config-structured-editor-v-baseline = v{ $active_revision } (BASELINE)
config-structured-editor-text-view = TEXT VIEW
config-structured-editor-history = HISTORY
config-structured-editor-revert = REVERT
config-structured-editor-stage-version = STAGE VERSION
config-structured-editor-rows-are-evaluated-top-to = rows are evaluated top to bottom
config-structured-editor-1-value-here-isnt = 1 value here isn't
config-structured-editor-values-here-arent = { $not_running } values here aren't
config-structured-editor-not-running-yet-what-the = NOT RUNNING YET · { $what } what the kernel runs now; they apply at boot or with sysctl --system
config-structured-editor-managed-by-it-rewrites-this = MANAGED BY { $m } · it rewrites this file, so edits here get overwritten; change the DNS settings in { $m2 } instead
config-structured-editor-read-only = READ-ONLY · { $r }
config-structured-editor-edit-rejected = EDIT REJECTED · { $e }
config-structured-editor-line = LINE
config-structured-editor-add-directive = + ADD DIRECTIVE
config-structured-editor-no-entries-only-comments-or = No entries — only comments or blank lines.
config-structured-editor-move-up-rules-are-read = Move up: rules are read top to bottom
config-structured-editor-scoped-to-match-read-only = Scoped to Match { $s } — read-only until crow-config models Match blocks (ERR-12)
config-structured-editor-add-row = + ADD ROW
config-structured-editor-add = ADD
config-structured-editor-added-at-the-end-of = Added at the end of this block. New blocks are typed in the text view.
config-structured-editor-added-at-the-end-of-2 = Added at the end of the file.
config-structured-editor-added-at-the-end-of-3 = Added at the end of this section. New sections are typed in the text view.
config-structured-editor-this-change-introduces-values-rated = This change introduces values rated NEVER ON PROD:
config-structured-editor-type-to-stage-it = Type { $risk_confirm_keyword } to stage it:
config-structured-editor-stage-anyway = STAGE ANYWAY
config-structured-editor-cancel = CANCEL
config-structured-editor-order-matters = ORDER MATTERS · { $ir }

## containers
containers-running = Running: { $b }…
containers-reading-the-servers-containers = Reading the server's containers…
containers-couldnt-read-the-server = Couldn't read the server: { $e }
containers-containers = CONTAINERS · { $server_name }
containers-running-2 = { $running }/{ $count } RUNNING
containers-reading = READING…
containers-refresh = REFRESH
containers-fix-it-on-the-server = Fix it on the server
containers-neither-docker-nor-podman-is = Neither Docker nor Podman is installed on this server.
containers-no-container-runtime-crow-can = No container runtime Crow can use here.
containers-no-containers-on-this-server = No containers on this server.
containers-standalone = STANDALONE
containers-not-in-a-compose-stack = { $count } not in a compose stack
containers-up = UP
containers-restart = RESTART
containers-down = DOWN
containers-running-3 = { $running }/{ $count } running
containers-create-and-start-every-service = Create and start every service in this stack (it may pull images)?
containers-restart-every-container-in-this = Restart every container in this stack? Each is down for a moment.
containers-stop-and-remove-this-stacks = Stop and remove this stack's containers and networks? Named volumes are kept.
containers-cancel = CANCEL
containers-name = NAME
containers-image = IMAGE
containers-state = STATE
containers-published-ports = PUBLISHED PORTS
containers-restarts = RESTARTS
containers-runtime = RUNTIME
containers-all-nets = ALL NETS
containers-root = {" "}· root
containers-no-output-yet = No output yet.
containers-reading-logs = Reading logs…
containers-reading-2 = Reading…
containers-start = START
containers-stop = STOP
containers-start-this-container = Start this container?
containers-stop-this-container-what-it = Stop this container? What it serves goes down until it's started again.
containers-restart-this-container-its-down = Restart this container? It's down for a moment.
containers-logs = LOGS
containers-inspect = INSPECT
containers-newest-first = newest first
containers-following = FOLLOWING
containers-follow = FOLLOW
containers-environment-values-hidden = environment values hidden

## files-collector
files-collector-binary-file-nothing-to-show = binary file: nothing to show as text
files-collector-invalid-folder-name = Invalid folder name

## files-models
files-models-kb = { $size_bytes } KB
files-models-mb = { $size_bytes } MB
files-models-gb = { $size_bytes } GB

## files
files-files = FILES
files-up = UP
files-folder = + FOLDER
files-name = NAME
files-size = SIZE
files-perms = PERMS
files-owner = OWNER
files-group = GROUP
files-modified = MODIFIED
files-empty-directory = Empty directory
files-create = CREATE ⏎
files-cancel = cancel
files-edit-in-a-terminal-nano = Edit in a terminal (nano)
files-delete-empty-directory-this-cannot = Delete empty directory { $del_name }? This cannot be undone.
files-delete-this-cannot-be-undone = Delete { $del_name }? This cannot be undone.
files-cancel-2 = Cancel
files-delete = DELETE ⏎
files-others = OTHERS
files-everyone-else = everyone else
files-setuid = SETUID
files-runs-as-its-owner = runs as its owner
files-setgid = SETGID
files-new-files-get-this-group = new files get this group
files-runs-as-its-group = runs as its group
files-sticky = STICKY
files-only-owners-delete-inside = only owners delete inside
files-anyone-on-the-server-can = anyone on the server can change this file
files-applying = applying…
files-chmod = chmod { $mode }
files-reading = reading…
files-first-kb-lines = first { $collector } KB · { $lines } lines
files-text = { $lines } { $lines ->
        [one] line
       *[other] lines
    }
files-edit-in-terminal = EDIT IN TERMINAL
files-reading-the-file = Reading the file…

## firewall-commands
firewall-commands-service-ingress = { $label } Service Ingress
firewall-commands-is-not-a-port-range = { $port } is not a port, range (a:b) or list (a,b)
firewall-commands-is-not-an-address-or = { $source } is not an address or network
firewall-commands-forward-rules-are-managed-with = forward rules are managed with `ufw route`, which Crow doesn't write yet
firewall-commands-this-rule-isnt-from-a = this rule isn't from a firewalld zone
firewall-commands-zone-accepts-everything-through-its = zone { $name } accepts everything through its target, not a rule; change it with firewall-cmd --permanent --zone={ $name } --set-target=default
firewall-commands-crow-doesnt-know-which-zone = Crow doesn't know which zone entry this rule came from
firewall-commands-firewalld-zones-filter-incoming-traffic = firewalld zones filter incoming traffic; outgoing and forwarded traffic need a policy, which Crow doesn't write yet
firewall-commands-is-not-an-address-or-2 = { $s } is not an address or network
firewall-commands-accept-limit-value-12-m = accept limit value="12/m"
firewall-commands-give-a-port-or-a = give a port, or a source to apply this to everything from
firewall-commands-rule-family-source-address = rule family="{ $s }" source address="{ $s2 }"
firewall-commands-add-rich-rule-port-port = --add-rich-rule={ $head } port port="{ $p }" protocol="{ $proto }" { $verdict }

## firewall-detector
firewall-detector-ufw-is-active-but-its = ufw is active but its status output could not be parsed.
firewall-detector-ufw-is-installed-but-inactive = UFW is installed but inactive. System netfilter packet filtering is disabled.
firewall-detector-ufw-is-installed-but-crow = ufw is installed, but Crow couldn't read its rules as root: { $e }
firewall-detector-firewalld-is-installed-but-not = firewalld is installed but not running, so it filters nothing.
firewall-detector-firewalld-is-running-but-crow = firewalld is running, but Crow couldn't read its zones as root: { $e }
firewall-detector-nftables-iptables-are-here-but = nftables/iptables are here, but Crow couldn't read the ruleset as root: { $e }
firewall-detector-no-firewall-tooling-ufw-firewalld = No firewall tooling (ufw, firewalld, nftables, iptables) was found on this host.
firewall-detector-anywhere = Anywhere

## firewalld
firewalld-not = not { $a }
firewalld-anywhere = Anywhere
firewalld-from = from { $z }
firewalld-on = on { $z }
firewalld-every-interface-not-in-another = every interface not in another zone
firewalld-zone-accepts-all-traffic = zone { $name } accepts all traffic ({ $binding })
firewalld-service-ports-not-known-zone = service { $svc } (ports not known) · zone { $name }
firewalld-service-zone = service { $svc } · zone { $name }
firewalld-port-zone = port · zone { $name }
firewalld-rich-rule-zone = rich rule · zone { $name }
firewalld-rich-rule-service-zone = rich rule, service { $svc } · zone { $name }
firewalld-rich-rule-not-interpreted = rich rule (not interpreted): { $raw_rule }
firewalld-accept = ACCEPT
firewalld-drop = DROP
firewalld-the-running-firewall-differs-from = The running firewall differs from its saved (permanent) config: a reload or reboot will drop the runtime-only changes.

## firewall-lockout
firewall-lockout-crow-cant-see-which-rules = Crow can't see which rules will apply after this, so it can't check that port { $port }/tcp, the SSH port it's connected through, stays allowed.
firewall-lockout-after-this-port-tcp-crows = After this, port { $port }/tcp (Crow's SSH connection) is only allowed from { $value }. If Crow doesn't connect from there, the next login is refused.
firewall-lockout-this-stops-port-tcp-the = This stops port { $port }/tcp, the SSH port Crow is connected through, from being allowed. The next SSH login, Crow's included, would be refused.
firewall-lockout-anywhere = Anywhere

## firewall-models
firewall-models-ufw-uncomplicated-firewall = UFW (Uncomplicated Firewall)
firewall-models-unmanaged-none-detected = Unmanaged / None Detected
firewall-models-ufw = UFW
firewall-models-firewalld = FIREWALLD
firewall-models-nftables = NFTABLES
firewall-models-iptables = IPTABLES
firewall-models-none = NONE
firewall-models-in = IN
firewall-models-out = OUT
firewall-models-fwd = FWD
firewall-models-allow = ALLOW
firewall-models-deny = DENY
firewall-models-reject = REJECT
firewall-models-limit = LIMIT
firewall-models-tcp = TCP
firewall-models-udp = UDP
firewall-models-tcp-udp = TCP+UDP
firewall-models-any = ANY
firewall-models-tcp-udp-2 = { $port }/tcp,udp
firewall-models-firewall-allowed = FIREWALL: ALLOWED
firewall-models-firewall-blocked = FIREWALL: BLOCKED
firewall-models-firewall-allow-default = FIREWALL: ALLOW (DEFAULT)
firewall-models-firewall-no-inbound-rule = FIREWALL: NO INBOUND RULE
firewall-models-firewall-inactive = FIREWALL: INACTIVE
firewall-models-firewall-not-read = FIREWALL: NOT READ
firewall-models-openssh-secure-shell = OpenSSH Secure Shell
firewall-models-web-server-http = Web Server (HTTP)
firewall-models-secure-web-server-https = Secure Web Server (HTTPS)
firewall-models-postgresql-db-cluster = PostgreSQL DB Cluster
firewall-models-redis-key-value-cache = Redis Key-Value Cache
firewall-models-docker-cluster-management = Docker Cluster Management

## firewall
firewall-firewall-is-currently-disabled-by = Firewall is currently disabled by administrator.
firewall-all = ALL
firewall-allow = ALLOW
firewall-deny = DENY
firewall-limit = LIMIT
firewall-firewall = FIREWALL
firewall-on = { $short_name } · on · { $total_rules } { $total_rules ->
        [one] rule
       *[other] rules
    }
firewall-stage = STAGE +{ $add_count } −{ $del_count }
firewall-staged-firewall-rule-changes = Staged firewall rule changes
firewall-audit = AUDIT
firewall-rule-table = RULE TABLE
firewall-raw-ruleset = RAW RULESET
firewall-reload = RELOAD
firewall-disable = DISABLE
firewall-add-rule = + ADD RULE
firewall-defaults = DEFAULTS
firewall-quick-open = QUICK OPEN
firewall-action = ACTION
firewall-dir = DIR
firewall-port = PORT
firewall-from = FROM
firewall-to = TO
firewall-comment = COMMENT
firewall-no-rules-yet-everything-follows = No rules yet: everything follows the defaults above
firewall-no-rules-match-try-all = No rules match: try ALL, or clear the search
firewall-running = Running: { $p } …
firewall-read-only-crow-reads-the = Read-only: Crow reads the { $label } ruleset (the input chain and what it jumps to) but never writes raw rulesets.
firewall-read-only-crow-reads-the-2 = Read-only: Crow reads { $label } (the running config) but doesn't change it yet; use its own tools to edit.
firewall-changes-go-to-firewallds-running = Changes go to firewalld's running config and its saved (permanent) config together, so they apply now and survive a reload.
firewall-this-could-lock-you-out = THIS COULD LOCK YOU OUT
firewall-type-confirm-to-apply-it = Type CONFIRM to apply it anyway:
firewall-apply-anyway = APPLY ANYWAY
firewall-cancel = CANCEL

## firewall-new-rule-modal
firewall-new-rule-modal-anywhere = Anywhere
firewall-new-rule-modal-proto-tcp = proto tcp{" "}
firewall-new-rule-modal-proto-udp = proto udp{" "}
firewall-new-rule-modal-from = from { $source_input }{" "}
firewall-new-rule-modal-to-any-port = to any port { $port_clean }
firewall-new-rule-modal-comment = {" "}comment '{ $comment_input }'
firewall-new-rule-modal-ufw = ufw { $action_str } { $proto_str }{ $from_str }{ $port_str }{ $comment_str }
firewall-new-rule-modal-will-run-running-saved-config = WILL RUN (running + saved config)
firewall-new-rule-modal-not-valid-for-firewalld = NOT VALID FOR FIREWALLD
firewall-new-rule-modal-will-run = WILL RUN
firewall-new-rule-modal-add-rule = ADD RULE
firewall-new-rule-modal-action = ACTION
firewall-new-rule-modal-allow = ALLOW
firewall-new-rule-modal-deny = DENY
firewall-new-rule-modal-reject = REJECT
firewall-new-rule-modal-limit = LIMIT
firewall-new-rule-modal-protocol = PROTOCOL
firewall-new-rule-modal-tcp = TCP
firewall-new-rule-modal-udp = UDP
firewall-new-rule-modal-tcp-udp = TCP+UDP
firewall-new-rule-modal-any = ANY
firewall-new-rule-modal-port = PORT
firewall-new-rule-modal-from-2 = FROM
firewall-new-rule-modal-comment-2 = COMMENT
firewall-new-rule-modal-enter-adds-it-esc-or = Enter adds it · Esc or a click outside cancels
firewall-new-rule-modal-cancel = CANCEL

## firewall-non-operational
firewall-non-operational-no-firewall-running = NO FIREWALL RUNNING
firewall-non-operational-why = WHY
firewall-non-operational-firewall-tools-on-the-host = FIREWALL TOOLS ON THE HOST
firewall-non-operational-no-standard-packet-filter-binaries = No standard packet filter binaries found (ufw, firewalld, iptables)
firewall-non-operational-to-turn-it-on = TO TURN IT ON:
firewall-non-operational-to-set-one-up-ufw = TO SET ONE UP (ufw):
firewall-non-operational-turn-the-firewall-on = TURN THE FIREWALL ON

## firewall-raw
firewall-raw-on = on { $i }
firewall-raw-connections = { $ct } connections
firewall-raw-not = not { $right }
firewall-raw-iptables-match = iptables match { $body }
firewall-raw-nfts-json-couldnt-be-read = nft's JSON couldn't be read: { $e }
firewall-raw-nfts-json-has-no-nftables = nft's JSON has no "nftables" list
firewall-raw-not-2 = not { $arg }
firewall-raw-ipv6 = IPv6
firewall-raw-ipv4 = IPv4
firewall-raw-chain = chain { $path }
firewall-raw-anywhere = Anywhere
firewall-raw-no-chain-is-hooked-on = No chain is hooked on input: nothing filters incoming traffic.
firewall-raw-only-ipv4-is-filtered-no = Only IPv4 is filtered: no input chain covers IPv6, so every port is open over IPv6.
firewall-raw-input-chains-run-one-after = { $count } input chains run one after another ({ $stacked }); a packet any of them drops is dropped.
firewall-raw-ip6tables-save =
    { $v4 }
    # ---- ip6tables-save ----
    { $v6 }
firewall-raw-no-nft-or-iptables-to = no nft or iptables to read

## firewall-rules-format
firewall-rules-format-etc-ufw-user-rules-managed =
    # /etc/ufw/user.rules - Managed by Crow Unified Engine
    {""}
firewall-rules-format-auditable-reversible-source-controlled-firewall =
    # Auditable, reversible, source-controlled firewall rules.
    {""}
firewall-rules-format-ufw-user-input-0-0 =
    :ufw-user-input - [0:0]
    {""}
firewall-rules-format-ufw-user-output-0-0 =
    :ufw-user-output - [0:0]
    {""}
firewall-rules-format-ufw-user-forward-0-0 =
    :ufw-user-forward - [0:0]
    {""}
    {""}
firewall-rules-format-rules =
    ### RULES ###
    {""}
firewall-rules-format-p-tcp = {" "}-p tcp
firewall-rules-format-p-udp = {" "}-p udp
firewall-rules-format-p-tcp-udp = {" "}-p tcp,udp
firewall-rules-format-j-accept = {" "}-j ACCEPT
firewall-rules-format-j-drop = {" "}-j DROP
firewall-rules-format-j-reject = {" "}-j REJECT
firewall-rules-format-m-limit-limit-3-minute = {" "}-m limit --limit 3/minute -j ACCEPT
firewall-rules-format-m-comment-comment = {" "}-m comment --comment "{ $clean_comment }"
firewall-rules-format-m-comment-comment-rule = {" "}-m comment --comment "rule_{ $idx }"
firewall-rules-format-commit =
    {""}
    COMMIT
    {""}
firewall-rules-format-anywhere = Anywhere

## fleet-archived
fleet-archived-active-fleet = ACTIVE FLEET ({ $active_n })
fleet-archived-archived = ARCHIVED ({ $archived_n })
fleet-archived-danger-zone = DANGER ZONE
fleet-archived-host = HOST
fleet-archived-address = ADDRESS
fleet-archived-env = ENV
fleet-archived-role = ROLE
fleet-archived-archived-2 = ARCHIVED
fleet-archived-stored-data = STORED DATA
fleet-archived-restore = RESTORE
fleet-archived-no-archived-servers = NO ARCHIVED SERVERS
fleet-archived-archiving-takes-a-server-out = Archiving takes a server out of the fleet without deleting it. Nothing is ever deleted outright.
fleet-archived-purge-log = PURGE LOG
fleet-archived-record-s-crows-own-audit = { $count } record(s) · Crow's own audit trail
fleet-archived-nothing-has-been-purged-yet = Nothing has been purged yet.
fleet-archived-archive = ARCHIVE { $name }
fleet-archived-leaves-the-active-fleet-crow = { $name } leaves the active fleet. Crow stops polling it and closes its SSH connection.
fleet-archived-its-stored-data-change-history = Its stored data — change history and key attachments — is deleted on { $purge_due }.
fleet-archived-the-server-record-itself-is = The server record itself is never deleted: address, port, user, key, jump host, environment, role, group, tags, host-key fingerprint and OS facts all stay. It can be restored from the Archived tab until the purge runs, and after that it returns without its history.
fleet-archived-cancel = CANCEL
fleet-archived-archive-server = ARCHIVE SERVER

## fleet-certificates
fleet-certificates-certificates = CERTIFICATES
fleet-certificates-served-expiring-within-days-not = { $total } served · { $soon } expiring within { $cert_warn_days } days · { $broken } { $broken ->
        [one] server
       *[other] servers
    } not renewing on its own
fleet-certificates-read-again = READ AGAIN
fleet-certificates-reading = READING { $count }…
fleet-certificates-renew-runs-certbots-dry-run = RENEW runs certbot's dry run first, then the renewal, then checks and reloads nginx or Apache (never a broken config), and reads the certificate again to confirm its new expiry. certbot only renews within 30 days of expiry; Let's Encrypt limits how often a certificate can be issued.
fleet-certificates-no-renewal-tool = no renewal tool
fleet-certificates-couldnt-read = couldn't read: { $e }
fleet-certificates-reading-2 = reading…
fleet-certificates-renews-on-its-own = renews on its own
fleet-certificates-no-certificates-found-lets-encrypts = No certificates found (Let's Encrypt's live folder, nginx and Apache configs).
fleet-certificates-no-name = (no name)
fleet-certificates-expired-days-ago = expired { $days } days ago
fleet-certificates-days-left = { $days } days left
fleet-certificates-renew = RENEW
fleet-certificates-not-certbot-acme-sh = not certbot/acme.sh
fleet-certificates-nothing-read-yet-read-again = Nothing read yet: READ AGAIN reads every reachable server.

## fleet-checks
fleet-checks-checks-from-outside = CHECKS FROM OUTSIDE
fleet-checks-checks-failing = { $count } checks · { $failing } failing
fleet-checks-run-from-this-machine-only = run from this machine, only while Crow is open
fleet-checks-http-s = HTTP(S)
fleet-checks-tcp-port = TCP PORT
fleet-checks-dns-name = DNS NAME
fleet-checks-every = EVERY
fleet-checks-1-min = 1 min
fleet-checks-5-min = 5 min
fleet-checks-attach-to = ATTACH TO
fleet-checks-no-server = no server
fleet-checks-add-check = ADD CHECK
fleet-checks-an-attached-checks-alerts-land = An attached check's alerts land on that server (its page, the map, email). A check alerts after two failures in a row and resolves on the first success.
fleet-checks-on = {" "}· on { $s }
fleet-checks-cert-days = {" "}· cert { $d } days
fleet-checks-checking = checking…
fleet-checks-not-checked-yet = not checked yet
fleet-checks-alerts-on = ALERTS ON
fleet-checks-alerts-off = ALERTS OFF
fleet-checks-no-checks-yet-add-a = No checks yet: add a website, a port or a DNS name above.
fleet-checks-every-s = { $target } · every { $every_secs } s{ $server }

## fleet-drift
fleet-drift-not-yet = not yet
fleet-drift-just-now = just now
fleet-drift-min-ago = { $s } min ago
fleet-drift-h-ago = { $s } h ago
fleet-drift-search-every-servers-configs = SEARCH EVERY SERVER'S CONFIGS
fleet-drift-from-what-crow-has-read = from what Crow has read; no server is contacted
fleet-drift-nothing-found-crow-searches-the = Nothing found. Crow searches the files it has read: open a server's Config screen to read its files.
fleet-drift-first-matches = first { $n } matches
fleet-drift-text = { $n } { $n ->
        [one] match
       *[other] matches
    }
fleet-drift-the-file = the file
fleet-drift-drift-from-baselines = DRIFT FROM BASELINES
fleet-drift-drifted-files-re-read-every = { $drifted } drifted · files re-read every 15 min while Crow runs · checked { $ago }
fleet-drift-check-now = CHECK NOW
fleet-drift-reading = READING { $count }…
fleet-drift-no-baselines-pinned-yet-pin = No baselines pinned yet. Pin one from a server's Config screen (a file's history → PIN AS BASELINE); every server in that group, or the fleet, is then compared with it.
fleet-drift-drifted = { $n_drift } drifted
fleet-drift-all-match = all match
fleet-drift-bring-all-back = BRING ALL { $n_drift } BACK
fleet-drift-matches-comments-spacing-differ = matches (comments/spacing differ)
fleet-drift-text-2 = { $count } { $count ->
        [one] difference
       *[other] differences
    }: { $cloned }
fleet-drift-vs-baseline = vs { $scope } baseline
fleet-drift-what-changed = WHAT CHANGED
fleet-drift-baseline-vs-server = { $name } · { $path } · baseline (−) vs server (+)
fleet-drift-crow-kept-only-hashes-of = Crow kept only hashes of one of the two: file contents are sealed with the vault's key, and there was none when it was read (or the vault is locked now). It can tell that they differ, not how, and can't push the baseline or pin this copy. Set up the vault (Settings → Vault & Security), then CHECK NOW.
fleet-drift-bring-back-writes-the-baseline = Bring back writes the baseline over this server's copy (checked by its validator, kept in history). Accept makes this copy the baseline: every other server in its scope is then compared with it.

## fleet-dr
fleet-dr-plans = PLANS
fleet-dr-new = + NEW
fleet-dr-no-plans-yet-a-plan = No plans yet. A plan says what matters on a server, where its backups go, how fast it must come back, and how to rebuild it.
fleet-dr-drill-min = {" "}· drill { $minutes } min{ $d }
fleet-dr-name = NAME
fleet-dr-rto-min = RTO (MIN)
fleet-dr-rpo-min = RPO (MIN)
fleet-dr-for = FOR
fleet-dr-group = group { $g }
fleet-dr-what-matters = WHAT MATTERS
fleet-dr-backups-path-or-glob-timer = BACKUPS (PATH OR GLOB | TIMER)
fleet-dr-runbook-markdown-with-commands = RUNBOOK (MARKDOWN, WITH COMMANDS)
fleet-dr-save = SAVE
fleet-dr-checking = CHECKING…
fleet-dr-check-against-the-servers = CHECK AGAINST THE SERVERS
fleet-dr-export-md = EXPORT (.md)
fleet-dr-delete-plan = DELETE PLAN
fleet-dr-drills = DRILLS
fleet-dr-rebuild-it-for-real-somewhere = Rebuild it for real somewhere harmless (a Multipass VM, a scratch server), following the runbook; Crow times it against the RTO.
fleet-dr-running-for-min = running for { $chrono } min
fleet-dr-finish-drill = FINISH DRILL
fleet-dr-start-a-drill = START A DRILL
fleet-dr-met-the-rto = met the RTO
fleet-dr-missed-the-rto = missed the RTO
fleet-dr-min = { $when } · { $minutes } min · { $d }{ $d2 }

## fleet-group-modal
fleet-group-modal-server = Server
fleet-group-modal-group = GROUP · { $server_name }
fleet-group-modal-move-this-server-to-a = Move this server to a group:
fleet-group-modal-no-group = no group

## fleet-hardening
fleet-hardening-not-yet = not yet
fleet-hardening-min-ago = { $chrono } min ago
fleet-hardening-posture = POSTURE
fleet-hardening-open-critical-accepted = { $open } open ({ $crit } critical) · { $accepted } accepted
fleet-hardening-read-as-root-where-crow = read { $checked }, as root where Crow can (sshd -T, the firewall's status, ss)
fleet-hardening-reading = READING…
fleet-hardening-check-now = CHECK NOW
fleet-hardening-export-report-md = EXPORT REPORT (.md)
fleet-hardening-reason-to-accept = REASON TO ACCEPT
fleet-hardening-fix-everywhere = fix everywhere:
fleet-hardening-every-fix-first-proves-a = Every fix first proves a fresh key login works, then changes one thing, checks it took (sshd -t, sshd -T, the firewall's status), and proves a fresh login still works. Anything that fails is undone. Crow never sets PermitRootLogin no where it logs in as root, and a firewall always allows the SSH port.
fleet-hardening-crow-logs-in-as = Crow logs in as { $user }
fleet-hardening-not-read-unreachable-or-not = not read (unreachable, or not checked yet)
fleet-hardening-nothing-found = nothing found
fleet-hardening-text = { $count } { $count ->
        [one] finding
       *[other] findings
    }
fleet-hardening-accepted-by = accepted by { $by }: { $reason }
fleet-hardening-fix = fix: { $f }

## fleet-import
fleet-import-import-from-providers = IMPORT FROM PROVIDERS
fleet-import-asking-account-s = asking { $loading } account(s)…
fleet-import-instance-s-not-in-the = { $count } instance(s) · { $importable } not in the fleet
fleet-import-instances-already-in-the-fleet = Instances already in the fleet are matched by IP and linked, so their region comes from the provider. IMPORT opens Add Server filled in: the host key and login are checked as usual.
fleet-import-linked-enrolled-server-s-to = Linked { $linked } enrolled server(s) to their instances.
fleet-import-no-address = no address
fleet-import-in-fleet-as = in fleet as { $server }
fleet-import-import = IMPORT →

## fleet-incidents
fleet-incidents-from-an-alert-last-7 = FROM AN ALERT (LAST 7 DAYS)
fleet-incidents-no-alerts-in-the-last = No alerts in the last 7 days.
fleet-incidents-resolved = {" "}· resolved
fleet-incidents-or-a-window-on-a = OR A WINDOW ON A SERVER
fleet-incidents-pick-an-alert-or-a = Pick an alert or a window: the timeline merges the alerts, Crow's own changes, failed checks and error logs (err and worse) around it.
fleet-incidents-reading-logs = {" "}· reading logs…
fleet-incidents-summarizing = SUMMARIZING…
fleet-incidents-summarize-clanker = SUMMARIZE (CLANKER)
fleet-incidents-export-md = EXPORT (.md)
fleet-incidents-via-a-likely-sequence-an = via { $via } · a likely sequence, an AI's reading: check it against the timeline
fleet-incidents-nothing-in-this-window = Nothing in this window.
fleet-incidents-events = { $when } → { $when2 } · { $count } events{ $st }

## fleet-lab-modal
fleet-lab-modal-local-vms-containers = LOCAL VMS & CONTAINERS
fleet-lab-modal-multipass-vms-and-lab-containers = Multipass VMs and lab containers on this machine, enrolled into Crow
fleet-lab-modal-multipass-vms = MULTIPASS VMS
fleet-lab-modal-ubuntu-vms-on-this-machine = Ubuntu VMs on this machine: launch, import and power them
fleet-lab-modal-local-lab-containers = LOCAL LAB CONTAINERS
fleet-lab-modal-lab-containers-on-this-machine = Lab containers on this machine, enrolled into Crow as test nodes
fleet-lab-modal-checking = checking…
fleet-lab-modal-not-installed = not installed
fleet-lab-modal-discovered-local-test-nodes = DISCOVERED LOCAL TEST NODES
fleet-lab-modal-detected = { $count } detected
fleet-lab-modal-refresh = REFRESH 🔄
fleet-lab-modal-no-local-test-nodes-found = No local test nodes found
fleet-lab-modal-use-the-form-below-to = Use the form below to spin up a fresh Ubuntu or Debian test node
fleet-lab-modal-close = CLOSE
fleet-lab-modal-endpoint = { $distro_display } · Endpoint: { $endpoint_display }
fleet-lab-modal-stop = ■ STOP
fleet-lab-modal-start = ▶ START
fleet-lab-modal-enrolled = ✓ ENROLLED
fleet-lab-modal-enroll-in-crow = + ENROLL IN CROW
fleet-lab-modal-checking-2 = CHECKING…
fleet-lab-modal-not-installed-2 = NOT INSTALLED
fleet-lab-modal-daemon-not-running = DAEMON NOT RUNNING
fleet-lab-modal-needs-a-fix = { $daemon } · NEEDS A FIX
fleet-lab-modal-ready = READY · { $daemon }
fleet-lab-modal-ubuntu-vms-on-this-machine-2 = Ubuntu VMs on this machine. Crow launches them with its own key, pins their host keys through Multipass, and adds them to the fleet as SSH servers.
fleet-lab-modal-no-vms-yet-launch-one = No VMs yet: launch one below.
fleet-lab-modal-copy = COPY
fleet-lab-modal-command-copied = Command copied
fleet-lab-modal-in-fleet = IN FLEET
fleet-lab-modal-import-to-crow = IMPORT TO CROW
fleet-lab-modal-no-address = no address
fleet-lab-modal-its-fleet-entry-goes-to = {" "}Its fleet entry goes to the archive.
fleet-lab-modal-delete-for-good = DELETE FOR GOOD
fleet-lab-modal-cancel = CANCEL
fleet-lab-modal-launch-a-vm = LAUNCH A VM
fleet-lab-modal-cpu = CPU
fleet-lab-modal-memory = MEMORY
fleet-lab-modal-gb = { $m } GB
fleet-lab-modal-disk = DISK
fleet-lab-modal-gb-2 = { $d } GB
fleet-lab-modal-newest-lts = Newest LTS
fleet-lab-modal-launch-add-to-fleet = LAUNCH & ADD TO FLEET
fleet-lab-modal-delete-and-its-disk-for = Delete { $name } and its disk for good?{ $in_fleet }

## fleet-lab-state
fleet-lab-state-starting = Starting { $name }…
fleet-lab-state-stopping = Stopping { $name }…
fleet-lab-state-restarting = Restarting { $name }…
fleet-lab-state-suspending = Suspending { $name }…
fleet-lab-state-deleting-and-its-disk = Deleting { $name } and its disk…

## fleet-logsearch
fleet-logsearch-search = SEARCH
fleet-logsearch-priority = PRIORITY
fleet-logsearch-when = WHEN
fleet-logsearch-at-most = AT MOST
fleet-logsearch-where = WHERE
fleet-logsearch-every-server = every server
fleet-logsearch-group = group { $g }
fleet-logsearch-search-2 = SEARCH ⏎
fleet-logsearch-searching = SEARCHING { $count }…
fleet-logsearch-save-as = SAVE AS
fleet-logsearch-save = SAVE
fleet-logsearch-saved = SAVED
fleet-logsearch-alert = ALERT { $level }
fleet-logsearch-alert-off = ALERT OFF
fleet-logsearch-in = {" "}in { $unit }
fleet-logsearch-prio = {" "}· prio ≤{ $p }
fleet-logsearch-group-2 = group { $group }
fleet-logsearch-an-alerting-search-runs-every = An alerting search runs every 10 minutes over the last 15, on its servers, while Crow is open; the alert resolves once it stops matching (and is mailed with the Email plugin).
fleet-logsearch-lines = { $count } LINES
fleet-logsearch-limit = {" "}(limit)
fleet-logsearch-summarizing = SUMMARIZING…
fleet-logsearch-summarize-clanker = SUMMARIZE (CLANKER)
fleet-logsearch-via-only-the-matching-lines = via { $via } · only the matching lines were sent · an AI's reading: check it

## fleet-overview
fleet-overview-none-enrolled = none enrolled
fleet-overview-1-group = 1 group
fleet-overview-groups = { $n } groups
fleet-overview-1-region = 1 region · { $group_note }
fleet-overview-regions = { $n } regions · { $group_note }
fleet-overview-avg-1m-load-vcpu = avg 1m load · { $total_vcpu } vCPU
fleet-overview-no-metrics-yet = no metrics yet
fleet-overview-not-read-yet = not read yet
fleet-overview-past-the-rotation-policy = { $overdue } past the rotation policy · { $note }
fleet-overview-not-read-yet-2 = { $note } · { $unknown } not read yet
fleet-overview-not-computed-yet = not computed yet
fleet-overview-no-baselines-pinned = no baselines pinned
fleet-overview-match-baselines-as-last-read = { $count } { $count ->
        [one] file
       *[other] files
    } match baselines · as last read
fleet-overview-on-as-last-read = on { $count } { $count ->
        [one] server
       *[other] servers
    } · as last read
fleet-overview-servers = SERVERS
fleet-overview-open-alerts = OPEN ALERTS
fleet-overview-crit-warn = { $crit } crit · { $warn } warn
fleet-overview-config-drift = CONFIG DRIFT
fleet-overview-fleet-load = FLEET LOAD
fleet-overview-oldest-host-key = OLDEST HOST KEY
fleet-overview-connected = CONNECTED
fleet-overview-reachable-over-their-transport = reachable over their transport
fleet-overview-launch-vm = + LAUNCH VM
fleet-overview-local-lab = LOCAL LAB
fleet-overview-ok = OK
fleet-overview-checking = CHECKING
fleet-overview-via = { $host } via { $name }
fleet-overview-no-servers-enrolled = No servers enrolled
fleet-overview-all-reachable = All { $connected_count } reachable
fleet-overview-checking-2 = Checking { $n } { $n ->
        [one] server
       *[other] servers
    }
fleet-overview-acknowledged = { $message } · acknowledged
fleet-overview-crow-wasnt-running-from-to = Crow wasn't running from { $fmt } to { $fmt2 }: nothing was watched then.
fleet-overview-all = ALL
fleet-overview-production = PRODUCTION
fleet-overview-staging = STAGING
fleet-overview-dev-lab = DEV / LAB
fleet-overview-by-region = BY REGION ({ $regions })
fleet-overview-by-region-2 = BY REGION
fleet-overview-by-group = BY GROUP ({ $groups_count })
fleet-overview-by-group-2 = BY GROUP
fleet-overview-hosts-sort-health-click-to = { $server_count } hosts · sort health ↓ · click to open
fleet-overview-of-hosts-click-to-open = { $visible_count } of { $server_count } hosts · click to open
fleet-overview-unknown = UNKNOWN
fleet-overview-detecting = DETECTING…
fleet-overview-detect-regions = DETECT REGIONS ↻
fleet-overview-group = GROUP:
fleet-overview-all-2 = ALL ({ $total_count })
fleet-overview-no-group = NO GROUP
fleet-overview-create = CREATE
fleet-overview-cancel = CANCEL
fleet-overview-new-group = + NEW GROUP
fleet-overview-host = HOST
fleet-overview-address = ADDRESS
fleet-overview-os = OS
fleet-overview-role = ROLE
fleet-overview-env = ENV
fleet-overview-cpu = CPU
fleet-overview-mem = MEM
fleet-overview-disk = DISK
fleet-overview-uptime = UPTIME
fleet-overview-alerts = ALERTS
fleet-overview-no-servers-enrolled-2 = NO SERVERS ENROLLED
fleet-overview-connect-remote-linux-servers-via = Connect remote Linux servers via SSH or spin up local isolated lab nodes.
fleet-overview-add-remote-server-ssh = + ADD REMOTE SERVER (SSH)
fleet-overview-hosts = HOSTS
fleet-overview-fleet-alerts = FLEET ALERTS
fleet-overview-open = { $count } open
fleet-overview-ack-all = ACK ALL
fleet-overview-activity = ACTIVITY
fleet-overview-audit-log-all-hosts = audit log · all hosts →
fleet-overview-no-recent-fleet-activity-recorded = No recent fleet activity recorded
fleet-overview-dismiss = DISMISS
fleet-overview-rolling-reboot = ROLLING REBOOT
fleet-overview-push-baseline-to-all = PUSH BASELINE TO ALL
fleet-overview-revoke-sessions = REVOKE SESSIONS
fleet-overview-rotate-host-keys = ROTATE HOST KEYS
fleet-overview-fleet-wide = FLEET-WIDE
fleet-overview-one-host-at-a-time = one host at a time · stops at the first failure
fleet-overview-rolling-reboot-2 = Rolling reboot
fleet-overview-reboots-every-reachable-server-in = Reboots every reachable server in turn, each back from a fresh boot before the next.
fleet-overview-push-baseline = Push baseline
fleet-overview-pushes-each-config-baseline-to = Pushes each config baseline to every server whose copy drifted from it.
fleet-overview-rotate-all-host-keys = Rotate all host keys
fleet-overview-new-ssh-host-keys-on = New SSH host keys on every server, re-pinned in known_hosts and proven with a fresh login; undone on any failure. Other machines will see a changed key.
fleet-overview-revoke-all-sessions = Revoke all sessions
fleet-overview-ends-every-ssh-login-on = Ends every SSH login on every server except Crow's own. Console logins and keys are untouched: anyone whose key still works can log back in.

## fleet-patching
fleet-patching-just-now = just now
fleet-patching-min-ago = { $s } min ago
fleet-patching-h-ago = { $s } h ago
fleet-patching-d-ago = { $s } d ago
fleet-patching-of-behind-on-security-updates = { $behind } of { $count } behind on security updates · { $reboots } waiting for a reboot
fleet-patching-every-server = every server
fleet-patching-group = group { $g }
fleet-patching-then-reboot-servers-that-need = then reboot servers that need it, servers behind a bastion before the bastion, waiting for services to come back
fleet-patching-each-run-lists-the-exact = Each run lists the exact packages per server and asks you to type a word before anything runs. One server at a time; the first failure stops it. Only the packages listed are upgraded (apt --only-upgrade, dnf upgrade, apk add --upgrade). Servers outside their group's window, unreachable ones and the machine Crow runs on are left out, with the reason.
fleet-patching-whos-behind = WHO'S BEHIND
fleet-patching-read-every-6-h-while = read { $ago } · every 6 h while Crow runs
fleet-patching-reading = READING { $count }…
fleet-patching-refresh = REFRESH
fleet-patching-server = SERVER
fleet-patching-group-2 = GROUP
fleet-patching-via = VIA
fleet-patching-security = SECURITY
fleet-patching-all = ALL
fleet-patching-kernel = KERNEL
fleet-patching-read = READ
fleet-patching-reading-2 = reading…
fleet-patching-not-read-yet = not read yet
fleet-patching-couldnt-read = couldn't read: { $error }
fleet-patching-needed = NEEDED
fleet-patching-maintenance-windows = MAINTENANCE WINDOWS
fleet-patching-per-group-local-time-a = per group, local time · a run leaves out servers outside theirs · blank: any time
fleet-patching-no-groups-yet-windows-are = No groups yet: windows are set per group (Fleet → BY GROUP).
fleet-patching-open-now = open now
fleet-patching-closed-now = closed now
fleet-patching-any-time = any time
fleet-patching-examples-sun-02-05-mon = Examples: Sun 02-05 · Mon-Fri 22-06 (past midnight belongs to the day it starts) · Sat,Sun 01-04 · daily 03-05. Enter saves.

## fleet-people
fleet-people-people = PEOPLE
fleet-people-accounts-keys-servers-read = { $count } accounts · { $key_rows_count } keys · { $scans_count } servers read
fleet-people-read-again = READ AGAIN
fleet-people-reading = READING { $count }…
fleet-people-couldnt-read = couldn't read: { $failed }
fleet-people-who-can-log-in-where = WHO CAN LOG IN WHERE
fleet-people-text = { $count } { $count ->
        [one] server
       *[other] servers
    }
fleet-people-offboard-everywhere-their-keys-are = Offboard everywhere: their keys are revoked first, then the account is locked and expired (reversible) or deleted (home kept). Servers where Crow logs in as this account are left out.
fleet-people-find-a-key-everywhere = FIND A KEY EVERYWHERE
fleet-people-every-key-in-any-authorized = every key in any authorized_keys, named when it's one of Crow's
fleet-people-crow = {" "}(Crow)
fleet-people-skips-crows-own-logins = (skips Crow's own logins)
fleet-people-an-account-with-their-key = an account with their key on a group's servers (created where missing)
fleet-people-every-server = every server
fleet-people-group = group { $g }
fleet-people-with-sudo = with sudo

## fleet-rollouts
fleet-rollouts-one-change-across-many-servers = One change across many servers: the canary first (the run pauses so you can check it), then the rest in batches that run side by side. The first failure stops everything, with that server's output.
fleet-rollouts-what = WHAT
fleet-rollouts-a-command = A COMMAND
fleet-rollouts-a-config-file = A CONFIG FILE
fleet-rollouts-path-enter-reads-every-servers = PATH (ENTER READS EVERY SERVER'S COPY)
fleet-rollouts-read = READ
fleet-rollouts-from-a-baseline = from a baseline:
fleet-rollouts-new-content = NEW CONTENT
fleet-rollouts-each-server-checks-it-with = Each server checks it with the format's own validator first (sshd -t, nginx -t, visudo, fstab…); refused content isn't written. Every write is kept in that server's config history.
fleet-rollouts-where = WHERE
fleet-rollouts-every-server = every server
fleet-rollouts-group = group { $g }
fleet-rollouts-left-out-unreachable-or-the = {" "}· { $count } left out (unreachable, or the machine Crow runs on)
fleet-rollouts-canary-runs-first-alone = CANARY · RUNS FIRST, ALONE
fleet-rollouts-no-canary = no canary
fleet-rollouts-then-at-a-time = THEN, AT A TIME
fleet-rollouts-preview-the-run = PREVIEW THE RUN →
fleet-rollouts-a-command-has-no-preview = A command has no preview: try it on the canary first.
fleet-rollouts-what-changes-per-server = WHAT CHANGES, PER SERVER
fleet-rollouts-type-the-path-and-press = Type the path and press Enter to read each server's copy.
fleet-rollouts-new-file = new file
fleet-rollouts-reading = reading…
fleet-rollouts-identical-this-server-is-left = Identical: this server is left out.
fleet-rollouts-no-such-file-there-yet = No such file there yet: it will be created.
fleet-rollouts-couldnt-read-it = Couldn't read it: { $e }
fleet-rollouts-reading-2 = Reading…
fleet-rollouts-start-from-this-servers-copy = START FROM THIS SERVER'S COPY
fleet-rollouts-text = { $count } { $count ->
        [one] server
       *[other] servers
    }{ $left_out }

## fleet-run-panel
fleet-run-panel-nothing-to-run = Nothing to run.
fleet-run-panel-pausing-after-the-canary = , pausing after the canary
fleet-run-panel-one-host-at-a-time = { $count } { $count ->
        [one] step
       *[other] steps
    }, one host at a time; the first failure stops the run.
fleet-run-panel-the-canary-is-done-check = The canary is done. Check it, then CONTINUE with the rest, or STOP here.
fleet-run-panel-stopping-after-the-current-host = Stopping after the current host…
fleet-run-panel-running = Running…
fleet-run-panel-not-run = not run
fleet-run-panel-left-out = left out · { $why }
fleet-run-panel-type-to-start = TYPE { $keyword } TO START
fleet-run-panel-continue-with-the-rest = CONTINUE WITH THE REST
fleet-run-panel-stop-here = STOP HERE
fleet-run-panel-stop-after-this-host = STOP AFTER THIS HOST
fleet-run-panel-close = CLOSE
fleet-run-panel-cancel = CANCEL
fleet-run-panel-in-the-first-failure-stops = { $count } { $count ->
        [one] step
       *[other] steps
    } in { $groups_count } { $groups_count ->
        [one] stage
       *[other] stages
    }{ $run }; the first failure stops the run.

## fleet-run
fleet-run-done = { $count } done
fleet-run-already-fine = { $skipped } already fine
fleet-run-stopped-at = stopped at { $failed }
fleet-run-stopped-on-request = stopped on request
fleet-run-not-run = { $not_run } not run

## fleet-setup
fleet-setup-default-unassigned = default / unassigned
fleet-setup-fleet-setup-policies = FLEET SETUP & POLICIES
fleet-setup-groups-tag-taxonomy-host-key = groups, tag taxonomy, host key lifecycle, and the fleet map
fleet-setup-close-esc = CLOSE esc
fleet-setup-enrolled-servers = ENROLLED SERVERS:
fleet-setup-fleet-groups = FLEET GROUPS:
fleet-setup-unique-tags = UNIQUE TAGS:
fleet-setup-host-keys-pinned = HOST KEYS PINNED:
fleet-setup-none-enrolled = NONE ENROLLED
fleet-setup-fleet-topology-group-assignments = FLEET TOPOLOGY & GROUP ASSIGNMENTS
fleet-setup-manage-new-group = + MANAGE / NEW GROUP
fleet-setup-no-servers-currently-enrolled-in = No servers currently enrolled in fleet.
fleet-setup-text = { $member_count } { $member_count ->
        [one] SERVER
       *[other] SERVERS
    }
fleet-setup-change-group = CHANGE GROUP ▾
fleet-setup-host-keys = HOST KEYS
fleet-setup-crow-pins-each-servers-host = Crow pins each server's host key fingerprint at enrollment and refuses to connect if it changes. Ages come from the key files' dates on each server; there's no rotation policy yet.
fleet-setup-fingerprint-pinned = Fingerprint pinned
fleet-setup-not-pinned = Not pinned
fleet-setup-keys-under-a-year-old = Keys under a year old
fleet-setup-keys-1-2-years-old = Keys 1–2 years old
fleet-setup-keys-over-2-years-old = Keys over 2 years old
fleet-setup-key-age-not-read-yet = Key age not read yet
fleet-setup-fleet-tag-taxonomy = FLEET TAG TAXONOMY
fleet-setup-no-custom-tags-defined-assign = No custom tags defined. Assign tags to nodes during onboarding or in server configuration to categorize your infrastructure.
fleet-setup-agentless-execution-guarantee = AGENTLESS EXECUTION GUARANTEE
fleet-setup-crow-talks-plain-ssh-and = Crow talks plain SSH and installs nothing on your servers. Fleet-wide actions aren't built yet; when they are, they'll run one host at a time and stop at the first failure.
fleet-setup-text-2 = { $count } { $count ->
        [one] server
       *[other] servers
    }
fleet-setup-groups-policies = GROUPS & POLICIES
fleet-setup-patching = PATCHING
fleet-setup-rollouts = ROLLOUTS
fleet-setup-drift-search = DRIFT & SEARCH
fleet-setup-hardening = HARDENING
fleet-setup-certificates = CERTIFICATES
fleet-setup-people = PEOPLE
fleet-setup-log-search = LOG SEARCH
fleet-setup-checks = CHECKS
fleet-setup-incidents = INCIDENTS
fleet-setup-dr-plans = DR PLANS
fleet-setup-team = TEAM
fleet-setup-audit-log = AUDIT LOG
fleet-setup-fleet-map = FLEET MAP

## fleet-state
fleet-state-no-data = NO DATA
fleet-state-connected-but-the-metrics-probe = connected, but the metrics probe failed
fleet-state-stopped = STOPPED
fleet-state-lab-container-isnt-answering = lab container isn't answering
fleet-state-the-metrics-probe-failed-on = the metrics probe failed on this machine

## fleet-team
fleet-team-team = TEAM
fleet-team-team-mode-on = { $count } { $count ->
        [one] member
       *[other] members
    } · team mode on
fleet-team-off-crow-is-single-user = off: Crow is single-user
fleet-team-sign-out-switch = SIGN OUT / SWITCH
fleet-team-who-uses-this-crow-with = Who uses this Crow, with which role, on which servers. Each member signs in with their own passphrase; their role is enforced on every action that changes a server (service actions, config writes, users and firewall, fleet runs), and every change record carries their name. Members live in this machine's vault: sharing them across machines needs Crow accounts (later).
fleet-team-you = {" "}(you)
fleet-team-fleet-runs-need-a-second = fleet runs need a second member's approval (their passphrase, typed at the run)
fleet-team-add-a-member = ADD A MEMBER
fleet-team-start-a-team-the-first = START A TEAM: THE FIRST MEMBER IS AN ADMIN (YOU)
fleet-team-role = ROLE
fleet-team-custom = CUSTOM
fleet-team-can = CAN
fleet-team-on = ON
fleet-team-group = group { $g }
fleet-team-env = env { $e }
fleet-team-none-picked-the-whole-fleet = (none picked: the whole fleet; envs win over groups)
fleet-team-add-member = ADD MEMBER
fleet-team-start-the-team = START THE TEAM
fleet-team-whos-at-the-keyboard = Who's at the keyboard?
fleet-team-this-crow-has-a-team = This Crow has a team: sign in so your role applies and your changes carry your name.
fleet-team-sign-in = SIGN IN

## history
history-at = at { $fmt }
history-cpu = CPU
history-memory = MEMORY
history-load-1m = LOAD (1m)
history-history = HISTORY
history-one-sample-a-minute-while = one sample a minute while Crow is open · { $readout }
history-now = { $chrono } → now
history-crow-wasnt-running-nothing-watched = Crow wasn't running: nothing watched
history-server-unreachable = server unreachable
history-alert-open-warn = alert open (warn)
history-alert-open-crit = alert open (crit)

## lock
lock-crow-vault-locked = CROW VAULT LOCKED
lock-password-2fa = PASSWORD + 2FA
lock-store-config-crow-crow-db = STORE: ~/.config/crow/crow.db
lock-2fa-mandatory = 2FA: MANDATORY
lock-master-password = MASTER PASSWORD
lock-hide = HIDE
lock-show = SHOW
lock-2fa-totp-code = 2FA TOTP CODE
lock-6-digits = 6 DIGITS
lock-tab-to-switch-to-unlock = Tab to switch · ↵ to unlock
lock-unlock-vault = UNLOCK VAULT

## lock-setup
lock-setup-setupstate = SetupState
lock-setup-security-notice-enabling-password-logon = SECURITY NOTICE · ENABLING PASSWORD LOGON
lock-setup-mandatory-2fa = MANDATORY 2FA
lock-setup-you-are-about-to-activate = You are about to activate local encrypted vault protection for Crow. Before proceeding, please review these essential security rules:
lock-setup-1-password-means-password-and = 1. PASSWORD MEANS PASSWORD AND 2FA
lock-setup-there-is-no-password-only = There is no password-only logon. Crow requires two-factor authentication (RFC 6238 TOTP) on every unlock. You will pair an authenticator app (1Password, Google Authenticator, YubiKey) in the next step.
lock-setup-2-what-the-password-protects = 2. WHAT THE PASSWORD PROTECTS
lock-setup-crow-encrypts-its-secrets-provider = Crow encrypts its secrets (provider tokens, AI API keys) with ChaCha20-Poly1305. With a password, their key is locked by it (Argon2id, 64MB, 3 iterations) instead of your OS keyring, is never written to disk unencrypted and is wiped from RAM on lock. The rest of ~/.config/crow/crow.db (server list, settings, change history) is not encrypted. Your SSH keys stay where they are in ~/.ssh.
lock-setup-3-no-cloud-backup-or = 3. NO CLOUD BACKUP OR PASSWORD RESET
lock-setup-this-is-a-local-only = This is a local-only, single-user system with zero telemetry. If you lose your master password or your 2FA authenticator, the secrets Crow stored (provider tokens, AI keys) can't be recovered and must be entered again. Your server list and settings stay readable.
lock-setup-cancel = CANCEL
lock-setup-i-understand-setup-password-2fa = I UNDERSTAND · SETUP PASSWORD & 2FA →
lock-setup-configure-password-mandatory-2fa = CONFIGURE PASSWORD & MANDATORY 2FA
lock-setup-step-2-of-2 = STEP 2 OF 2
lock-setup-master-password = MASTER PASSWORD
lock-setup-hide-password = HIDE PASSWORD
lock-setup-show-password = SHOW PASSWORD
lock-setup-confirm-master-password = CONFIRM MASTER PASSWORD
lock-setup-mandatory-two-factor-authentication-totp = MANDATORY TWO-FACTOR AUTHENTICATION (TOTP)
lock-setup-required = REQUIRED
lock-setup-scan-the-code-with-your = Scan the code with your authenticator app (1Password, Google Authenticator, Aegis…), or enter this key:
lock-setup-copied = ✓ COPIED
lock-setup-copy-key = COPY KEY
lock-setup-enter-6-digit-code-to = ENTER 6-DIGIT CODE TO CONFIRM PAIRING:
lock-setup-back = ← BACK
lock-setup-activate-password-2fa = ACTIVATE PASSWORD & 2FA

## logs
logs-systemd-journal-logs = SYSTEMD JOURNAL LOGS
logs-journalctl-o-json-output-fields = journalctl -o json --output-fields=*
logs-entries = { $filtered_count }/{ $total_count } entries
logs-volatile-lost-on-reboot = ⚠ VOLATILE — LOST ON REBOOT
logs-no-age-limit = no age limit
logs-no-size-cap = no size cap
logs-persistent = { $days } · { $size } PERSISTENT
logs-explain = ✦ EXPLAIN
logs-boundaries = ⚙ BOUNDARIES: { $bound_text }
logs-live-tail-on = LIVE TAIL ON
logs-paused = PAUSED
logs-export = EXPORT ⧉
logs-show-all = SHOW ALL
logs-clear = CLEAR
logs-range = RANGE
logs-boot = BOOT
logs-dedupe-repeats = DEDUPE REPEATS
logs-quick = QUICK
logs-load-older = ↑ load { $limit } older
logs-all = ALL
logs-err = ERR+
logs-warn = WARN+
logs-info = INFO+
logs-kernel = KERNEL
logs-crow-actions = ✓ CROW ACTIONS
logs-crow-actions-2 = CROW ACTIONS
logs-pid = PID { $fpid }
logs-terminate-pid = terminate PID { $fpid }?
logs-kill-sigterm = KILL (SIGTERM) ⏎
logs-cancel = cancel
logs-kill-pid = ⏻ KILL PID
logs-timestamp = TIMESTAMP
logs-level = LEVEL
logs-unit-identifier = UNIT / IDENTIFIER
logs-pid-2 = PID
logs-message = MESSAGE
logs-no-journal-log-entries-matched = No journal log entries matched your filter
logs-try-clearing-the-search-query = Try clearing the search query or selecting 'ALL' levels
logs-structured-systemd-journal-fields = STRUCTURED SYSTEMD JOURNAL FIELDS
logs-copy-message = COPY MESSAGE
logs-copy-json = COPY JSON
logs-asking-about-lines = Asking { $provider } about { $lines_sent } lines…
logs-lines-sent = { $provider } · { $lines_sent } lines sent
logs-get-a-plain-english-reading = Get a plain-English reading of the lines on screen: what's happening, what looks wrong, likely causes and what to check next.
logs-explain-logs = ✦ EXPLAIN LOGS
logs-working = working…
logs-no-provider = no provider
logs-sends-the-log-lines-on = Sends the log lines on screen (not host names) to { $p }. Nothing is sent until you click.
logs-add-an-api-key-for = Add an API key for a provider in Settings → Clankers to enable this.

## logs-retention-modal
logs-retention-modal-systemd-journal-retention-boundaries = SYSTEMD JOURNAL RETENTION & BOUNDARIES
logs-retention-modal-eliminate-30-minute-silent-log = Eliminate 30-minute silent log rotations & prevent crash wipeouts
logs-retention-modal-critical-storage-is-volatile-auto = ⚠ CRITICAL: STORAGE IS VOLATILE / AUTO
logs-retention-modal-if-this-host-crashes-or = If this host crashes or powers off, logs will be permanently wiped from RAM.
logs-retention-modal-lock-to-disk = LOCK TO DISK 🔒
logs-retention-modal-crash-safe-persistent-storage-active = CRASH-SAFE PERSISTENT STORAGE ACTIVE
logs-retention-modal-active-journals-persist-in-var = Active journals persist in /var/log/journal/ across server crashes and reboots.
logs-retention-modal-current-disk-usage = CURRENT DISK USAGE
logs-retention-modal-active-files = { $active_files_count } active files
logs-retention-modal-retention-horizon = RETENTION HORIZON
logs-retention-modal-hours = { $retention_horizon_hours } hours
logs-retention-modal-oldest = Oldest: { $oldest_timestamp }
logs-retention-modal-daily-write-rate = DAILY WRITE RATE
logs-retention-modal-mb-day = { $daily_burn_rate_mb } MB/day
logs-retention-modal-days-at-current-rate = ~{ $estimated_retained_days } days at current rate
logs-retention-modal-allocation = ALLOCATION: { $disk_usage_display } / { $max_use_display } ({ $usage_pct }%)
logs-retention-modal-systemmaxuse-boundary = SystemMaxUse boundary
logs-retention-modal-storage-persistence-storage = STORAGE PERSISTENCE (Storage)
logs-retention-modal-controls-whether-logs-survive-host = Controls whether logs survive host reboots & power loss
logs-retention-modal-persistent-rec = PERSISTENT (Rec)
logs-retention-modal-volatile-ram-only = VOLATILE (RAM-only)
logs-retention-modal-auto-fragile = AUTO (Fragile)
logs-retention-modal-max-disk-quota-systemmaxuse = MAX DISK QUOTA (SystemMaxUse)
logs-retention-modal-hard-ceiling-before-older-journals = Hard ceiling before older journals rotate out
logs-retention-modal-500-mb = 500 MB
logs-retention-modal-2-0-gb = 2.0 GB
logs-retention-modal-4-0-gb-rec = 4.0 GB (Rec)
logs-retention-modal-8-0-gb = 8.0 GB
logs-retention-modal-16-gb = 16 GB
logs-retention-modal-custom = Custom
logs-retention-modal-time-horizon-maxretentionsec = TIME HORIZON (MaxRetentionSec)
logs-retention-modal-maximum-age-before-logs-are = Maximum age before logs are purged
logs-retention-modal-7-days = 7 Days
logs-retention-modal-14-days = 14 Days
logs-retention-modal-30-days-rec = 30 Days (Rec)
logs-retention-modal-90-days = 90 Days
logs-retention-modal-180-days = 180 Days
logs-retention-modal-unlimited = Unlimited
logs-retention-modal-safety-disk-reserve-systemkeepfree = SAFETY DISK RESERVE (SystemKeepFree)
logs-retention-modal-guarantees-disk-space-stays-free = Guarantees disk space stays free to avoid root disk crashes
logs-retention-modal-1-0-gb = 1.0 GB
logs-retention-modal-2-0-gb-rec = 2.0 GB (Rec)
logs-retention-modal-5-0-gb = 5.0 GB
logs-retention-modal-edit-in-crow-config = EDIT IN CROW-CONFIG ↗
logs-retention-modal-apply-to-etc-systemd-journald = APPLY TO /etc/systemd/journald.conf

## logs-state
logs-state-oom-kills = OOM KILLS
logs-state-out-of-memory-oom-kill = out of memory|oom-kill|killed process
logs-state-segfaults = SEGFAULTS
logs-state-segfault-general-protection-traps = segfault|general protection|traps:
logs-state-firewall-blocks = FIREWALL BLOCKS
logs-state-ufw-block-ufw-audit-drop = ufw block|ufw audit|drop
logs-state-disk-errors = DISK ERRORS
logs-state-i-o-error-blk-update = i/o error|blk_update_request|ext4-fs error|xfs.*error|medium error
logs-state-ssh-login-failures = SSH LOGIN FAILURES
logs-state-failed-password-invalid-user-authentication = failed password|invalid user|authentication failure|maximum authentication attempts

## onboard
onboard-connect = Connect
onboard-verify = Verify
onboard-name = Name
onboard-check-connection = CHECK CONNECTION →
onboard-next-name-it = NEXT: NAME IT →
onboard-add-server = ADD SERVER ⏎
onboard-add-server-2 = ADD SERVER
onboard-cancel-esc = CANCEL esc
onboard-back = ← BACK
onboard-where-is-it-and-how = Where is it, and how does Crow log in?
onboard-crow-connects-over-ssh-and = Crow connects over SSH and installs nothing. The next step checks the server's host key before anything is sent.
onboard-or-a-lab-container-on = OR A LAB CONTAINER ON THIS MACHINE
onboard-address = ADDRESS
onboard-port = PORT
onboard-user = USER
onboard-reach-it = REACH IT
onboard-directly = Directly
onboard-via = via { $route }
onboard-only-reachable-through-another-server = Only reachable through another server? Mark that one as a bastion first: the "bastion" switch in its header.
onboard-every-connection-goes-through-the = Every connection goes through the bastion, logging in there with the bastion's own key. How the bastion reaches this server is up to its network; Crow only needs the address as the bastion sees it.
onboard-pick-a-bastion-when-this = Pick a bastion when this server is only reachable through one.
onboard-log-in-with = LOG IN WITH
onboard-a-key-in-crow = A key in Crow
onboard-a-key-from-settings-keys = A key from Settings → Keys, already on the server.
onboard-a-password-once = A password, once
onboard-crow-logs-in-once-installs = Crow logs in once, installs its own key, and forgets the password.
onboard-my-ssh-agent = My SSH agent
onboard-whatever-your-agent-and-ssh = Whatever your agent and ~/.ssh/config would use.
onboard-no-keys-in-crow-yet = No keys in Crow yet: use "A password, once", or add one in Settings → Keys & Rotation.
onboard-used-for-one-login-only = Used for one login only, after the host key is checked. Never saved.
onboard-crow-wont-know-which-key = Crow won't know which key that is, so it can't rotate or audit it.
onboard-checking = Checking…
onboard-not-checked-yet = Not checked yet.
onboard-no-answer = no answer
onboard-logging-in-and-reading-the = Logging in and reading the server…
onboard-reached-host-key-trusted-logged = Reached, host key trusted, logged in.
onboard-the-host-key-is-trusted = The host key is trusted, but logging in didn't work: see the log below.
onboard-the-host-key-changed-since = The host key changed since it was last trusted.
onboard-reached-check-the-host-key = Reached. Check the host key below, then trust it.
onboard-through = through { $v }
onboard-checking-2 = CHECKING…
onboard-check-again = CHECK AGAIN
onboard-host-key-trusted = HOST KEY · TRUSTED
onboard-host-key-changed-not-trusted = HOST KEY · CHANGED, NOT TRUSTED
onboard-host-key-new-to-this = HOST KEY · NEW TO THIS MACHINE
onboard-compare-it-with-the-servers = Compare it with the server's own, e.g. from its console: ssh-keygen -lf /etc/ssh/ssh_host_ed25519_key.pub
onboard-it-matches-trust-it = ✓ IT MATCHES: TRUST IT
onboard-possibly-a-reinstalled-server-possibly = Possibly a reinstalled server, possibly someone in the middle. If the change is expected, remove the old entry from ~/.ssh/known_hosts yourself, then check again.
onboard-log = LOG
onboard-distro = DISTRO
onboard-kernel = KERNEL
onboard-arch = ARCH
onboard-memory = MEMORY
onboard-disk = DISK
onboard-init = INIT
onboard-firewall = FIREWALL
onboard-open-ports = OPEN PORTS
onboard-time-sync = TIME SYNC
onboard-what-crow-read = WHAT CROW READ
onboard-config-tools = CONFIG TOOLS
onboard-crow-knows-debian-and-red = Crow knows Debian- and Red Hat-family config layouts; on this distro config discovery may miss files.
onboard-your-ssh-agent = your SSH agent
onboard-password-once = password, once
onboard-how-should-it-show-in = How should it show in the fleet?
onboard-all-of-this-can-be = All of this can be changed later.
onboard-name-2 = NAME
onboard-environment = ENVIRONMENT
onboard-group = GROUP
onboard-tags = TAGS
onboard-bastion = Bastion
onboard-other-servers-can-be-added = Other servers can be added through this one.
onboard-through-2 = through { $v } ·{" "}
onboard-group-2 = {" "}· group { $group }
onboard-crow-couldnt-log-in-yet = Crow couldn't log in yet: the server will be added, but shows as unreachable until it can.
onboard-couldnt-reach-it = Couldn't reach it: { $r }
onboard-logs-in-with = { $via }logs in with { $login }{ $state }{ $state2 }

## onboard-probe
onboard-probe-is-not-a-valid-host = { $host } is not a valid host
onboard-probe-could-not-resolve = could not resolve { $host }
onboard-probe-tcp = tcp { $host }:{ $port }: { $e }
onboard-probe-tcp-connect-failed = tcp connect { $host }:{ $port } failed: { $e }
onboard-probe-tcp-connect = tcp connect { $host }:{ $port }
onboard-probe-ssh-banner = ssh banner { $banner }
onboard-probe-no-ssh-banner-is-this = no SSH banner — is this an SSH port?
onboard-probe-host-key = host key { $host_key_fingerprint }
onboard-probe-could-not-fetch-the-servers = could not fetch the server's host keys (ssh-keyscan)
onboard-probe-couldnt-make-a-scratch-known = couldn't make a scratch known_hosts: { $e }
onboard-probe-userknownhostsfile = UserKnownHostsFile={ $kh }
onboard-probe-proxycommand = ProxyCommand={ $proxy }
onboard-probe-batchmode-yes = BatchMode=yes
onboard-probe-globalknownhostsfile-dev-null = GlobalKnownHostsFile=/dev/null
onboard-probe-stricthostkeychecking-no = StrictHostKeyChecking=no
onboard-probe-hashknownhosts-no = HashKnownHosts=no
onboard-probe-updatehostkeys-no = UpdateHostKeys=no
onboard-probe-pubkeyauthentication-no = PubkeyAuthentication=no
onboard-probe-passwordauthentication-no = PasswordAuthentication=no
onboard-probe-kbdinteractiveauthentication-no = KbdInteractiveAuthentication=no
onboard-probe-gssapiauthentication-no = GSSAPIAuthentication=no
onboard-probe-hostbasedauthentication-no = HostbasedAuthentication=no
onboard-probe-controlmaster-no = ControlMaster=no
onboard-probe-controlpath-none = ControlPath=none
onboard-probe-connecttimeout-10 = ConnectTimeout=10
onboard-probe-no-answer = no answer
onboard-probe-let-crow-in-but-couldnt = { $via } let Crow in but couldn't reach { $host }:{ $port }, or { $via } refused Crow
onboard-probe-through = through { $via }: { $why }
onboard-probe-reached-through = reached { $host }:{ $port } through { $via }
onboard-probe-ssh-banner-2 = ssh banner { $b }
onboard-probe-through-2 = through { $via }
onboard-probe-host-key-matches-ssh-known = host key matches ~/.ssh/known_hosts
onboard-probe-host-key-changed-known-hosts = HOST KEY CHANGED: known_hosts has a different key for this host — possible man-in-the-middle; not trusting it
onboard-probe-host-key-not-in-known = host key not in known_hosts — verify the fingerprint, then accept
onboard-probe-no-home-directory = no home directory
onboard-probe-could-not-read-facts-over = could not read facts over SSH: { $e }
onboard-probe-privileges = privileges: { $v }
onboard-probe-unknown-location = unknown location
onboard-probe-region = region { $place } · { $provider } { $code }
onboard-probe-cloud-metadata = cloud metadata
onboard-probe-ufw-installed = ufw installed
onboard-probe-read-os-release-uname-meminfo = read os-release, uname, meminfo, df, listening ports

## overview-collector
overview-collector-active = ACTIVE
overview-collector-inactive = INACTIVE
overview-collector-failed = FAILED
overview-collector-degraded = DEGRADED
overview-collector-unit-on = { $action } unit { $unit } on { $label }
overview-collector-sent-signal-to-pid-on = Sent signal { $signal } to PID { $pid } on { $label }

## overview-dashboard
overview-dashboard-open = OPEN →
overview-dashboard-services = SERVICES
overview-dashboard-active = ACTIVE
overview-dashboard-failed = FAILED
overview-dashboard-degraded = DEGRADED
overview-dashboard-inactive = INACTIVE
overview-dashboard-needs-attention = NEEDS ATTENTION
overview-dashboard-no-failed-or-degraded-units = No failed or degraded units.
overview-dashboard-cpu = { $cpu }% CPU
overview-dashboard-processes = PROCESSES
overview-dashboard-total = TOTAL
overview-dashboard-zombie = ZOMBIE
overview-dashboard-blocked-d = BLOCKED (D)
overview-dashboard-top-cpu = TOP CPU
overview-dashboard-no-processes-sampled-yet = No processes sampled yet.
overview-dashboard-top-memory = TOP MEMORY
overview-dashboard-sockets = SOCKETS
overview-dashboard-listening = LISTENING
overview-dashboard-all-interfaces = ALL INTERFACES
overview-dashboard-interface = INTERFACE
overview-dashboard-local-only = LOCAL ONLY
overview-dashboard-listening-ports-widest-exposure-first = LISTENING PORTS · WIDEST EXPOSURE FIRST
overview-dashboard-no-listening-sockets-sampled-yet = No listening sockets sampled yet.
overview-dashboard-updates-security = UPDATES & SECURITY
overview-dashboard-checked = checked { $checked_label }
overview-dashboard-checking = CHECKING…
overview-dashboard-refresh = REFRESH ↻
overview-dashboard-reading-the-package-manager-and = Reading the package manager and looking up CVEs…
overview-dashboard-not-checked-yet = Not checked yet.
overview-dashboard-pending = PENDING
overview-dashboard-security = SECURITY
overview-dashboard-cves-fixed-by-updating = CVES FIXED BY UPDATING
overview-dashboard-apply-security = APPLY { $security } SECURITY
overview-dashboard-apply-all = APPLY ALL { $count }
overview-dashboard-kernel = KERNEL
overview-dashboard-running = running { $u }
overview-dashboard-update-pending = { $is_security } update pending: { $change }
overview-dashboard-reboot-required-to-finish-installed = reboot required to finish installed updates
overview-dashboard-package-lists-refreshed-h-ago = package lists refreshed { $age_h }h ago
overview-dashboard-package-lists-are-days-old = package lists are { $h } days old — run `apt update` for current results
overview-dashboard-cves-fixed-by-pending-updates = CVES FIXED BY PENDING UPDATES
overview-dashboard-none-found-for-the-pending = None found for the pending updates.
overview-dashboard-more = +{ $fixed } more
overview-dashboard-known-cves-in-running-software = { $no_fix_yet } known CVEs in running software have no fix yet
overview-dashboard-cves = CVES
overview-dashboard-pending-updates-security-first = PENDING UPDATES · SECURITY FIRST
overview-dashboard-everything-is-up-to-date = Everything is up to date.
overview-dashboard-more-2 = +{ $pending } more
overview-dashboard-cve-data-osv-dev-crow = CVE data: OSV.dev · Crow sends package names and versions only, never host names

## overview-log-tail
overview-log-tail-log-tail = LOG TAIL
overview-log-tail-journalctl-f = journalctl -f
overview-log-tail-expand = EXPAND ↗
overview-log-tail-error = ERROR { $err_count }
overview-log-tail-warn = WARN { $warn_count }
overview-log-tail-info = INFO { $info_count }
overview-log-tail-stream-active = stream active
overview-log-tail-port = port { $port_str }
overview-log-tail-this-server = this server
overview-log-tail-socket-log-tail = SOCKET LOG TAIL
overview-log-tail-pid = PID { $p }
overview-log-tail-all-host-network-sockets-click = All host network sockets · click any socket row to trace
overview-log-tail-filter-socket = FILTER: SOCKET ✓
overview-log-tail-filter-all = FILTER: ALL
overview-log-tail-time = TIME
overview-log-tail-level = LEVEL
overview-log-tail-unit = UNIT
overview-log-tail-pid-2 = PID
overview-log-tail-message = MESSAGE
overview-log-tail-journalctl-f-u-service = journalctl -f -u { $process }.service
overview-log-tail-lines-live-stream = { $count } lines · live stream

## overview-models
overview-models-sleep = SLEEP
overview-models-disk = DISK
overview-models-zombie = ZOMBIE
overview-models-idle = IDLE
overview-models-loopback = LOOPBACK
overview-models-private-net = PRIVATE NET
overview-models-public-internet = PUBLIC INTERNET

## overview-service-inspector
overview-service-inspector-service-manager = SERVICE MANAGER
overview-service-inspector-select-a-service-on-the = Select a service on the left to manage it
overview-service-inspector-pid = PID
overview-service-inspector-cpu = CPU
overview-service-inspector-mem = MEM
overview-service-inspector-rss = RSS
overview-service-inspector-uptime = UPTIME
overview-service-inspector-did-not-reach-the-expected = { $unit } did not reach the expected state after the last action.
overview-service-inspector-retry = RETRY
overview-service-inspector-lifecycle = LIFECYCLE
overview-service-inspector-view-logs-for-this-unit = VIEW LOGS FOR THIS UNIT
overview-service-inspector-configuration = CONFIGURATION
overview-service-inspector-edit = EDIT { $file }
overview-service-inspector-open-in-crow-config-structured = open in crow-config — structured, validated editing
overview-service-inspector-recent-actions = RECENT ACTIONS
overview-service-inspector-ok = OK
overview-service-inspector-failed = FAILED
overview-service-inspector-pending = PENDING
overview-service-inspector-start = ▶ START
overview-service-inspector-reload = ⟳ RELOAD
overview-service-inspector-restart = ↻ RESTART
overview-service-inspector-stop = ■ STOP
overview-service-inspector-no-active-connections-will-be = No active connections will be dropped.
overview-service-inspector-established-and-listening-will-be = { $established } established { $established ->
        [one] connection
       *[other] connections
    } and { $listening } listening { $listening ->
        [one] socket
       *[other] sockets
    } will be dropped.
overview-service-inspector-checking-active-connections = Checking active connections…
overview-service-inspector-cancel = Cancel
overview-service-inspector-asking-the-clanker-what-this = Asking the clanker what this service is…
overview-service-inspector-low-risk = LOW RISK
overview-service-inspector-medium-risk = MEDIUM RISK
overview-service-inspector-high-risk = HIGH RISK
overview-service-inspector-critical = CRITICAL
overview-service-inspector-to-stop-or-restart = to stop or restart: { $reason }
overview-service-inspector-via-the-backup-the-primary = via the backup, { $via } · the primary failed: { $why }
overview-service-inspector-via-an-ais-reading-check = via { $via } · an AI's reading: check before acting on it

## overview-services-table
overview-services-table-table = TABLE
overview-services-table-map = MAP
overview-services-table-log-drawer-active = LOG DRAWER (ACTIVE)
overview-services-table-log-drawer = LOG DRAWER
overview-services-table-sort-port = sort port ↑
overview-services-table-sort-status-name = sort status · name
overview-services-table-sort-cpu = sort cpu ↓
overview-services-table-no-processes-match-try-all = No processes match — try ALL, or clear the search
overview-services-table-no-services-match-try-all = No services match — try ALL, or clear the search
overview-services-table-of = { $first }–{ $last } of { $total }
overview-services-table-prev = ‹ PREV
overview-services-table-next = NEXT ›
overview-services-table-group-by-status = GROUP BY STATUS
overview-services-table-group-by-name = GROUP BY NAME
overview-services-table-processes = { $total } processes
overview-services-table-running = { $running } running
overview-services-table-high-cpu = { $high_cpu } high cpu
overview-services-table-sockets = { $total } sockets
overview-services-table-listening = { $listen } listening
overview-services-table-established = { $estab } established
overview-services-table-units = { $total } units
overview-services-table-active = { $active } active
overview-services-table-degraded = { $degraded } degraded
overview-services-table-failed = { $failed } failed
overview-services-table-pid = PID
overview-services-table-user = USER
overview-services-table-cpu = CPU %
overview-services-table-mem = MEM %
overview-services-table-rss = RSS
overview-services-table-stat = STAT
overview-services-table-time = TIME
overview-services-table-command = COMMAND
overview-services-table-actions = ACTIONS
overview-services-table-proto = PROTO
overview-services-table-local-endpoint-firewall = LOCAL ENDPOINT / FIREWALL
overview-services-table-peer-endpoint = PEER ENDPOINT
overview-services-table-state = STATE
overview-services-table-process-service = PROCESS / SERVICE
overview-services-table-unit = UNIT
overview-services-table-uptime = UPTIME
overview-services-table-no-systemd-service-units-detected = No systemd service units detected on active host
overview-services-table-no-active-connections-will-be = No active connections will be dropped.
overview-services-table-established-and-listening-will-be = { $established } established { $established ->
        [one] connection
       *[other] connections
    } and { $listening } listening { $listening ->
        [one] socket
       *[other] sockets
    } will be dropped.
overview-services-table-checking-active-connections = Checking active connections…
overview-services-table-restart = Restart { $name }? { $blast_radius_text }
overview-services-table-cancel = Cancel{" "}
overview-services-table-esc = esc
overview-services-table-restart-2 = RESTART ⏎
overview-services-table-no-processes-detected-on-active = No processes detected on active host
overview-services-table-terminate-pid-send-sigterm = Terminate { $comm } (PID { $pid })? Send SIGTERM.
overview-services-table-kill-sigterm = KILL (SIGTERM) ⏎
overview-services-table-no-open-network-sockets-detected = No open network sockets detected on active host
overview-services-table-allow = allow #{ $rule_number }
overview-services-table-deny = deny #{ $rule_number }
overview-services-table-default-allow = default allow
overview-services-table-no-rule = no rule
overview-services-table-fw-off = fw off
overview-services-table-fw = fw ?

## overview-sockets-map
overview-sockets-map-gb = { $bytes } GB
overview-sockets-map-mb = { $bytes } MB
overview-sockets-map-kb = { $bytes } KB
overview-sockets-map-incoming-peers = INCOMING PEERS
overview-sockets-map-remote-clients-initiating-traffic-into = Remote clients initiating traffic into listening services
overview-sockets-map-local-processes-daemons = LOCAL PROCESSES & DAEMONS
overview-sockets-map-host-processes-maintaining-open-sockets = Host processes maintaining open sockets
overview-sockets-map-outgoing-peers = OUTGOING PEERS
overview-sockets-map-external-endpoints-reached-by-host = External endpoints reached by host processes
overview-sockets-map-hide-127-0-0-1 = HIDE 127.0.0.1
overview-sockets-map-focus = FOCUS: { $p_name }
overview-sockets-map-showing-of-established-connections = SHOWING { $shown_conns } OF { $total_conns } ESTABLISHED CONNECTIONS
overview-sockets-map-no-connections-in-this-category = No connections in this category.
overview-sockets-map-ports = { $count } { $count ->
        [one] connection
       *[other] connections
    } · ports: { $ports_summary }
overview-sockets-map-no-active-process-connections = No active process connections.
overview-sockets-map-pid = PID { $p }
overview-sockets-map-proc-unprivileged = proc unprivileged
overview-sockets-map-in = ← { $in_count } in
overview-sockets-map-out = → { $out_count } out
overview-sockets-map-protocol-state-direction = Protocol: { $protocol } · State: { $state } · Direction: { $direction }
overview-sockets-map-process-pid = Process: { $process } (PID { $pid })
overview-sockets-map-view-in-processes = VIEW IN PROCESSES →
overview-sockets-map-view-in-firewall = VIEW IN FIREWALL →

## overview-state
overview-state-all-traffic = ALL TRAFFIC
overview-state-incoming = INCOMING
overview-state-outgoing = OUTGOING
overview-state-live = LIVE
overview-state-failed = FAILED
overview-state-inactive = INACTIVE
overview-state-all = ALL
overview-state-apps = APPS
overview-state-root = ROOT
overview-state-cpu-1 = CPU > 1%
overview-state-kernel = KERNEL

## overview-summary
overview-summary-all-interfaces = ALL INTERFACES
overview-summary-interface = INTERFACE
overview-summary-local = LOCAL
overview-summary-failed = FAILED
overview-summary-degraded = DEGRADED

## overview-updates
overview-updates-ubuntu = Ubuntu:{ $os_version }
overview-updates-debian = Debian:{ $os_version }

## settings-clankers
settings-clankers-clankers-llm-providers-keys = CLANKERS · LLM PROVIDERS & KEYS
settings-clankers-configured = { $configured_count }/{ $count } CONFIGURED
settings-clankers-30d-cumulative-calls = 30d Cumulative Calls:
settings-clankers-calls = { $total_calls_30d } calls
settings-clankers-primary-backup = Primary · backup:
settings-clankers-no-backup = no backup
settings-clankers-ai-usability-engine-log-clarification = AI Usability Engine (Log Clarification & ELI5 Assistance)
settings-clankers-configure-api-keys-for-llm = Configure API keys for LLM providers. Crow calls them only when you ask, to explain journal lines in plain English. Keys are stored encrypted in Crow's vault (its key is held by your OS keyring, or locked by your vault password when it's on) and sent through curl's stdin, never the command line. Log lines you send go to that provider.
settings-clankers-keys-cant-be-saved-right = Keys can't be saved right now: { $why }.
settings-clankers-supported-clanker-providers-30-day = SUPPORTED CLANKER PROVIDERS & 30-DAY TELEMETRY
settings-clankers-set-ends = set · ends …{ $tail }
settings-clankers-no-key-configured = NO KEY CONFIGURED
settings-clankers-primary = ★ PRIMARY
settings-clankers-backup = ↺ BACKUP
settings-clankers-ready = READY
settings-clankers-not-setup = NOT SETUP
settings-clankers-api-key = API Key:
settings-clankers-model = Model:
settings-clankers-endpoint = Endpoint:
settings-clankers-30-day-activity = 30-Day Activity:
settings-clankers-calls-2 = { $calls_30d } calls
settings-clankers-last-used = Last used: { $ts }
settings-clankers-never-used = Never used
settings-clankers-edit-key = ⚙ Edit Key
settings-clankers-testing = testing…
settings-clankers-test-key = ⚡ Test Key
settings-clankers-reset = Reset
settings-clankers-remove-backup = Remove backup
settings-clankers-make-backup = Make backup
settings-clankers-make-primary = Make primary
settings-clankers-ai-usability-sandbox-wtf-is = AI USABILITY SANDBOX: "WTF IS THIS LOG TRYING TO SAY?"
settings-clankers-eli5-assistant-preview = ELI5 ASSISTANT PREVIEW
settings-clankers-load-presets = Load Presets:
settings-clankers-kernel-129-412033-out-of = kernel: [  129.412033] Out of memory: Kill process 28419 (mysqld) score 812 or sacrifice child
settings-clankers-oom-killer = OOM Killer
settings-clankers-nginx-1482-segfault-at-0 = nginx[1482]: segfault at 0 ip 00007f3b48201a08 sp 00007ffe3410 error 4 in libc.so.6
settings-clankers-segfault-in-libc = Segfault in Libc
settings-clankers-systemd-1-postgresql-16-main = systemd[1]: postgresql@16-main.service: Main process exited, code=exited, status=1/FAILURE
settings-clankers-systemd-failure = Systemd Failure
settings-clankers-sample-raw-journal-kernel-panic = Sample Raw Journal / Kernel Panic Log Line:
settings-clankers-asking = ASKING…
settings-clankers-translate-log-eli5 = TRANSLATE LOG (ELI5) ↵
settings-clankers-asking-the-provider = asking the provider…
settings-clankers-models-offered-for-this-key = { $count } models offered for this key
settings-clankers-load-models = LOAD MODELS
settings-clankers-configure-clanker-api-key = Configure { $p_name } Clanker API Key
settings-clankers-api-key-stored-encrypted-in = API Key (stored encrypted in Crow's vault; never shown again):
settings-clankers-custom-api-endpoint-proxy-optional = Custom API Endpoint / Proxy (optional):
settings-clankers-save-config = SAVE CONFIG ↵

## settings-clankers-state
settings-clankers-state-kernel-129-412033-out-of = kernel: [  129.412033] Out of memory: Kill process 28419 (mysqld) score 812 or sacrifice child

## settings-config-table
settings-config-table-yes = YES
settings-config-table-no = NO
settings-config-table-ms = { $n } ms
settings-config-table-empty = (empty)
settings-config-table-none = (none)
settings-config-table-enable-setting = Enable setting
settings-config-table-disable-setting = Disable setting
settings-config-table-aggressive-timeout = Aggressive timeout
settings-config-table-standard-recommended = Standard recommended
settings-config-table-tolerant-of-high-latency-links = Tolerant of high-latency links
settings-config-table-slow-satellite-dialup = Slow satellite / dialup
settings-config-table-aggressive-ping = Aggressive ping
settings-config-table-relaxed-heartbeat = Relaxed heartbeat
settings-config-table-off = OFF
settings-config-table-disable-keepalive-ping = Disable keepalive ping
settings-config-table-2-attempts = 2 attempts
settings-config-table-fast-dead-peer-detection = Fast dead-peer detection
settings-config-table-3-attempts = 3 attempts
settings-config-table-5-attempts = 5 attempts
settings-config-table-tolerate-transient-packet-drop = Tolerate transient packet drop
settings-config-table-10-attempts = 10 attempts
settings-config-table-very-tolerant = Very tolerant
settings-config-table-1s-high-resolution = 1s (High resolution)
settings-config-table-higher-local-remote-cpu-use = Higher local/remote CPU use
settings-config-table-2s-default = 2s (Default)
settings-config-table-smooth-streaming-low-load = Smooth streaming & low load
settings-config-table-5s-battery-eco = 5s (Battery/Eco)
settings-config-table-low-background-overhead = Low background overhead
settings-config-table-10s-minimal = 10s (Minimal)
settings-config-table-minimal-polling-footprint = Minimal polling footprint
settings-config-table-chacha20-poly1305-fast-modern = chacha20-poly1305 (Fast & modern)
settings-config-table-hardware-agnostic-aead = Hardware-agnostic AEAD
settings-config-table-aes128-gcm-aes-ni-accelerated = aes128-gcm (AES-NI accelerated)
settings-config-table-fast-on-modern-intel-amd = Fast on modern Intel/AMD
settings-config-table-aes256-gcm-maximum-depth = aes256-gcm (Maximum depth)
settings-config-table-high-security-environments = High security environments
settings-config-table-aes256-gcm-only = AES256-GCM only
settings-config-table-default-ed25519-key = Default Ed25519 key
settings-config-table-dedicated-bastion-key = Dedicated bastion key
settings-config-table-legacy-rsa-identity = Legacy RSA identity
settings-config-table-risk = Risk: { $risk }
settings-config-table-preparing = preparing…
settings-config-table-fleet-page-background = FLEET PAGE BACKGROUND
settings-config-table-a-picture-behind-the-server = A picture behind the server list, faded and softened so the table stays readable. PNG, JPEG or WebP; Crow keeps its own copy.
settings-config-table-change-picture = CHANGE PICTURE…
settings-config-table-choose-picture = CHOOSE PICTURE…
settings-config-table-remove = REMOVE
settings-config-table-general = GENERAL
settings-config-table-general-application-behavior = {"["}general] · application behavior
settings-config-table-connection-ssh = CONNECTION & SSH
settings-config-table-connection-applies-to-every-host = {"["}connection] · applies to every host unless overridden
settings-config-table-keys-rotation = KEYS & ROTATION
settings-config-table-keys-key-distribution-policies = {"["}keys] · key distribution & policies
settings-config-table-vault-security = VAULT & SECURITY
settings-config-table-vault-encrypted-secrets-who-can = {"["}vault] · encrypted secrets & who can open Crow
settings-config-table-servers-archives = SERVERS & ARCHIVES
settings-config-table-servers-archiving-and-how-long = {"["}servers] · archiving and how long stored data is kept
settings-config-table-ui-components-lab = UI COMPONENTS LAB
settings-config-table-lab-gpui-component-testbed-sandbox = {"["}lab] · gpui-component testbed & sandbox
settings-config-table-clankers-ai-usability = CLANKERS (AI USABILITY)
settings-config-table-clankers-api-keys-log-eli5 = {"["}clankers] · api keys & log eli5 helpers
settings-config-table-providers = PROVIDERS
settings-config-table-providers-cloud-accounts-your-servers = {"["}providers] · cloud accounts your servers run on
settings-config-table-plugins = PLUGINS
settings-config-table-plugins-integrations-off-until-you = {"["}plugins] · integrations, off until you switch them on
settings-config-table-personalisation = PERSONALISATION
settings-config-table-appearance-make-crow-yours-saved = {"["}appearance] · make Crow yours · saved as you change it
settings-config-table-password-logon-is-not-configured = PASSWORD LOGON IS NOT CONFIGURED
settings-config-table-disabled-default = DISABLED (DEFAULT)
settings-config-table-without-a-password-anyone-using = Without a password, anyone using your computer account can open Crow. Secrets (provider tokens, AI keys) are still encrypted, with the key held by your OS keyring. A master password with two-factor authentication (RFC 6238 TOTP) makes Crow ask before it opens, and locks the secrets' key with that password instead.
settings-config-table-set-password-2fa = SET PASSWORD & 2FA →
settings-config-table-password-2fa-vault-configured = PASSWORD + 2FA VAULT CONFIGURED
settings-config-table-locked-stance = LOCKED STANCE
settings-config-table-database-and-keys-are-secured = Database and keys are secured. Unlocking requires password and 6-digit TOTP code. Auto-lock engages after the configured timeout.
settings-config-table-lock-vault-now-l = LOCK VAULT NOW (⇧⌘L)
settings-config-table-its = it's
settings-config-table-theyre = they're
settings-config-table-changed = CHANGED
settings-config-table-or-enter-custom-value = OR ENTER CUSTOM VALUE:
settings-config-table-apply = APPLY
settings-config-table-reset-section = RESET SECTION
settings-config-table-edit-config-toml = EDIT config.toml ⌘/
settings-config-table-save-s-pending = SAVE ⌘S ({ $total_changed } pending)
settings-config-table-save-s = SAVE ⌘S
settings-config-table-more-in-this-section-not = { $hidden_settings } more { $hidden_settings ->
        [one] setting
       *[other] settings
    } in this section { $hidden_settings ->
        [one] is
       *[other] are
    } not wired to anything yet, so { $hidden_settings2 } hidden rather than pretending to work.

## settings-diff-rail
settings-diff-rail-in-sync = in sync
settings-diff-rail-pending-diff = PENDING DIFF
settings-diff-rail-configuration-in-sync = Configuration in sync
settings-diff-rail-working-copy-matches-config-crow = Working copy matches ~/.config/crow/config.toml
settings-diff-rail-affects-live-sessions = AFFECTS LIVE SESSIONS
settings-diff-rail-pending-ssh-connection-changes-take = Pending SSH connection changes take effect upon reconnection. Existing active sessions will retain their current parameters.
settings-diff-rail-keychain = KEYCHAIN
settings-diff-rail-no-keys-enrolled-yet = No keys enrolled yet.

## settings-email-form
settings-email-form-just-now = just now
settings-email-form-min-ago = { $secs } min ago
settings-email-form-h-ago = { $secs } h ago
settings-email-form-d-ago = { $secs } d ago
settings-email-form-waiting-retried-every-5-min = {" "}· { $count } waiting, retried every 5 min
settings-email-form-in-the-next-digest = {" "}· { $count } in the next digest
settings-email-form-in-the-next-digest-2 = { $count } in the next digest
settings-email-form-security = SECURITY
settings-email-form-smtp-server = SMTP SERVER
settings-email-form-port = PORT
settings-email-form-no-encryption-the-password-and = No encryption: the password and mail cross the network in the clear. Only for a relay on this machine or your own network.
settings-email-form-username = USERNAME
settings-email-form-password = PASSWORD
settings-email-form-from = FROM
settings-email-form-to-comma-separated = TO (comma separated)
settings-email-form-critical-alerts = CRITICAL ALERTS
settings-email-form-warnings = WARNINGS
settings-email-form-digest-every-minutes = DIGEST EVERY (MINUTES)
settings-email-form-quiet-hours-digests-wait = QUIET HOURS (DIGESTS WAIT)
settings-email-form-resolved-alerts-send-nothing-alerts = Resolved alerts send nothing. Alerts already open when you switch this on aren't mailed. Crow has no agent: mail only goes out while Crow is running.
settings-email-form-save = SAVE
settings-email-form-sending = SENDING…
settings-email-form-send-test = SEND TEST
settings-email-form-test-failed = Test failed: { $m }
settings-email-form-last-send-failed = Last send failed: { $e }{ $d }
settings-email-form-last-mail-sent = Last mail sent { $ago }{ $d }

## settings-keys
settings-keys-scan-idle = Scan idle
settings-keys-all = ALL ({ $count })
settings-keys-group = + GROUP
settings-keys-scan = SCAN ⟳
settings-keys-path = + PATH
settings-keys-generate-key = + GENERATE KEY
settings-keys-auto-monitored-paths = auto-monitored paths
settings-keys-unenrolled-ssh-discovered-on-disk = { $count } UNENROLLED SSH { $count ->
        [one] KEY
       *[other] KEYS
    } DISCOVERED ON DISK
settings-keys-found-in-scan-paths-click = found in scan paths · click Import to enroll
settings-keys-fp = fp: { $fingerprint }
settings-keys-import-to = + IMPORT TO { $target_grp }
settings-keys-enrolled-keys-in-memory = ENROLLED KEYS IN MEMORY ({ $count })
settings-keys-filtered-by-group = filtered by group: { $g }
settings-keys-all-groups = all groups
settings-keys-persisted-in-config-crow-crow = persisted in ~/.config/crow/crow.db
settings-keys-no-enrolled-keys-matching-this = NO ENROLLED KEYS MATCHING THIS FILTER
settings-keys-scan-ssh-to-import-existing = Scan ~/.ssh to import existing keys, or click '+ GENERATE KEY' to create one.
settings-keys-private-key-linked = ● PRIVATE KEY LINKED
settings-keys-public-key-only = ○ PUBLIC KEY ONLY
settings-keys-reachable-servers = REACHABLE SERVERS:
settings-keys-attach-host = + ATTACH HOST
settings-keys-sha256-fingerprint = SHA256 FINGERPRINT:
settings-keys-path-2 = path: { $path }
settings-keys-comment = comment: { $c }
settings-keys-copied-fingerprint-to-clipboard = Copied fingerprint to clipboard
settings-keys-copy-fingerprint = COPY FINGERPRINT
settings-keys-copied-openssh-public-key-to = Copied OpenSSH public key to clipboard
settings-keys-copy-public-key = COPY PUBLIC KEY
settings-keys-enrolled = enrolled: { $key }
settings-keys-edit-attach-servers = EDIT / ATTACH SERVERS ✎
settings-keys-switch-attached-servers-to-it = SWITCH ATTACHED SERVERS TO IT
settings-keys-rotate = ROTATE
settings-keys-remove-from-memory = REMOVE FROM MEMORY ✕
settings-keys-scan-paths = SCAN PATHS
settings-keys-add-path = + ADD PATH
settings-keys-default = default
settings-keys-group-taxonomy = GROUP TAXONOMY
settings-keys-new-group = + NEW GROUP
settings-keys-text = { $count } { $count ->
        [one] key
       *[other] keys
    }
settings-keys-key-rotation-audit = KEY ROTATION & AUDIT
settings-keys-private-keys-stay-on-this = Private keys stay on this machine; only public keys go to servers. ROTATE makes a new key, installs it on every server that uses the old one, proves it logs in, then removes the old one, one server at a time. There is no automatic rotation policy.
settings-keys-keypair-generated-and-enrolled = ✓ KEYPAIR GENERATED AND ENROLLED
settings-keys-generate-new-openssh-keypair = + GENERATE NEW OPENSSH KEYPAIR
settings-keys-esc = esc
settings-keys-private-key-written-to-disk =
    Private key written to disk with permissions 0600:
    { $priv_path }
settings-keys-fingerprint = Fingerprint: { $fp }
settings-keys-copied-public-key-to-clipboard = Copied public key to clipboard
settings-keys-key-name = KEY NAME:
settings-keys-algorithm = ALGORITHM:
settings-keys-ed25519-recommended = Ed25519 (Recommended)
settings-keys-rsa-4096-bit = RSA 4096-bit
settings-keys-comment-optional = COMMENT (OPTIONAL):
settings-keys-save-directory = SAVE DIRECTORY:
settings-keys-tab-to-switch-fields-enter = tab to switch fields · enter to generate
settings-keys-cancel = CANCEL
settings-keys-generate-keypair = GENERATE KEYPAIR ↵
settings-keys-add-ssh-key-scan-directory = ADD SSH KEY SCAN DIRECTORY
settings-keys-enter-directory-path-e-g = Enter directory path (e.g. ~/work-keys or /etc/ssh):
settings-keys-add-scan = ADD & SCAN ↵
settings-keys-create-new-key-group = CREATE NEW KEY GROUP
settings-keys-enter-group-name-e-g = Enter group name (e.g. Staging Fleet, Edge Bastions):
settings-keys-create-group = CREATE GROUP ↵
settings-keys-edit-key-attached-servers = EDIT KEY & ATTACHED SERVERS
settings-keys-name = NAME:
settings-keys-group-assignment = GROUP ASSIGNMENT:
settings-keys-attach-to-fleet-servers = ATTACH TO FLEET SERVERS:
settings-keys-save-changes = SAVE CHANGES ↵

## settings-keys-state
settings-keys-state-scanned-found-new = Scanned { $count } { $count ->
        [one] path
       *[other] paths
    } · { $discovered_count } { $discovered_count ->
        [one] key
       *[other] keys
    } found ({ $new_count } new)

## settings-lab
settings-lab-type-something-here-to-test = Type something here to test gpui-component...
settings-lab-type-to-reveal-clear-button = Type to reveal clear button...
settings-lab-search-fleet-by-tag-or = search fleet by tag or region...
settings-lab-enter-sensitive-secret = Enter sensitive secret...
settings-lab-fleet-deployment-manifest-env-production =
    # Fleet Deployment Manifest
    env: production
    replicas: 4
    region: us-east-1
    auto_drain: true
settings-lab-ui-components-lab = UI COMPONENTS LAB
settings-lab-gpui-component-transition-testbed = · gpui-component transition testbed
settings-lab-isolated-sandbox = ISOLATED SANDBOX
settings-lab-evaluation-transition-sandbox = EVALUATION & TRANSITION SANDBOX
settings-lab-this-page-runs-gpui-component = This page runs gpui-component widgets directly inside Crow to test platform-native text selection, cursor blinking, clipboard chords, undo/redo, and HarfBuzz text shaping before migrating existing screens.
settings-lab-1-primary-native-gpui-input = 1. PRIMARY NATIVE GPUI INPUT
settings-lab-native-gpui-component-input = NATIVE GPUI-COMPONENT INPUT
settings-lab-rope-buffer-shaping = Rope Buffer + Shaping
settings-lab-native-mouse-drag-selection-cmd =
    ✓ Native mouse drag selection
    ✓ Cmd+A, Cmd+C, Cmd+V, Cmd+Z (undo!)
    ✓ Word jumps (Alt+Arrows) & line ends (Cmd+Arrows)
settings-lab-2-input-variants-capabilities = 2. INPUT VARIANTS & CAPABILITIES
settings-lab-password-input-with-native-mask = PASSWORD INPUT (WITH NATIVE MASK TOGGLE & CLEAR BUTTON):
settings-lab-search-filter-input-cleanable = SEARCH / FILTER INPUT (CLEANABLE):
settings-lab-prefix-suffix-adornments = PREFIX & SUFFIX ADORNMENTS:
settings-lab-port-22 = PORT 22
settings-lab-3-6-digit-otp-authenticator = 3. 6-DIGIT OTP AUTHENTICATOR INPUT
settings-lab-grouped-otp-cells-split-3 = GROUPED OTP CELLS (SPLIT 3-3 WITH AUTO-ADVANCE & BACKSPACE):
settings-lab-direct-replacement-candidate-for-vault = Direct replacement candidate for Vault Lock and Setup screens.
settings-lab-4-multi-line-textarea = 4. MULTI-LINE TEXTAREA
settings-lab-multi-line-editor-with-word = MULTI-LINE EDITOR WITH WORD-WRAP & MULTI-LINE SELECTION:
settings-lab-5-gpui-component-button-matrix = 5. GPUI-COMPONENT BUTTON MATRIX
settings-lab-primary-action = Primary Action
settings-lab-secondary = Secondary
settings-lab-ghost = Ghost
settings-lab-destructive = Destructive

## settings
settings-general = General
settings-connection-ssh = Connection & SSH
settings-keys-rotation = Keys & Rotation
settings-vault-security = Vault & Security
settings-servers-archives = Servers & Archives
settings-ui-components-lab = UI Components Lab
settings-clankers-ai = Clankers (AI)
settings-providers = Providers
settings-plugins = Plugins
settings-personalisation = Personalisation
settings-in-sync = { $path_str } · in sync
settings-pending = { $path_str } · { $total_changed } pending { $total_changed ->
        [one] change
       *[other] changes
    }
settings-no-servers = no servers
settings-1-server = 1 server
settings-servers = { $n } servers
settings-settings = SETTINGS
settings-close-esc = CLOSE esc

## settings-plugins
settings-plugins-local-virtual-machines = LOCAL VIRTUAL MACHINES
settings-plugins-containers = CONTAINERS
settings-plugins-cloud-providers = CLOUD PROVIDERS
settings-plugins-notifications = NOTIFICATIONS
settings-plugins-plugins-what-crow-connects-to = PLUGINS · WHAT CROW CONNECTS TO
settings-plugins-on = { $on }/{ $count } ON
settings-plugins-re-check = RE-CHECK
settings-plugins-crow-does-nothing-with-these = Crow does nothing with these until you switch them on. A plugin that's on adds what it brings where it belongs (Multipass adds + LAUNCH VM to the Fleet; a cloud provider adds import, power and snapshots); one that's off isn't probed or shown.
settings-plugins-checking = checking…
settings-plugins-not-installed-on-this-machine = not installed on this machine
settings-plugins-not-in-this-build = not in this build
settings-plugins-not-checked-yet = not checked yet
settings-plugins-no-account-yet-set-one = No account yet · set one up in Providers →
settings-plugins-1-account-providers = 1 account · Providers →
settings-plugins-accounts-providers = { $n } accounts · Providers →

## settings-providers
settings-providers-providers-where-your-servers-live = PROVIDERS · WHERE YOUR SERVERS LIVE
settings-providers-set-up = { $configured }/{ $count } SET UP
settings-providers-import-instances = IMPORT INSTANCES
settings-providers-tokens-cant-be-saved-right = Tokens can't be saved right now: { $why }.
settings-providers-no-cloud-provider-plugin-is = No cloud provider plugin is on. Turn on Linode or UpCloud in Settings → Plugins to connect an account.
settings-providers-connect-the-cloud-accounts-your = Connect the cloud accounts your servers run on. Crow reads your instances from them and, where the provider supports it, can power servers and take snapshots from outside SSH. Tokens are stored encrypted in Crow's vault: its key is held by your OS keyring (Keychain, Secret Service) or, with a vault password, locked by that password. They reach the provider through curl's stdin, never the command line.
settings-providers-vault-security = Vault & Security →
settings-providers-not-set-up = not set up
settings-providers-testing = testing…
settings-providers-connected = connected · { $msg }
settings-providers-saved-not-tested-yet = saved · not tested yet
settings-providers-checked = checked { $short_time }
settings-providers-edit = EDIT
settings-providers-set-up-2 = SET UP
settings-providers-test-connection = TEST CONNECTION
settings-providers-click-again-to-remove = CLICK AGAIN TO REMOVE
settings-providers-remove = REMOVE
settings-providers-crow-is-open-anyone-at = Crow is OPEN: anyone at this session can use this token to power servers off or snapshot them. See the stance badge.
settings-providers-not-saved = Not saved: { $e }
settings-providers-secret-vault = secret · vault
settings-providers-clear = CLEAR
settings-providers-save-test = SAVE & TEST
settings-providers-cancel = CANCEL

## settings-vault-manage
settings-vault-manage-password = PASSWORD
settings-vault-manage-change-password = CHANGE PASSWORD…
settings-vault-manage-go-back-to-open = GO BACK TO OPEN…
settings-vault-manage-going-back-to-open-means = Going back to OPEN means:
settings-vault-manage-i-understand-without-the-password = I understand: without the password, anything running as me can read Crow's secrets.
settings-vault-manage-change-password-2 = CHANGE PASSWORD
settings-vault-manage-go-open = GO OPEN
settings-vault-manage-cancel = CANCEL

## terminal
terminal-ctrl-shift-t = Ctrl+Shift+T
terminal-ctrl-shift-d = Ctrl+Shift+D
terminal-ctrl-shift-s = Ctrl+Shift+S
terminal-ctrl-shift-w = Ctrl+Shift+W
terminal-enter-renames-esc-cancels-empty = Enter renames · Esc cancels · empty follows the shell
terminal-press-enter-to-start-a = { $why } · press Enter to start a new one
terminal-starting = starting…
terminal-lines-back-type-to-return = ↑ { $scrolled } lines back · type to return
terminal-no-terminals-open-opens-one = No terminals open. { $shortcut } opens one.

## topology
topology-crow = CROW
topology-server = SERVER
topology-multipass-vm = MULTIPASS VM
topology-lab-container = LAB CONTAINER
topology-container = CONTAINER
topology-ssh-key = SSH KEY
topology-checking-posture = checking posture…
topology-posture-not-checked-yet = posture not checked yet
topology-just-now = just now
topology-min = { $mins } min
topology-all = ALL
topology-at-risk-only-crit-warn = AT RISK ONLY · { $crit } crit · { $warn } warn
topology-check-posture = CHECK POSTURE
topology-reset-view = RESET VIEW
topology-no-servers-yet-enroll-one = No servers yet: enroll one, or launch a Multipass VM from LOCAL LAB & VMS.
topology-ssh = SSH
topology-via-jump-host = via jump host
topology-key-unlocks = key unlocks
topology-critical-risk = critical risk
topology-text = { $count } { $count ->
        [one] RISK
       *[other] RISKS
    }
topology-no-risks-found = NO RISKS FOUND
topology-risks = RISKS
topology-env-group = env · group
topology-host-key = host key
topology-not-pinned = not pinned
topology-cpu-mem-disk = cpu { $cpu_pct }% · mem { $mem_pct }% · disk { $disk_pct }%
topology-posture = POSTURE
topology-not-checked-yet-check-posture = Not checked yet (CHECK POSTURE reads it on reachable servers).
topology-couldnt-read-needs-root = couldn't read (needs root)
topology-root-login = root login
topology-open-to-all = open to all
topology-key = KEY
topology-private-key = private key
topology-not-on-this-machine = not on this machine
topology-connected-to = CONNECTED TO
topology-via-jump = via jump
topology-open-server = OPEN SERVER
topology-terminal = TERMINAL
topology-posture-checked-ago-servers = posture checked { $mins } ago · { $count } servers

## users-host-data
users-host-data-cant-read-etc-passwd = can't read /etc/passwd: { $e }
users-host-data-unnamed-key = unnamed key
users-host-data-is-not-a-valid-name = { $name } is not a valid { $kind } name
users-host-data-the-password-is-empty = the password is empty
users-host-data-the-password-cant-contain-line = the password can't contain line breaks
users-host-data-the-password-is-longer-than = the password is longer than 256 characters
users-host-data-ag = -aG
users-host-data-is-not-an-absolute-shell = { $shell } is not an absolute shell path
users-host-data-the-full-name-cant-contain = the full name can't contain ':' or line breaks
users-host-data-the-root-account-cant-be = the root account can't be deleted
users-host-data-not-a-valid-openssh-public = not a valid OpenSSH public key

## users-models
users-models-active = ACTIVE
users-models-locked = LOCKED
users-models-system = SYSTEM
users-models-all = ALL
users-models-login = LOGIN
users-models-sudo = SUDO

## users
users-users = USERS
users-new-user = + NEW USER
users-user = USER
users-uid = UID
users-groups = GROUPS
users-shell = SHELL
users-keys = KEYS
users-status = STATUS
users-couldnt-read-accounts = Couldn't read accounts: { $e }
users-reading-accounts = Reading accounts…
users-no-accounts-match-try-all = No accounts match — try ALL, or clear the search
users-select-an-account-to-see = Select an account to see its details.
users-running = Running: { $p } …
users-sudo = SUDO
users-root = ROOT
users-account = ACCOUNT
users-uid-gid = uid / gid
users-can-log-in = can log in
users-no-password-locked = no — password locked
users-no-no-login-shell = no — no login shell
users-primary = { $g } (primary)
users-remove-from-no-sudo = REMOVE FROM { $group } (NO SUDO)
users-make-admin-add-to-sudo = MAKE ADMIN: ADD TO { $group } (SUDO)
users-authorized-ssh-keys = AUTHORIZED SSH KEYS
users-no-keys-in-ssh-authorized = No keys in ~/.ssh/authorized_keys.
users-add-key-from-vault = + ADD KEY FROM VAULT
users-login-shell = LOGIN SHELL
users-password = PASSWORD
users-locked-no-password-login-setting = locked — no password login (setting one unlocks it)
users-set-can-log-in-with = set (can log in with a password)
users-set-with-chpasswd-over-stdin = Set with chpasswd over stdin; Crow doesn't keep it.
users-set-password = SET PASSWORD
users-cancel = CANCEL
users-set-password-2 = SET PASSWORD…
users-actions = ACTIONS
users-unlock-password = UNLOCK PASSWORD
users-lock-password = LOCK PASSWORD
users-view-passwd-entry = VIEW PASSWD ENTRY
users-click-again-to-delete = CLICK AGAIN TO DELETE { $username }
users-delete-user = DELETE USER

## users-new-user-modal
users-new-user-modal-provision-new-system-user = PROVISION NEW SYSTEM USER
users-new-user-modal-login-username-posix = LOGIN USERNAME (POSIX)
users-new-user-modal-full-name-gecos-metadata = FULL NAME (GECOS METADATA)
users-new-user-modal-password-optional = PASSWORD (OPTIONAL)
users-new-user-modal-the-passwords-dont-match = The passwords don't match.
users-new-user-modal-leave-empty-for-key-only = Leave empty for key-only login (add a key from the inspector afterwards).
users-new-user-modal-set-with-chpasswd-over-stdin = Set with chpasswd over stdin; Crow doesn't keep it.
users-new-user-modal-login-shell = LOGIN SHELL
users-new-user-modal-initial-privileges = INITIAL PRIVILEGES
users-new-user-modal-admin-group = ✓ ADMIN ({ $g } group)
users-new-user-modal-standard-user-click-to-add = STANDARD USER · click to add to { $g }
users-new-user-modal-supplementary-groups-usermod-ag = SUPPLEMENTARY GROUPS (usermod -aG)
users-new-user-modal-create-user-home-directory-home = Create user home directory (/home/<username>) with skeleton dotfiles
users-new-user-modal-enabled = ENABLED
users-new-user-modal-disabled = DISABLED
users-new-user-modal-provision-account = PROVISION ACCOUNT

## users-passwd-inspector
users-passwd-inspector-return-to-users = ← RETURN TO USERS
users-passwd-inspector-account = ACCOUNT: { $uname }
users-passwd-inspector-open-in-config-editor = OPEN IN CONFIG EDITOR
users-passwd-inspector-raw-7-field-posix-passwd = RAW 7-FIELD POSIX PASSWD RECORD
users-passwd-inspector-delimiter-colon = DELIMITER: ':' (COLON)
users-passwd-inspector-exploded-field-anatomy-system-semantics = EXPLODED FIELD ANATOMY & SYSTEM SEMANTICS
users-passwd-inspector-switch-passwd-record-to-inspect = SWITCH PASSWD RECORD TO INSPECT
users-passwd-inspector-login-username = LOGIN USERNAME
users-passwd-inspector-password-placeholder = PASSWORD PLACEHOLDER
users-passwd-inspector-indicates-account-password-is-locked = '!' indicates account password is locked. Login through PAM/password is denied.
users-passwd-inspector-x-indicates-hashed-passwords-and = 'x' indicates hashed passwords and salt are shadowed in /etc/shadow, restricted to root (0640).
users-passwd-inspector-user-identifier-uid = USER IDENTIFIER (UID)
users-passwd-inspector-uid-0-superuser-root-with = UID 0: Superuser (root) with unrestricted Linux kernel privileges.
users-passwd-inspector-system-service-uid-reserved-for = System Service UID: Reserved for isolated background daemons and system services.
users-passwd-inspector-interactive-login-uid-standard-regular = Interactive Login UID: Standard regular human user accounts start at UID 1000+.
users-passwd-inspector-primary-group-gid = PRIMARY GROUP (GID)
users-passwd-inspector-primary-numeric-group-id-applied = Primary numeric group ID applied to new files and processes created by this user. Defined in /etc/group.
users-passwd-inspector-gecos-metadata = GECOS METADATA
users-passwd-inspector-empty = (empty)
users-passwd-inspector-general-electric-comprehensive-operating-supervisor = General Electric Comprehensive Operating Supervisor legacy field. Holds Full Name, Office, Phone numbers.
users-passwd-inspector-home-directory = HOME DIRECTORY
users-passwd-inspector-initial-working-directory-set-upon = Initial working directory set upon login session initialization. Houses ~/.ssh, ~/.config, and dotfiles.
users-passwd-inspector-login-shell = LOGIN SHELL
users-passwd-inspector-prohibits-interactive-terminal-access-daemons = Prohibits interactive terminal access. Daemons and service accounts use nologin for defense in depth.
users-passwd-inspector-command-interpreter-spawned-upon-login = Command interpreter spawned upon login session (e.g. /bin/bash, /bin/zsh, /usr/bin/fish).
users-passwd-inspector-portion = PORTION #{ $num }

## users-ssh-attach-modal
users-ssh-attach-modal-ssh-keychain-attachment = SSH KEYCHAIN ATTACHMENT · { $uname }
users-ssh-attach-modal-attached = { $count } ATTACHED
users-ssh-attach-modal-target-file = TARGET FILE: { $target_auth_path }
users-ssh-attach-modal-permissions-0600-secure = PERMISSIONS: 0600 (SECURE)
users-ssh-attach-modal-currently-authorized-keys-for-user = CURRENTLY AUTHORIZED KEYS FOR USER
users-ssh-attach-modal-no-ssh-keys-currently-installed = No SSH keys currently installed for this user. Attach a key below.
users-ssh-attach-modal-revoke = REVOKE
users-ssh-attach-modal-select-from-crow-ssh-keychain = SELECT FROM CROW SSH KEYCHAIN
users-ssh-attach-modal-enrolled-vault-keys = { $count } ENROLLED VAULT KEYS
users-ssh-attach-modal-no-enrolled-keys-found-in = No enrolled keys found in Crow Vault. Go to Settings > SSH Keys to generate or scan keys.
users-ssh-attach-modal-attached-2 = ✓ ATTACHED
users-ssh-attach-modal-attach-key = + ATTACH KEY
