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

/**
 * Parses a `.env`-style file into key/value pairs. Lines without `=` yield a
 * key with an empty value so the user can fill it in manually.
 */
export function parseEnv(text: string): { key: string; value: string }[] {
  const result: { key: string; value: string }[] = [];
  for (const rawLine of text.split(/\r?\n/)) {
    let line = rawLine.trim();
    if (!line || line.startsWith("#")) continue;
    if (line.startsWith("export ")) line = line.slice("export ".length).trim();

    const eq = line.indexOf("=");
    let key: string;
    let value: string;
    if (eq === -1) {
      key = line;
      value = "";
    } else {
      key = line.slice(0, eq).trim();
      value = line.slice(eq + 1).trim();
    }
    if (!key) continue;

    if (value.length >= 2 && value.startsWith('"') && value.endsWith('"')) {
      // Undo the escaping `toEnvFile` applies to double-quoted values.
      value = value
        .slice(1, -1)
        .replace(/\\(["\\n])/g, (_, c: string) => (c === "n" ? "\n" : c));
    } else if (value.length >= 2 && value.startsWith("'") && value.endsWith("'")) {
      // Single quotes are literal in dotenv: nothing to unescape.
      value = value.slice(1, -1);
    }
    result.push({ key, value });
  }
  return result;
}

/**
 * Entries from pasted text. Unlike a `.env` import, at least one `KEY=value`
 * line is required so pasting unrelated text never creates junk keys.
 */
export function parsePasted(text: string): { key: string; value: string }[] {
  if (!/^\s*(export\s+)?[^#=\s][^=]*=/m.test(text)) return [];
  return parseEnv(text);
}

/** File-safe version of a context name. */
export function safeFileName(name: string): string {
  return name.replace(/[^a-zA-Z0-9_-]/g, "_");
}
