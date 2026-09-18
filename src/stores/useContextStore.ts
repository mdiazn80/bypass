import { create } from "zustand";
import type { Context } from "../types";
import * as api from "../services/tauri";

export const SYSTEM_HOSTS_ID = "__system_hosts__";

interface ContextStore {
  contexts: Context[];
  selectedId: string | null;
  loading: boolean;
  systemHosts: string;
  /**
   * Context whose toggle is in flight. Applying hosts prompts for
   * administrator credentials, which can take a few seconds to appear.
   */
  togglingId: string | null;
  /** A reorder is being applied to the hosts file (needs administrator rights). */
  reordering: boolean;
  error: string | null;
  load: () => Promise<void>;
  loadSystemHosts: () => Promise<void>;
  create: (name: string) => Promise<void>;
  update: (id: string, name?: string, content?: string) => Promise<void>;
  remove: (id: string) => Promise<void>;
  toggle: (id: string) => Promise<void>;
  /** Moves the context one position up (`-1`) or down (`+1`). */
  move: (id: string, delta: -1 | 1) => Promise<void>;
  select: (id: string | null) => void;
  clearError: () => void;
}

const refreshSystemHosts = async (set: (partial: Partial<ContextStore>) => void) => {
  try {
    const systemHosts = await api.getSystemHosts();
    set({ systemHosts });
  } catch {
    // Non-critical; system hosts view will show stale content
  }
};

export const useContextStore = create<ContextStore>((set, get) => ({
  contexts: [],
  selectedId: SYSTEM_HOSTS_ID,
  loading: false,
  systemHosts: "",
  togglingId: null,
  reordering: false,
  error: null,

  load: async () => {
    set({ loading: true, error: null });
    try {
      const contexts = await api.listContexts();
      set({ contexts, loading: false });
      await refreshSystemHosts(set);
    } catch (err) {
      set({ loading: false, error: String(err) });
    }
  },

  loadSystemHosts: async () => {
    try {
      const systemHosts = await api.getSystemHosts();
      set({ systemHosts });
    } catch {
      // Non-critical
    }
  },

  create: async (name: string) => {
    try {
      const ctx = await api.createContext(name);
      set((s) => ({
        contexts: [...s.contexts, ctx],
        selectedId: ctx.id,
        error: null,
      }));
    } catch (err) {
      set({ error: String(err) });
    }
  },

  update: async (id: string, name?: string, content?: string) => {
    try {
      const updated = await api.updateContext(id, name, content);
      set((s) => ({
        contexts: s.contexts.map((c) => (c.id === id ? updated : c)),
        error: null,
      }));
      await refreshSystemHosts(set);
    } catch (err) {
      set({ error: String(err) });
    }
  },

  remove: async (id: string) => {
    try {
      await api.deleteContext(id);
      set((s) => ({
        contexts: s.contexts.filter((c) => c.id !== id),
        selectedId: s.selectedId === id ? null : s.selectedId,
        error: null,
      }));
      await refreshSystemHosts(set);
    } catch (err) {
      set({ error: String(err) });
    }
  },

  toggle: async (id: string) => {
    set({ togglingId: id });
    try {
      const updated = await api.toggleContext(id);
      set((s) => ({
        contexts: s.contexts.map((c) => (c.id === id ? updated : c)),
        error: null,
      }));
      await refreshSystemHosts(set);
    } catch (err) {
      set({ error: String(err) });
    } finally {
      set({ togglingId: null });
    }
  },

  move: async (id: string, delta: -1 | 1) => {
    const ids = get().contexts.map((c) => c.id);
    const from = ids.indexOf(id);
    const to = from + delta;
    if (from === -1 || to < 0 || to >= ids.length) return;
    [ids[from], ids[to]] = [ids[to], ids[from]];
    set({ reordering: true });
    try {
      const contexts = await api.reorderContexts(ids);
      set({ contexts, error: null });
      await refreshSystemHosts(set);
    } catch (err) {
      set({ error: String(err) });
    } finally {
      set({ reordering: false });
    }
  },

  select: (id: string | null) => {
    set({ selectedId: id });
  },

  clearError: () => {
    set({ error: null });
  },
}));
