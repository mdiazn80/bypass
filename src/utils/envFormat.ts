/**
 * Serializes key/value pairs as a `.env` file. Values are written bare when
 * they are safe as-is and double-quoted (with `\`, `"` and newlines escaped)
 * otherwise, so the file round-trips through common dotenv parsers and the
 * app's own importer.
 */
export function toEnvFile(entries: { key: string; value: string }[]): string {
  return entries.map(({ key, value }) => `${key}=${envValue(value)}`).join("\n") + "\n";
}

function envValue(value: string): string {
  if (value === "" || /^[A-Za-z0-9_./:@+%,=-]+$/.test(value)) return value;
  const escaped = value
    .replace(/\\/g, "\\\\")
    .replace(/"/g, '\\"')
    .replace(/\r?\n/g, "\\n");
  return `"${escaped}"`;
}

/** File-safe version of a context name. */
export function safeFileName(name: string): string {
  return name.replace(/[^a-zA-Z0-9_-]/g, "_");
}
