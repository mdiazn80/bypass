import { create } from "zustand";
import type { AppConfig, ShellStatus } from "../types";
import * as api from "../services/tauri";

interface ConfigStore {
  config: AppConfig;
  autostart: boolean;
  shellStatus: ShellStatus | null;
  load: () => Promise<void>;
  update: (minimize_to_tray?: boolean, start_minimized?: boolean) => Promise<void>;
  setAutostart: (enabled: boolean) => Promise<void>;
  loadShellStatus: () => Promise<void>;
  setShellAgentEnabled: (enabled: boolean) => Promise<void>;
  installShell: () => Promise<void>;
  uninstallShell: () => Promise<void>;
  setActiveContext: (name: string | null) => Promise<void>;
}

const DEFAULT_CONFIG: AppConfig = {
  minimize_to_tray: true,
  start_minimized: false,
  shell_integration_enabled: false,
  shell_integration_installed: false,
  active_context: null,
};

async function setAutostartEnabled(enabled: boolean): Promise<boolean> {
  try {
    const { enable, disable } = await import("@tauri-apps/plugin-autostart");
    if (enabled) {
      await enable();
    } else {
      await disable();
    }
    return enabled;
  } catch {
    // Autostart plugin may not be available
    return enabled;
  }
}

export const useConfigStore = create<ConfigStore>((set, get) => ({
  config: DEFAULT_CONFIG,
  autostart: false,
  shellStatus: null,

  load: async () => {
    const config = await api.getConfig();
    set({ config });

    try {
      const { isEnabled } = await import("@tauri-apps/plugin-autostart");
      const enabled = await isEnabled();
      set({ autostart: enabled });
    } catch {
      // Autostart plugin may not be available
    }

    await get().loadShellStatus();
  },

  update: async (minimize_to_tray?: boolean, start_minimized?: boolean) => {
    const config = await api.updateConfig(minimize_to_tray, start_minimized);
    set({ config });
  },

  setAutostart: async (enabled: boolean) => {
    const result = await setAutostartEnabled(enabled);
    set({ autostart: result });
  },

  loadShellStatus: async () => {
    try {
      const shellStatus = await api.getShellStatus();
      set({ shellStatus });
    } catch {
      // Ignore: shell integration is optional
    }
  },

  setShellAgentEnabled: async (enabled: boolean) => {
    const shellStatus = await api.setShellAgentEnabled(enabled);
    set({ shellStatus });
    // Enabling the agent also enables autostart so it is available at login.
    if (enabled) {
      const result = await setAutostartEnabled(true);
      set({ autostart: result });
    }
  },

  installShell: async () => {
    const shellStatus = await api.installShellIntegration();
    set({ shellStatus });
  },

  uninstallShell: async () => {
    const shellStatus = await api.uninstallShellIntegration();
    set({ shellStatus });
  },

  setActiveContext: async (name: string | null) => {
    const shellStatus = await api.setActiveContext(name);
    set({ shellStatus });
  },
}));
