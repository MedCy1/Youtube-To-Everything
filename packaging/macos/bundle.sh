#!/bin/sh
# Builds y2e.app from a macOS binary and zips it. Runs on macOS only.
#
#   packaging/macos/bundle.sh <binary> <version> <output.zip> [file...]
#
# Any further files (license texts) are placed in the bundle's Resources.
set -eu

bin="$1"; version="${2#v}"; zip="$3"
shift 3
here=$(cd "$(dirname "$0")/.." && pwd)
app="y2e.app"

rm -rf "$app" y2e.iconset
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources" y2e.iconset
cp "$bin" "$app/Contents/MacOS/y2e-gui"

for size in 16 32 128 256 512; do
    sips -z $size $size "$here/y2e.png" --out "y2e.iconset/icon_${size}x${size}.png" >/dev/null
    sips -z $((size * 2)) $((size * 2)) "$here/y2e.png" --out "y2e.iconset/icon_${size}x${size}@2x.png" >/dev/null
done
iconutil -c icns y2e.iconset -o "$app/Contents/Resources/y2e.icns"
rm -rf y2e.iconset
for extra in "$@"; do cp "$extra" "$app/Contents/Resources/"; done

cat > "$app/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key><string>YouTube to Everything</string>
    <key>CFBundleDisplayName</key><string>YouTube to Everything</string>
    <key>CFBundleIdentifier</key><string>io.github.medcy1.y2e</string>
    <key>CFBundleExecutable</key><string>y2e-gui</string>
    <key>CFBundleIconFile</key><string>y2e</string>
    <key>CFBundlePackageType</key><string>APPL</string>
    <key>CFBundleShortVersionString</key><string>$version</string>
    <key>CFBundleVersion</key><string>$version</string>
    <key>LSMinimumSystemVersion</key><string>11.0</string>
    <key>NSHighResolutionCapable</key><true/>
</dict>
</plist>
PLIST

# Not signed by an Apple developer, but a signature of any kind is required to run on Apple silicon.
codesign --force --deep --sign - "$app"
ditto -c -k --keepParent "$app" "$zip"
