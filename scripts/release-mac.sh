#!/bin/zsh
# Signs a Handy-build CI run with the Developer ID, has Apple notarize it,
# staples it, and publishes murmur.dmg + murmur-mac.zip + SHA256SUMS.txt as a
# GitHub release on Solomon-69/murmur.
# Usage: scripts/release-mac.sh <run-id> <version>
set -euo pipefail
RUN_ID=${1:?run id}; VER=${2:?version}
ID=${MURMUR_DEVELOPER_ID:?set MURMUR_DEVELOPER_ID to your Developer ID Application identity}
HERE=${0:A:h}; ENT="$HERE/../src-tauri/Entitlements.plist"
W=$(mktemp -d); cd "$W"
gh run download "$RUN_ID" -R Solomon-69/murmur-build -n murmur-aarch64-apple-darwin -D dl
ZIP=$(find dl -name 'murmur-macos-arm64.zip' | head -1)
ditto -x -k "$ZIP" x; APP="x/murmur.app"
xattr -cr "$APP"
codesign --force --deep --options runtime --timestamp --entitlements "$ENT" --sign "$ID" "$APP"
codesign --verify --deep --strict "$APP"
ditto -c -k --keepParent "$APP" sub.zip
xcrun notarytool submit sub.zip --keychain-profile notary --wait
xcrun stapler staple "$APP"
ditto -c -k --keepParent "$APP" murmur-mac.zip
mkdir dm; ditto "$APP" dm/murmur.app; ln -s /Applications dm/Applications
hdiutil create -volname murmur -srcfolder dm -fs HFS+ -format UDZO -ov murmur.dmg >/dev/null
codesign --force --timestamp --sign "$ID" murmur.dmg
xcrun notarytool submit murmur.dmg --keychain-profile notary --wait
xcrun stapler staple murmur.dmg
shasum -a 256 murmur.dmg murmur-mac.zip > SHA256SUMS.txt
spctl --assess --type execute -vv "$APP" 2>&1 | head -2
gh release create "v$VER" -R Solomon-69/murmur --target main --title "murmur $VER" \
  --notes "murmur $VER: New shortcut: Control+Shift+Space (no more clash with Claude's Option+Space). Turn into a prompt now always gives a proper prompt layout. Also new: a wave app icon and a sound-bar menu bar icon, colour themes and glass colours, a smart prompt capsule that turns what you say into a clean prompt, a wider transcribing bar, and a quiet microphone boost so dictation never comes back empty. Automatic update checks are off. Everything runs offline on your Mac. Based on Handy by cjpais (MIT)." \
  murmur.dmg murmur-mac.zip SHA256SUMS.txt
echo "DONE $VER"; rm -rf "$W"
