#!/bin/sh
# The controller, in the plugin's data folder: a venv with pinned
# ansible-core, and the devsec.hardening collection with its dependencies.
set -e
echo "creating $VENV"
python3 -m venv "$VENV"
echo "installing ansible-core $ANSIBLE_CORE"
"$VENV/bin/pip" install --disable-pip-version-check --no-input -q "ansible-core==$ANSIBLE_CORE"
echo "installing devsec.hardening $DEVSEC_HARDENING"
"$VENV/bin/ansible-galaxy" collection install --force --collections-path "$COLLECTIONS" "devsec.hardening:==$DEVSEC_HARDENING"
"$VENV/bin/ansible" --version | head -n 1
echo "ready"
