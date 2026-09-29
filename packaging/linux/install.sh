#!/bin/sh
# Installs Crow for the current user: the binary in ~/.local/bin, and the
# desktop entry and icons so launchers and the window list show Crow's icon.
# Run it from the unpacked release directory. `./install.sh --uninstall`
# removes what it installed.
set -eu
here=$(cd "$(dirname "$0")" && pwd)
bin="${XDG_BIN_HOME:-$HOME/.local/bin}"
data="${XDG_DATA_HOME:-$HOME/.local/share}"
id=rs.crow.Crow

if [ "${1:-}" = "--uninstall" ]; then
    rm -f "$bin/crow" "$data/applications/$id.desktop"
    find "$data/icons/hicolor" -name "$id.*" -delete 2>/dev/null || true
    echo "Crow removed. Your vault and settings in ~/.config/crow are untouched."
    exit 0
fi

install -Dm755 "$here/crow" "$bin/crow"
# The entry runs the installed binary by its full path, PATH or not.
mkdir -p "$data/applications"
sed "s|^Exec=crow$|Exec=$bin/crow|" "$here/share/applications/$id.desktop" > "$data/applications/$id.desktop"
( cd "$here/share/icons" && find hicolor -type f ) | while read -r f; do
    install -Dm644 "$here/share/icons/$f" "$data/icons/$f"
done
# Refresh caches where the tools exist; launchers pick the icon up either way.
command -v gtk-update-icon-cache >/dev/null && gtk-update-icon-cache -qtf "$data/icons/hicolor" 2>/dev/null || true
command -v update-desktop-database >/dev/null && update-desktop-database -q "$data/applications" 2>/dev/null || true

echo "Crow installed to $bin/crow."
case ":$PATH:" in
    *":$bin:"*) ;;
    *) echo "Note: $bin isn't on your PATH, so run it as $bin/crow from a shell; the app launcher entry works as is." ;;
esac
