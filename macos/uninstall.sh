#!/bin/sh
pkill -x shut-up 2>/dev/null || true
launchctl bootout "gui/$(id -u)/com.metixw.shut-up" 2>/dev/null || true
rm -f "$HOME/Library/LaunchAgents/com.metixw.shut-up.plist"
rm -rf "$HOME/Applications/Shut up.app" "/Applications/Shut up.app"
rm -rf "$HOME/Library/Application Support/shut-up"
echo "shut up uninstalled"
