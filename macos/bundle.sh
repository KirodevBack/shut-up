#!/bin/sh
set -e
cd "$(dirname "$0")/.."

BIN="${1:-target/release/shut-up}"
OUT="${2:-target}"
VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)"
APP="$OUT/Shut up.app"
ICONSET="$OUT/shut-up.iconset"

rm -rf "$APP" "$ICONSET"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources" "$ICONSET"
cp "$BIN" "$APP/Contents/MacOS/shut-up"

for size in 16 32 128 256; do
  sips -z "$size" "$size" assets/shut-it-up.png --out "$ICONSET/icon_${size}x${size}.png" >/dev/null
done
for size in 16 32 128; do
  double=$((size * 2))
  sips -z "$double" "$double" assets/shut-it-up.png --out "$ICONSET/icon_${size}x${size}@2x.png" >/dev/null
done
iconutil -c icns "$ICONSET" -o "$APP/Contents/Resources/shut-up.icns"
rm -rf "$ICONSET"

cat > "$APP/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key>
    <string>Shut up</string>
    <key>CFBundleDisplayName</key>
    <string>Shut up</string>
    <key>CFBundleIdentifier</key>
    <string>com.metixw.shut-up</string>
    <key>CFBundleExecutable</key>
    <string>shut-up</string>
    <key>CFBundleIconFile</key>
    <string>shut-up</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>$VERSION</string>
    <key>CFBundleVersion</key>
    <string>$VERSION</string>
    <key>LSMinimumSystemVersion</key>
    <string>11.0</string>
    <key>LSUIElement</key>
    <true/>
</dict>
</plist>
EOF

codesign --force --deep --sign - "$APP" 2>/dev/null || true
echo "$APP"
