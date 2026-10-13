#!/bin/sh
# The fullest disk as a status; $1 is the percentage that's a warning.
limit="$1"
worst=$(df -P -x tmpfs -x devtmpfs -x squashfs 2>/dev/null | awk 'NR > 1 { sub("%", "", $5); if ($5 + 0 > max) { max = $5 + 0; at = $6 } } END { printf "%d %s", max, at }')
pct=${worst%% *}
at=${worst#* }
state=ok
[ "$pct" -ge "$limit" ] && state=warn
printf '{"state":"%s","label":"fullest disk %s%% (%s)"}\n' "$state" "$pct" "$at"
