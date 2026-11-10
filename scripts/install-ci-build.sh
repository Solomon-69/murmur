#!/bin/zsh
# Downloads the macOS app from a GitHub Actions run of solomon-build.yml, signs
# it with your Apple Development identity (so permissions survive later
# updates), backs up the installed app once, replaces it and relaunches it in
# the background. Settings in ~/Library/Application Support are untouched.
# Usage: scripts/install-ci-build.sh <run-id>
set -euo pipefail
RUN_ID=${1:?run id}
REPO=Solomon-69/murmur-build
IDENTITY=${MURMUR_SIGN_ID:-$(security find-identity -v -p codesigning | sed -n 's/.*"\(Apple Development: [^"]*\)".*/\1/p' | head -1)}
HERE=${0:A:h}
WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT

gh run download "$RUN_ID" -R "$REPO" -n murmur-aarch64-apple-darwin -D "$WORK"
ZIP=$(find "$WORK" -name 'murmur-macos-arm64.zip' | head -1)
ditto -x -k "$ZIP" "$WORK/out"
APP=$(find "$WORK/out" -maxdepth 1 -name '*.app' | head -1)
[[ -n $APP ]] || { echo "no .app in zip"; exit 1; }
xattr -cr "$APP"
codesign --force --deep --options runtime --timestamp=none \
  --entitlements "$HERE/../src-tauri/Entitlements.plist" \
  --sign "$IDENTITY" "$APP"
codesign --verify --deep --strict "$APP"

BACKUP=~/Applications/Handy-0.10.0-backup.app
mkdir -p ~/Applications
if [[ ! -d $BACKUP && -d /Applications/Handy.app ]]; then
  ditto /Applications/Handy.app "$BACKUP"
fi
osascript -e 'tell application id "com.pais.handy" to quit' 2>/dev/null || true
# Builds before the rename run as "handy", later ones as "murmur".
for _ in {1..20}; do pgrep -x murmur >/dev/null || pgrep -x handy >/dev/null || break; sleep 0.5; done
pkill -x murmur 2>/dev/null || true
pkill -x handy 2>/dev/null || true
rm -rf /Applications/murmur.app
ditto "$APP" /Applications/murmur.app
open -g -a /Applications/murmur.app --args --start-hidden
echo "installed: $(defaults read /Applications/murmur.app/Contents/Info CFBundleShortVersionString)"

# Once murmur is running, the old Handy.app goes to the Trash (never deleted outright).
ok=0
for _ in {1..20}; do pgrep -f '/Applications/murmur.app/' >/dev/null && { ok=1; break; }; sleep 0.5; done
if [[ $ok == 1 && -d /Applications/Handy.app ]]; then
  # /usr/bin/trash moves it to the Trash silently (Finder would play a sound).
  /usr/bin/trash /Applications/Handy.app && echo "old Handy.app moved to Trash"
elif [[ $ok != 1 ]]; then
  echo "murmur did not start; Handy.app left in place" >&2
  exit 1
fi
