#!/bin/sh
# Installs y2e for the current user, from the folder of an extracted release.
# Uninstall: sh install.sh --uninstall
set -eu

here=$(cd "$(dirname "$0")" && pwd)
bin="${HOME}/.local/bin"
apps="${XDG_DATA_HOME:-$HOME/.local/share}/applications"
icons="${XDG_DATA_HOME:-$HOME/.local/share}/icons/hicolor/scalable/apps"

if [ "${1:-}" = "--uninstall" ]; then
    rm -f "$bin/y2e-gui" "$apps/y2e.desktop" "$icons/y2e.svg"
    echo "y2e removed. Its downloaded tools stay in ~/.local/share/y2e (delete it to free the space)."
    exit 0
fi

mkdir -p "$bin" "$apps" "$icons"
install -m 755 "$here/y2e-gui" "$bin/y2e-gui"
install -m 644 "$here/y2e.desktop" "$apps/y2e.desktop"
install -m 644 "$here/y2e.svg" "$icons/y2e.svg"
command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database "$apps" || true

echo "y2e installed in $bin."
case ":$PATH:" in *":$bin:"*) ;; *) echo "Add $bin to your PATH to start it from a terminal." ;; esac
