"""Logs in to each server in Crow's inventory through Crow's ssh_config."""
import json
import subprocess
import sys

from tabulate import tabulate

inventory, ssh_config, results = sys.argv[1:4]
hosts = []
section = None
for line in open(inventory):
    line = line.strip()
    if line.startswith("["):
        section = line
    elif line and section == "[crow]":
        hosts.append(line.split()[0])

rows, reached = [], 0
with open(results, "w") as out:
    for host in hosts:
        print(f"reaching {host}…", flush=True)
        r = subprocess.run(["ssh", "-F", ssh_config, host, "uname -sr; id -un"], capture_output=True, text=True, timeout=60)
        ok = r.returncode == 0
        kernel, user = (r.stdout.splitlines() + ["", ""])[:2] if ok else ("", "")
        reached += ok
        rows.append([host, "yes" if ok else "no", kernel, user or r.stderr.strip()[:60]])
        out.write(json.dumps({"host": host, "ok": ok, "summary": kernel if ok else r.stderr.strip()[:120]}) + "\n")

print(tabulate(rows, headers=["server", "reached", "kernel", "as"]))
print(f"reached {reached} of {len(hosts)}")
sys.exit(0 if reached == len(hosts) else 1)
