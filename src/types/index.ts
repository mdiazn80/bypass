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
