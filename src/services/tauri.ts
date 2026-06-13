import { invoke } from "@tauri-apps/api/core";
import type { AppConfig, Context } from "../types";

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

export async function getConfig(): Promise<AppConfig> {
  return invoke("get_config");
}

export async function updateConfig(
  minimize_to_tray?: boolean,
  start_minimized?: boolean
): Promise<AppConfig> {
  return invoke("update_config", { minimize_to_tray, start_minimized });
}

export async function checkBiometricAvailable(): Promise<boolean> {
  return invoke("check_biometric_available");
}
