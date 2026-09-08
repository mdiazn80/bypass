import { useEffect, useState } from "react";
import { useSystemStore } from "../stores/useSystemStore";
import "./Dashboard.css";

function formatBytes(bytes: number): string {
  if (bytes <= 0) return "—";
  const units = ["B", "KB", "MB", "GB", "TB"];
  let value = bytes;
  let unit = 0;
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024;
    unit += 1;
  }
  return `${value.toFixed(value >= 10 || unit === 0 ? 0 : 1)} ${units[unit]}`;
}

function formatUptime(seconds: number): string {
  const days = Math.floor(seconds / 86400);
  const hours = Math.floor((seconds % 86400) / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  if (days > 0) return `${days}d ${hours}h`;
  if (hours > 0) return `${hours}h ${minutes}m`;
  return `${minutes}m`;
}

function InfoRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="dashboard-info-row">
      <span className="dashboard-info-label">{label}</span>
      <span className="dashboard-info-value">{value}</span>
    </div>
  );
}

interface Address {
  value: string;
  /** Adapter the address is bound to, shown beside it when known. */
  meta?: string;
}

/** The async clipboard API is available in the webview, but a denied
 * permission or an older runtime falls through to the legacy selection path
 * rather than silently doing nothing. */
async function copyText(text: string): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(text);
    return true;
  } catch {
    // Fall through to the textarea path below.
  }

  try {
    const area = document.createElement("textarea");
    area.value = text;
    area.setAttribute("readonly", "");
    area.style.position = "fixed";
    area.style.top = "0";
    area.style.opacity = "0";
    document.body.appendChild(area);
    area.select();
    const copied = document.execCommand("copy");
    document.body.removeChild(area);
    return copied;
  } catch {
    return false;
  }
}

const COPY_FEEDBACK_MS = 1500;

function CopyButton({ value }: { value: string }) {
  const [copied, setCopied] = useState(false);

  // The tick is a transient acknowledgement, not state worth keeping.
  useEffect(() => {
    if (!copied) return;
    const timer = window.setTimeout(() => setCopied(false), COPY_FEEDBACK_MS);
    return () => window.clearTimeout(timer);
  }, [copied]);

  return (
    <button
      type="button"
      className={copied ? "dashboard-copy copied" : "dashboard-copy"}
      onClick={async () => {
        if (await copyText(value)) setCopied(true);
      }}
      title={copied ? "Copied" : `Copy ${value}`}
      aria-label={copied ? "Copied" : `Copy ${value}`}
    >
      <svg
        width="13"
        height="13"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth="2"
        strokeLinecap="round"
        strokeLinejoin="round"
        aria-hidden="true"
      >
        {copied ? (
          <polyline points="20 6 9 17 4 12" />
        ) : (
          <>
            <rect x="9" y="9" width="13" height="13" rx="2" />
            <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
          </>
        )}
      </svg>
    </button>
  );
}

/** One labelled address row, or a placeholder when nothing is assigned. */
function AddressRow({ label, addresses }: { label: string; addresses: Address[] }) {
  return (
    <div className="dashboard-addresses">
      <span className="dashboard-family">{label}</span>
      {addresses.length === 0 ? (
        <span className="dashboard-address dashboard-empty">Not assigned</span>
      ) : (
        <div className="dashboard-address-list">
          {addresses.map((address) => (
            <div key={`${address.meta ?? ""}/${address.value}`} className="dashboard-address-item">
              <span className="dashboard-address">{address.value}</span>
              {address.meta && <span className="dashboard-address-meta">{address.meta}</span>}
              <CopyButton value={address.value} />
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

export default function Dashboard() {
  const { info, publicIps, loading, error, publicError, refresh } = useSystemStore();

  useEffect(() => {
    refresh();
  }, [refresh]);

  const machine = info?.machine;

  // Loopback is never what someone means by their private address, so the
  // adapter carrying 127.0.0.1 is left out entirely.
  const privateIpv4: Address[] = (info?.interfaces ?? [])
    .filter((iface) => !iface.is_loopback)
    .flatMap((iface) => iface.ipv4.map((value) => ({ value, meta: iface.name })));

  const publicIpv4: Address[] = publicIps?.ipv4 ? [{ value: publicIps.ipv4 }] : [];
  const publicIpv6: Address[] = publicIps?.ipv6 ? [{ value: publicIps.ipv6 }] : [];

  return (
    <div className="dashboard">
      <div className="dashboard-header">
        <h2>Dashboard</h2>
        <button
          className="dashboard-refresh"
          onClick={refresh}
          disabled={loading}
          title="Refresh system information"
        >
          <span className={loading ? "dashboard-refresh-icon spinning" : "dashboard-refresh-icon"}>
            ⟳
          </span>
          {loading ? "Refreshing…" : "Refresh"}
        </button>
      </div>

      <div className="dashboard-body">
        {error && <div className="dashboard-error">{error}</div>}

        <div className="dashboard-column">
          <div className="dashboard-group">
            <h3>This computer</h3>
            <div className="dashboard-card">
              {machine ? (
                <>
                  <InfoRow label="Hostname" value={machine.hostname} />
                  <InfoRow label="Operating system" value={machine.os} />
                  <InfoRow label="Kernel" value={machine.kernel_version} />
                  <InfoRow label="Architecture" value={machine.arch} />
                  <InfoRow
                    label="CPU"
                    value={`${machine.cpu_brand} (${machine.cpu_count} cores)`}
                  />
                  <InfoRow
                    label="Memory"
                    value={`${formatBytes(machine.used_memory)} used of ${formatBytes(
                      machine.total_memory
                    )}`}
                  />
                  <InfoRow label="Uptime" value={formatUptime(machine.uptime)} />
                </>
              ) : (
                <div className="dashboard-placeholder">Loading system information…</div>
              )}
            </div>
          </div>
        </div>

        <div className="dashboard-column">
          <div className="dashboard-group">
            <h3>Network</h3>
            <div className="dashboard-card">
              <AddressRow label="IPv4 Public" addresses={publicIpv4} />
              <AddressRow label="IPv6 Public" addresses={publicIpv6} />
              <AddressRow label="IPv4 Private" addresses={privateIpv4} />
            </div>
            {publicError && (
              <div className="dashboard-note">Could not reach the public IP lookup service.</div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
