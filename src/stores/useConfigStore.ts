import { create } from "zustand";
import type { AppConfig, ShellStatus } from "../types";
import * as api from "../services/tauri";

interface ConfigStore {
  config: AppConfig;
  autostart: boolean;
  shellStatus: ShellStatus | null;
  shellError: string | null;
  load: () => Promise<void>;
  update: (minimize_to_tray: boolean, start_minimized: boolean) => Promise<void>;
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
    if (enabled) {
      await api.enableAutostart();
    } else {
      await api.disableAutostart();
    }
    return await api.isAutostartEnabled();
  } catch (err) {
    console.error("[autostart] failed to update launch-at-login:", err);
    return enabled;
  }
}

export const useConfigStore = create<ConfigStore>((set, get) => ({
  config: DEFAULT_CONFIG,
  autostart: false,
  shellStatus: null,
  shellError: null,

  load: async () => {
    const config = await api.getConfig();
    set({ config });

    try {
      const enabled = await api.isAutostartEnabled();
      set({ autostart: enabled });
    } catch {
      // Autostart status may be unavailable (e.g. unsigned dev build)
    }

    await get().loadShellStatus();
  },

  update: async (minimize_to_tray: boolean, start_minimized: boolean) => {
    const previous = get().config;
    try {
      const config = await api.updateConfig(minimize_to_tray, start_minimized);
      set({ config });
    } catch (err) {
      console.error("[config] update_config failed:", err);
      set({ config: previous });
    }
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
    try {
      const shellStatus = await api.setShellAgentEnabled(enabled);
      set({ shellStatus, shellError: null });
      // Enabling the agent also enables autostart so it is available at login.
      if (enabled) {
        const result = await setAutostartEnabled(true);
        set({ autostart: result });
      }
    } catch (err) {
      set({ shellError: String(err) });
    }
  },

  installShell: async () => {
    try {
      const shellStatus = await api.installShellIntegration();
      set({ shellStatus, shellError: null });
    } catch (err) {
      set({ shellError: String(err) });
    }
  },

  uninstallShell: async () => {
    try {
      const shellStatus = await api.uninstallShellIntegration();
      set({ shellStatus, shellError: null });
    } catch (err) {
      set({ shellError: String(err) });
    }
  },

  setActiveContext: async (name: string | null) => {
    const shellStatus = await api.setActiveContext(name);
    set({ shellStatus });
  },
}));
