#!/usr/bin/env bash
# Builds Abstractions-macos-universal.dmg from two release binaries.
#
# Usage: bundle.sh <version> <aarch64-binary> <x86_64-binary>
# Run on macOS (needs lipo, iconutil, codesign, hdiutil). Output lands in the current directory.
# The file name carries no version, so releases/latest/download/<name> stays a stable link.
# The version goes into the bundle's Info.plist.
# The app is ad-hoc signed only, not notarized.
set -euo pipefail

if [ "$#" -ne 3 ]; then
  echo "usage: $0 <version> <aarch64-binary> <x86_64-binary>" >&2
  exit 1
fi

version="$1"
arm_bin="$2"
x86_bin="$3"
here="$(cd "$(dirname "$0")" && pwd)"
out="$(pwd)"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

app="$work/Abstractions.app"
staging="$work/staging"
dmg="$out/Abstractions-macos-universal.dmg"

mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources" "$staging"

lipo -create "$arm_bin" "$x86_bin" -output "$app/Contents/MacOS/abstractions"
chmod +x "$app/Contents/MacOS/abstractions"

sed "s/__VERSION__/$version/g" "$here/Info.plist" > "$app/Contents/Info.plist"
iconutil --convert icns --output "$app/Contents/Resources/AppIcon.icns" "$here/AppIcon.iconset"

codesign --force --deep --sign - "$app"

cp -R "$app" "$staging/"
ln -s /Applications "$staging/Applications"

rm -f "$dmg"
for attempt in 1 2 3; do
  if hdiutil create -volname Abstractions -srcfolder "$staging" -ov -format UDZO "$dmg"; then
    echo "created $dmg"
    exit 0
  fi
  echo "hdiutil failed (attempt $attempt), retrying" >&2
  sleep 5
done

echo "hdiutil failed after 3 attempts" >&2
exit 1
