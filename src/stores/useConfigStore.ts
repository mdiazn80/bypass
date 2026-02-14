import { create } from "zustand";
import type { AppConfig } from "../types";
import * as api from "../services/tauri";

interface ConfigStore {
  config: AppConfig;
  autostart: boolean;
  load: () => Promise<void>;
  update: (minimize_to_tray?: boolean, start_minimized?: boolean) => Promise<void>;
  setAutostart: (enabled: boolean) => Promise<void>;
}

export const useConfigStore = create<ConfigStore>((set) => ({
  config: { minimize_to_tray: true, start_minimized: false },
  autostart: false,

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
  },

  update: async (minimize_to_tray?: boolean, start_minimized?: boolean) => {
    const config = await api.updateConfig(minimize_to_tray, start_minimized);
    set({ config });
  },

  setAutostart: async (enabled: boolean) => {
    try {
      const { enable, disable } = await import("@tauri-apps/plugin-autostart");
      if (enabled) {
        await enable();
      } else {
        await disable();
      }
      set({ autostart: enabled });
    } catch {
      // Autostart plugin may not be available
    }
  },
}));
