#!/bin/sh
# The fullest real disk as a status; $1 is the percentage that's a warning.
# Pseudo and read-only image filesystems are left out: by type where df
# has -x (GNU), else by name (busybox).
limit="$1"
worst=$({ df -P -x tmpfs -x devtmpfs -x squashfs -x composefs 2>/dev/null || df -P ; } | awk 'NR > 1 && $1 !~ /^(tmpfs|devtmpfs|shm|udev|none|composefs)$/ && $1 !~ /^\/dev\/loop/ && $6 !~ /^\/(dev|proc|sys|run|snap)(\/|$)/ {
  sub("%", "", $5); if ($5 + 0 >= max) { max = $5 + 0; at = $6 }
} END { if (at != "") printf "%d %s", max, at }')
if [ -z "$worst" ]; then
  printf '{"state":"unknown","label":"no disks read"}\n'
  exit 0
fi
pct=${worst%% *}
at=${worst#* }
state=ok
[ "$pct" -ge "$limit" ] && state=warn
printf '{"state":"%s","label":"fullest disk %s%% (%s)"}\n' "$state" "$pct" "$at"
