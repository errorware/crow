#!/bin/sh
# A venv in the plugin's data folder, with one pinned package.
set -e
echo "creating $VENV"
python3 -m venv "$VENV"
echo "installing $PACKAGE"
"$VENV/bin/pip" install --disable-pip-version-check --no-input -q "$PACKAGE"
"$VENV/bin/python" -c 'import tabulate; print("tabulate", tabulate.__version__, "is ready")'
