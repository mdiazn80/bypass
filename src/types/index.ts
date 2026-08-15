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
  active_context: string | null;
}

export interface ShellStatus {
  enabled: boolean;
  installed: boolean;
  socket_active: boolean;
  active_context: string | null;
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
