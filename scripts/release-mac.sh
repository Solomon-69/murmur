#!/bin/zsh
# Signs a murmur CI run (solomon-build.yml) with the Developer ID, has Apple notarize it,
# staples it, and publishes murmur.dmg + murmur-mac.zip + SHA256SUMS.txt as a
# GitHub release on Solomon-69/murmur.
# Usage: scripts/release-mac.sh <run-id> <version>
set -euo pipefail
RUN_ID=${1:?run id}; VER=${2:?version}
ID=${MURMUR_DEVELOPER_ID:?set MURMUR_DEVELOPER_ID to your Developer ID Application identity}
HERE=${0:A:h}; ENT="$HERE/../src-tauri/Entitlements.plist"
W=$(mktemp -d); cd "$W"
gh run download "$RUN_ID" -R Solomon-69/murmur -n murmur-aarch64-apple-darwin -D dl
ZIP=$(find dl -name 'murmur-macos-arm64.zip' | head -1)
ditto -x -k "$ZIP" x; APP="x/murmur.app"
xattr -cr "$APP"
codesign --force --deep --options runtime --timestamp --entitlements "$ENT" --sign "$ID" "$APP"
codesign --verify --deep --strict "$APP"
ditto -c -k --keepParent "$APP" sub.zip
xcrun notarytool submit sub.zip --keychain-profile notary --wait
xcrun stapler staple "$APP"
ditto -c -k --keepParent "$APP" murmur-mac.zip
# Pretty install window (shared Mac-Apps tool), signed, notarized, stapled.
python3 "$HOME/Code/Mac-Apps/tools/dmg/make_dmg.py" --app "$APP" --name murmur --style murmur --color "#4a4a50" --out murmur.dmg >/dev/null
codesign --force --timestamp --sign "$ID" murmur.dmg
xcrun notarytool submit murmur.dmg --keychain-profile notary --wait
xcrun stapler staple murmur.dmg
# Update feed for the in-app "Check for updates" button: signed app bundle + latest.json.
KEY=${MURMUR_UPDATER_KEY:-$HOME/.tauri/murmur-updater.key}
tar -czf murmur.app.tar.gz -C x murmur.app
(cd "$HERE/.." && bunx tauri signer sign -f "$KEY" -p "" "$W/murmur.app.tar.gz" >/dev/null)
NOTES=${MURMUR_NOTES:-"murmur $VER"}
python3 - "$VER" "$NOTES" <<'PY'
import json, sys, datetime
ver, notes = sys.argv[1], sys.argv[2]
sig = open("murmur.app.tar.gz.sig").read().strip()
url = f"https://github.com/Solomon-69/murmur/releases/download/v{ver}/murmur.app.tar.gz"
json.dump({"version": ver, "notes": notes,
           "pub_date": datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
           "platforms": {"darwin-aarch64": {"signature": sig, "url": url}}}, open("latest.json", "w"), indent=2)
PY
shasum -a 256 murmur.dmg murmur-mac.zip murmur.app.tar.gz > SHA256SUMS.txt
spctl --assess --type execute -vv "$APP" 2>&1 | head -2
gh release create "v$VER" -R Solomon-69/murmur --target main --title "murmur $VER" --notes "$NOTES" \
  murmur.dmg murmur-mac.zip murmur.app.tar.gz murmur.app.tar.gz.sig latest.json SHA256SUMS.txt
echo "DONE $VER"; rm -rf "$W"
