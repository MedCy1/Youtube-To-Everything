#!/bin/sh
# Builds y2e.app from a macOS binary and wraps it in a disk image. Runs on macOS only.
#
#   packaging/macos/bundle.sh <binary> <version> <output.dmg> [file...]
#
# Any further files (license texts) are placed in the bundle's Resources.
#
# With SIGN_IDENTITY set (a "Developer ID Application: ..." identity already in
# the keychain) the app is signed with the hardened runtime, and when
# NOTARY_APPLE_ID, NOTARY_TEAM_ID and NOTARY_PASSWORD are set too, the disk
# image is notarized and stapled. Without them the app only gets an ad-hoc
# signature, which Apple silicon needs to run it at all.
set -eu

bin="$1"; version="${2#v}"; dmg="$3"
shift 3
here=$(cd "$(dirname "$0")/.." && pwd)
app="y2e.app"

rm -rf "$app" y2e.iconset dmgroot
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

if [ -n "${SIGN_IDENTITY:-}" ]; then
    codesign --force --deep --options runtime --timestamp --sign "$SIGN_IDENTITY" "$app"
else
    codesign --force --deep --sign - "$app"
fi

mkdir dmgroot
cp -R "$app" dmgroot/
ln -s /Applications dmgroot/Applications
hdiutil create -volname "YouTube to Everything" -srcfolder dmgroot -ov -format UDZO "$dmg"
rm -rf dmgroot

if [ -n "${SIGN_IDENTITY:-}" ]; then
    codesign --force --timestamp --sign "$SIGN_IDENTITY" "$dmg"
    if [ -n "${NOTARY_APPLE_ID:-}" ] && [ -n "${NOTARY_TEAM_ID:-}" ] && [ -n "${NOTARY_PASSWORD:-}" ]; then
        xcrun notarytool submit "$dmg" --apple-id "$NOTARY_APPLE_ID" --team-id "$NOTARY_TEAM_ID" --password "$NOTARY_PASSWORD" --wait
        xcrun stapler staple "$dmg"
    fi
fi
