import { create } from "zustand";
import type { PublicIps, SystemInfo } from "../types";
import * as api from "../services/tauri";

interface SystemStore {
  info: SystemInfo | null;
  publicIps: PublicIps | null;
  /** True while a refresh is in flight, including the initial load. */
  loading: boolean;
  error: string | null;
  /** Set when the public lookup fails; local info stays usable regardless. */
  publicError: string | null;
  refresh: () => Promise<void>;
}

export const useSystemStore = create<SystemStore>((set) => ({
  info: null,
  publicIps: null,
  loading: false,
  error: null,
  publicError: null,

  refresh: async () => {
    set({ loading: true });

    // The local snapshot is instant; the public lookup goes over the network
    // and is allowed to fail on its own without blanking the rest of the view.
    const [local, remote] = await Promise.allSettled([
      api.getSystemInfo(),
      api.getPublicIps(),
    ]);

    set({
      loading: false,
      ...(local.status === "fulfilled"
        ? { info: local.value, error: null }
        : { error: String(local.reason) }),
      ...(remote.status === "fulfilled"
        ? { publicIps: remote.value, publicError: null }
        : { publicIps: null, publicError: String(remote.reason) }),
    });
  },
}));
