#!/bin/sh
# The newest restic snapshot's age as a status: {"state": ..., "label": ...}.
# $1: warn when it's older than this many hours.
max_hours="$1"
if ! latest=$(restic snapshots --latest 1 --json 2>/dev/null); then
    echo '{"state":"bad","label":"repository unreadable"}'
    exit 0
fi
time=$(printf '%s' "$latest" | sed -n 's/.*"time":"\([^"]*\)".*/\1/p' | head -n 1)
if [ -z "$time" ]; then
    echo '{"state":"warn","label":"no snapshots yet"}'
    exit 0
fi
then_s=$(date -d "$time" +%s 2>/dev/null) || { echo '{"state":"unknown","label":"unreadable snapshot time"}'; exit 0; }
age=$(( ($(date +%s) - then_s) / 3600 ))
state=ok
[ "$age" -gt "$max_hours" ] && state=warn
printf '{"state":"%s","label":"last backup %sh ago"}\n' "$state" "$age"
