import { invoke } from "@tauri-apps/api/core";
import type { AppConfig, Context, CredentialContext, ShellStatus } from "../types";

export async function listContexts(): Promise<Context[]> {
  return invoke("list_contexts");
}

export async function createContext(name: string): Promise<Context> {
  return invoke("create_context", { name });
}

export async function updateContext(
  id: string,
  name?: string,
  content?: string
): Promise<Context> {
  return invoke("update_context", { id, name, content });
}

export async function deleteContext(id: string): Promise<void> {
  return invoke("delete_context", { id });
}

export async function toggleContext(id: string): Promise<Context> {
  return invoke("toggle_context", { id });
}

export async function getSystemHosts(): Promise<string> {
  return invoke("get_system_hosts");
}

export async function readFileText(path: string): Promise<string> {
  return invoke("read_file_text", { path });
}

export async function getConfig(): Promise<AppConfig> {
  return invoke("get_config");
}

export async function updateConfig(
  minimize_to_tray: boolean,
  start_minimized: boolean,
): Promise<AppConfig> {
  return invoke("update_config", { minimize_to_tray, start_minimized });
}

export async function checkBiometricAvailable(): Promise<boolean> {
  return invoke("check_biometric_available");
}

// --- Shell integration ---

export async function getShellStatus(): Promise<ShellStatus> {
  return invoke("get_shell_status");
}

export async function setActiveContext(name: string | null): Promise<ShellStatus> {
  return invoke("set_active_context", { name });
}

export async function setShellAgentEnabled(enabled: boolean): Promise<ShellStatus> {
  return invoke("set_shell_agent_enabled", { enabled });
}

export async function installShellIntegration(): Promise<ShellStatus> {
  return invoke("install_shell_integration");
}

export async function uninstallShellIntegration(): Promise<ShellStatus> {
  return invoke("uninstall_shell_integration");
}

// --- Credential contexts ---

export async function listCredentialContexts(): Promise<CredentialContext[]> {
  return invoke("list_credential_contexts");
}

export async function createCredentialContext(
  name: string,
  description: string
): Promise<void> {
  return invoke("create_credential_context", { name, description });
}

export async function updateCredentialContext(
  name: string,
  description: string
): Promise<void> {
  return invoke("update_credential_context", { name, description });
}

export async function renameCredentialContext(
  oldName: string,
  newName: string
): Promise<void> {
  return invoke("rename_credential_context", { old_name: oldName, new_name: newName });
}

export async function deleteCredentialContext(name: string): Promise<void> {
  return invoke("delete_credential_context", { name });
}

export async function listCredentialVars(context: string): Promise<string[]> {
  return invoke("list_credential_vars", { context });
}

export async function getCredentialVar(
  context: string,
  key: string
): Promise<string> {
  return invoke("get_credential_var", { context, key });
}

export async function setCredentialVar(
  context: string,
  key: string,
  value: string
): Promise<void> {
  return invoke("set_credential_var", { context, key, value });
}

export async function deleteCredentialVar(
  context: string,
  key: string
): Promise<void> {
  return invoke("delete_credential_var", { context, key });
}
