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
}

export interface CredentialContext {
  name: string;
  description: string;
  created_at: string;
  updated_at: string;
}
