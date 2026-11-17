#!/bin/sh
set -e
cd "$(dirname "$0")/.."
cargo build --release
./macos/bundle.sh target/release/shut-up target
pkill -x shut-up 2>/dev/null || true
mkdir -p "$HOME/Applications"
rm -rf "$HOME/Applications/Shut up.app"
cp -R "target/Shut up.app" "$HOME/Applications/"
open "$HOME/Applications/Shut up.app"
echo "shut up installed in ~/Applications, it starts with your mac"
