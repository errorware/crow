"""Runs devsec.hardening on the servers in Crow's inventory.

--check: what applying would change (ansible-playbook --check --diff).
--apply: applies the roles.
--status: a check run, reported per server as hardened or changes pending.

Crow's own login is kept working: root logs in with keys only
(PermitRootLogin prohibit-password), other login users keep key login, and
a bastion keeps TCP forwarding for the servers behind it. Crow proves its
login before and after an apply.
"""
import argparse
import json
import os
import re
import subprocess
import sys
import tempfile

p = argparse.ArgumentParser()
mode = p.add_mutually_exclusive_group(required=True)
mode.add_argument("--check", action="store_true")
mode.add_argument("--apply", action="store_true")
mode.add_argument("--status", action="store_true")
p.add_argument("--roles", required=True)
p.add_argument("--inventory", required=True)
p.add_argument("--results", required=True)
a = p.parse_args()

known = ["os_hardening", "ssh_hardening", "nginx_hardening", "mysql_hardening"]
roles = [r for r in a.roles.split(",") if r in known]
if not roles:
    print("no roles chosen", file=sys.stderr)
    sys.exit(2)

venv = os.path.join(os.environ["HOME"], "venv")
collections = os.path.join(os.environ["HOME"], "collections")

playbook = """
- name: devsec.hardening, from Crow
  hosts: crow
  become: true
  vars:
    # Crow's own login keeps working.
    ssh_permit_root_login: "{{ 'without-password' if ansible_user == 'root' else 'no' }}"
    ssh_server_password_login: false
    ssh_allow_tcp_forwarding: "{{ 'yes' if (crow_bastion | default('no')) == 'yes' else 'no' }}"
    ssh_allow_agent_forwarding: false
  roles:
""" + "".join(f"    - devsec.hardening.{r}\n" for r in roles)

with tempfile.NamedTemporaryFile("w", suffix=".yml", delete=False, dir=os.environ.get("TMPDIR")) as f:
    f.write(playbook)
    book = f.name

argv = [os.path.join(venv, "bin", "ansible-playbook"), "-i", a.inventory, book]
if not a.apply:
    argv += ["--check", "--diff"]
env = dict(os.environ, ANSIBLE_COLLECTIONS_PATH=collections, ANSIBLE_RETRY_FILES_ENABLED="0", ANSIBLE_STDOUT_CALLBACK="default", ANSIBLE_NOCOLOR="1")

recap = {}
proc = subprocess.Popen(argv, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, env=env)
in_recap = False
for line in proc.stdout:
    print(line, end="", flush=True)
    if line.startswith("PLAY RECAP"):
        in_recap = True
    elif in_recap:
        m = re.match(r"^(\S+)\s+:\s+(.*)$", line.strip())
        if m:
            counts = dict((k, int(v)) for k, v in re.findall(r"(\w+)=(\d+)", m.group(2)))
            recap[m.group(1)] = counts
proc.wait()

with open(a.results, "w") as out:
    for host, c in recap.items():
        failed = c.get("failed", 0) + c.get("unreachable", 0)
        changed = c.get("changed", 0)
        line = {"host": host, "ok": failed == 0, "changed": changed}
        if a.status:
            if failed:
                line.update(state="bad", label="DevSec check failed")
            elif changed:
                line.update(state="warn", label=f"DevSec: {changed} changes pending")
            else:
                line.update(state="ok", label="DevSec hardened")
            line["summary"] = line["label"]
        elif a.check:
            line["summary"] = "unreachable" if c.get("unreachable") else f"{changed} would change"
        else:
            line["summary"] = "unreachable" if c.get("unreachable") else f"{changed} changed, {c.get('failed', 0)} failed"
        out.write(json.dumps(line) + "\n")

sys.exit(proc.returncode)
