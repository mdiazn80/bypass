# shellcheck shell=bash
# Normalize secrets pasted with line breaks (GitHub / copied from Keychain).
# - TAURI_SIGNING_PRIVATE_KEY: strict Base64; \n breaks decoding.
# - APPLE_SIGNING_IDENTITY: Tauri requires it to match the .p12 cert; \n/stray spaces cause a mismatch.
# - APPLE_ID / APPLE_PASSWORD / APPLE_TEAM_ID: notarization credentials; \n breaks notarytool.
if [ -n "${TAURI_SIGNING_PRIVATE_KEY:-}" ]; then
  TAURI_SIGNING_PRIVATE_KEY="${TAURI_SIGNING_PRIVATE_KEY//$'\r'/}"
  TAURI_SIGNING_PRIVATE_KEY="${TAURI_SIGNING_PRIVATE_KEY//$'\n'/}"
  export TAURI_SIGNING_PRIVATE_KEY
fi
if [ -n "${APPLE_SIGNING_IDENTITY:-}" ]; then
  APPLE_SIGNING_IDENTITY="${APPLE_SIGNING_IDENTITY//$'\r'/}"
  APPLE_SIGNING_IDENTITY="${APPLE_SIGNING_IDENTITY//$'\n'/}"
  export APPLE_SIGNING_IDENTITY
fi
if [ -n "${APPLE_ID:-}" ]; then
  APPLE_ID="${APPLE_ID//$'\r'/}"
  APPLE_ID="${APPLE_ID//$'\n'/}"
  export APPLE_ID
fi
if [ -n "${APPLE_PASSWORD:-}" ]; then
  APPLE_PASSWORD="${APPLE_PASSWORD//$'\r'/}"
  APPLE_PASSWORD="${APPLE_PASSWORD//$'\n'/}"
  export APPLE_PASSWORD
fi
if [ -n "${APPLE_TEAM_ID:-}" ]; then
  APPLE_TEAM_ID="${APPLE_TEAM_ID//$'\r'/}"
  APPLE_TEAM_ID="${APPLE_TEAM_ID//$'\n'/}"
  export APPLE_TEAM_ID
fi
