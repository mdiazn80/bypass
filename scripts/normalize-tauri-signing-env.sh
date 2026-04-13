# shellcheck shell=bash
# Normaliza secretos pegados con saltos de línea (GitHub / copiar desde Keychain).
# - TAURI_SIGNING_PRIVATE_KEY: Base64 estricto; \n rompe el decode.
# - APPLE_SIGNING_IDENTITY: Tauri exige que coincida con el cert del .p12; \n/espacios raros provocan mismatch.
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
