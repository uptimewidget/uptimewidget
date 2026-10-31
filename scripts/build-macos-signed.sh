#!/usr/bin/env bash
# Builds a signed and notarized macOS release locally.
# Requires the Developer ID certificate in the login keychain and the
# app-specific password stored as generic password "uptimewidget-notarize".
set -euo pipefail

export APPLE_TEAM_ID="${APPLE_TEAM_ID:-B5A364J7AV}"
export APPLE_ID="${APPLE_ID:-peter.bagi@gmail.com}"
# Use the certificate's SHA-1 hash if several identities share the same name.
export APPLE_SIGNING_IDENTITY="${APPLE_SIGNING_IDENTITY:-Developer ID Application: Peter Bagi (B5A364J7AV)}"
APPLE_PASSWORD="$(security find-generic-password -s uptimewidget-notarize -a "$APPLE_ID" -w)"
export APPLE_PASSWORD

cd "$(dirname "$0")/.."
npm run tauri build -- --bundles app,dmg "$@"

# Tauri notarizes the .app only; notarize and staple the DMG as well.
for dmg in src-tauri/target/*release/bundle/dmg/*.dmg; do
  xcrun notarytool submit "$dmg" --apple-id "$APPLE_ID" --team-id "$APPLE_TEAM_ID" \
    --password "$APPLE_PASSWORD" --wait
  xcrun stapler staple "$dmg"
done
