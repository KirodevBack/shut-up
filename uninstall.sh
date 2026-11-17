#!/bin/sh
pkill -x shut-up 2>/dev/null || true
rm -f "$HOME/.local/bin/shut-up" "$HOME/.config/autostart/shut-up.desktop"
rm -rf "${XDG_CONFIG_HOME:-$HOME/.config}/shut-up"
echo "shut up uninstalled"
