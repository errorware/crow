#!/bin/sh
# Writes the note, a step at a time so the console shows it.
set -e
echo "making /etc/motd.d"
mkdir -p /etc/motd.d
sleep 1
echo "writing the note"
printf '%s\n' "$1" > /etc/motd.d/50-crow-note.tmp
sleep 1
mv /etc/motd.d/50-crow-note.tmp /etc/motd.d/50-crow-note
echo "wrote $(wc -c < /etc/motd.d/50-crow-note) bytes to /etc/motd.d/50-crow-note"
