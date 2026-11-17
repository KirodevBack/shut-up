#!/bin/sh
set -e
cd "$(dirname "$0")"
cargo build --release
BIN="$HOME/.local/bin"
mkdir -p "$BIN" "$HOME/.config/autostart"
install -m 755 target/release/shut-up "$BIN/shut-up"
cat > "$HOME/.config/autostart/shut-up.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=Shut up
Exec=$BIN/shut-up
X-GNOME-Autostart-enabled=true
NoDisplay=true
EOF
if command -v setcap >/dev/null 2>&1; then
  sudo setcap cap_sys_nice=ep "$BIN/shut-up" || true
fi
pkill -x shut-up 2>/dev/null || true
nohup "$BIN/shut-up" >/dev/null 2>&1 &
echo "shut up installed in $BIN, it starts with your session"
