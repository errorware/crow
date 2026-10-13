#!/bin/sh
# What the note would become, next to what's there now.
echo "now:"
if [ -f /etc/motd.d/50-crow-note ]; then sed 's/^/  /' /etc/motd.d/50-crow-note; else echo "  (no note)"; fi
echo "after:"
printf '  %s\n' "$1"
