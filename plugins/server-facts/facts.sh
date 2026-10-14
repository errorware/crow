#!/bin/sh
# What this server is, as one JSON object.
esc() { printf '%s' "$1" | sed 's/\\/\\\\/g; s/"/\\"/g'; }
os=$( . /etc/os-release 2>/dev/null && printf '%s' "$PRETTY_NAME" )
up=$(cut -d' ' -f1 /proc/uptime 2>/dev/null | cut -d. -f1)
printf '{"hostname":"%s","os":"%s","kernel":"%s","arch":"%s","uptime":"%sd %sh","cpus":"%s"}\n' \
    "$(esc "$(hostname 2>/dev/null || cat /etc/hostname)")" "$(esc "${os:-unknown}")" "$(esc "$(uname -r)")" "$(uname -m)" \
    "$(( ${up:-0} / 86400 ))" "$(( ${up:-0} % 86400 / 3600 ))" "$(getconf _NPROCESSORS_ONLN 2>/dev/null || echo ?)"
