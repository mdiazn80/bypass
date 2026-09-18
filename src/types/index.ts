export interface Context {
  id: string;
  name: string;
  content: string;
  enabled: boolean;
  created_at: string;
  updated_at: string;
}

export interface AppConfig {
  minimize_to_tray: boolean;
  start_minimized: boolean;
  shell_integration_enabled: boolean;
  shell_integration_installed: boolean;
  /** Credential contexts served to shells; see `credential_order` for priority. */
  active_contexts: string[];
  /** Display/priority order of the credential contexts, first wins. */
  credential_order: string[];
}

export interface ShellStatus {
  enabled: boolean;
  installed: boolean;
  socket_active: boolean;
  /** Highest priority first. */
  active_contexts: string[];
  detected_shell: string | null;
  rc_path: string | null;
}

export interface CredentialContext {
  name: string;
  description: string;
  created_at: string;
  updated_at: string;
}

/** A credential variable with its `{$VAR}` references resolved. */
export interface ResolvedVar {
  key: string;
  /** The template as stored — what the editor shows and saves. */
  raw: string;
  /** The value after interpolation, which is what shells receive. */
  value: string;
  /** Set when a reference is missing, cyclic or too deep. */
  issue: string | null;
}

/** A variable of the merged set every shell receives. */
export interface MergedVar {
  key: string;
  /** The template as stored in `source`. */
  raw: string;
  /** The value after interpolation against the merged set. */
  value: string;
  /** Context that owns the key (highest-priority one defining it). */
  source: string;
  /** Lower-priority contexts that also define the key, in priority order. */
  shadowed: string[];
  issue: string | null;
}

export interface MachineInfo {
  hostname: string;
  os: string;
  kernel_version: string;
  arch: string;
  cpu_brand: string;
  cpu_count: number;
  /** Bytes. */
  total_memory: number;
  /** Bytes. */
  used_memory: number;
  /** Seconds since boot. */
  uptime: number;
}

export interface NetworkInterface {
  name: string;
  ipv4: string[];
  ipv6: string[];
  is_loopback: boolean;
}

/** Either family is null when the network has no connectivity for it. */
export interface PublicIps {
  ipv4: string | null;
  ipv6: string | null;
}

export interface SystemInfo {
  machine: MachineInfo;
  interfaces: NetworkInterface[];
}
