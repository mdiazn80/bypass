# shellcheck shell=bash
# Tauri decodifica TAURI_SIGNING_PRIVATE_KEY como Base64 estricto; saltos de línea (símbolo ASCII10)
# al pegar el secreto en GitHub rompen el decode. Quitar \r y \n del valor.
if [ -n "${TAURI_SIGNING_PRIVATE_KEY:-}" ]; then
  TAURI_SIGNING_PRIVATE_KEY="${TAURI_SIGNING_PRIVATE_KEY//$'\r'/}"
  TAURI_SIGNING_PRIVATE_KEY="${TAURI_SIGNING_PRIVATE_KEY//$'\n'/}"
  export TAURI_SIGNING_PRIVATE_KEY
fi
