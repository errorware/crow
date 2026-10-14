#!/bin/sh
# Real disks as a table. Pseudo and read-only image filesystems are left
# out: by type where df has -x (GNU), else by name (busybox).
echo "Filesystem Size Used Avail Use% Mounted"
{ df -P -h -x tmpfs -x devtmpfs -x squashfs -x composefs 2>/dev/null || df -P -h; } | awk 'NR > 1 && $1 !~ /^(tmpfs|devtmpfs|shm|udev|none|composefs)$/ && $1 !~ /^\/dev\/loop/ && $6 !~ /^\/(dev|proc|sys|run|snap)(\/|$)/ {
  m = $6; for (i = 7; i <= NF; i++) m = m " " $i
  print $1, $2, $3, $4, $5, m
}'
