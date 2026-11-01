/** A problem found on one line of a hosts context. `line` is 1-based. */
export interface HostsIssue {
  line: number;
  message: string;
}

/** Another context, as it appears in the sidebar (which is priority order). */
export interface OtherContext {
  name: string;
  content: string;
  enabled: boolean;
}

interface Entry {
  line: number;
  ip: string;
  hosts: string[];
}

type Family = "IPv4" | "IPv6";

function ipFamily(ip: string): Family | null {
  const v4 = ip.match(/^(\d{1,3})\.(\d{1,3})\.(\d{1,3})\.(\d{1,3})$/);
  if (v4) return v4.slice(1).every((octet) => Number(octet) <= 255) ? "IPv4" : null;
  if (!ip.includes(":")) return null;
  // The URL parser is a complete IPv6 validator; a zone index (`%en0`) is
  // valid in hosts files but not in URLs, so it is checked separately.
  const [address, zone, ...extra] = ip.split("%");
  if (extra.length > 0 || (zone !== undefined && !/^[\w.-]+$/.test(zone))) return null;
  try {
    new URL(`http://[${address}]/`);
    return "IPv6";
  } catch {
    return null;
  }
}

function hostnameProblem(host: string): string | null {
  if (host.includes("*")) return `"${host}": wildcards are not supported in hosts files`;
  const name = host.endsWith(".") ? host.slice(0, -1) : host;
  if (name.length === 0 || name.length > 253) return `"${host}" is not a valid hostname`;
  const labelsOk = name
    .split(".")
    .every((label) => /^[A-Za-z0-9_](?:[A-Za-z0-9_-]{0,61}[A-Za-z0-9_])?$/.test(label));
  return labelsOk ? null : `"${host}" is not a valid hostname`;
}

/** Entry lines with their inline comment removed; blank and comment lines skipped. */
function entries(content: string): Entry[] {
  const result: Entry[] = [];
  content.split("\n").forEach((raw, index) => {
    const fields = raw.replace(/#.*$/, "").trim().split(/\s+/).filter(Boolean);
    if (fields.length === 0) return;
    const [ip, ...hosts] = fields;
    result.push({ line: index + 1, ip, hosts });
  });
  return result;
}

/**
 * Warnings for the hosts entries of one context: malformed lines, hostnames
 * repeated within it, and hostnames that an enabled context also defines.
 * `others` lists every other context in sidebar order and `position` is this
 * context's index in that order, so the warning can say which one wins: the
 * resolver takes the first match, and the managed block follows the sidebar.
 */
export function validateHosts(
  content: string,
  others: OtherContext[],
  position: number
): HostsIssue[] {
  const issues: HostsIssue[] = [];
  const seen = new Map<string, number>();

  // Hostnames from enabled contexts, keyed by family: an IPv4 and an IPv6
  // entry for the same name do not shadow each other.
  const elsewhere = new Map<string, { name: string; wins: boolean }>();
  others.forEach((other, index) => {
    if (!other.enabled) return;
    for (const entry of entries(other.content)) {
      const family = ipFamily(entry.ip);
      if (!family) continue;
      for (const host of entry.hosts) {
        const key = `${family} ${host.toLowerCase()}`;
        if (!elsewhere.has(key)) elsewhere.set(key, { name: other.name, wins: index < position });
      }
    }
  });

  for (const entry of entries(content)) {
    const family = ipFamily(entry.ip);
    if (!family) {
      issues.push({ line: entry.line, message: `"${entry.ip}" is not a valid IPv4 or IPv6 address` });
      continue;
    }
    if (entry.hosts.length === 0) {
      issues.push({ line: entry.line, message: `No hostname after ${entry.ip}` });
      continue;
    }
    for (const host of entry.hosts) {
      const problem = hostnameProblem(host);
      if (problem) {
        issues.push({ line: entry.line, message: problem });
        continue;
      }
      const key = `${family} ${host.toLowerCase()}`;
      const firstLine = seen.get(key);
      if (firstLine !== undefined) {
        issues.push({
          line: entry.line,
          message: `${host} is already defined on line ${firstLine}, which wins`,
        });
        continue;
      }
      seen.set(key, entry.line);
      const conflict = elsewhere.get(key);
      if (conflict) {
        issues.push({
          line: entry.line,
          message: conflict.wins
            ? `${host} is also defined in "${conflict.name}", which comes first and wins`
            : `${host} is also defined in "${conflict.name}"; this context comes first and wins`,
        });
      }
    }
  }
  return issues;
}
