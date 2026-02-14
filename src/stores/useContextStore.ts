import { create } from "zustand";
import type { Context } from "../types";
import * as api from "../services/tauri";

export const SYSTEM_HOSTS_ID = "__system_hosts__";

interface ContextStore {
  contexts: Context[];
  selectedId: string | null;
  loading: boolean;
  systemHosts: string;
  load: () => Promise<void>;
  loadSystemHosts: () => Promise<void>;
  create: (name: string) => Promise<void>;
  importContext: (name: string, content: string) => Promise<void>;
  update: (id: string, name?: string, content?: string) => Promise<void>;
  remove: (id: string) => Promise<void>;
  toggle: (id: string) => Promise<void>;
  select: (id: string | null) => void;
}

const refreshSystemHosts = async (set: (partial: Partial<ContextStore>) => void) => {
  try {
    const systemHosts = await api.getSystemHosts();
    set({ systemHosts });
  } catch {
    // ignore
  }
};

export const useContextStore = create<ContextStore>((set) => ({
  contexts: [],
  selectedId: SYSTEM_HOSTS_ID,
  loading: false,
  systemHosts: "",

  load: async () => {
    set({ loading: true });
    const contexts = await api.listContexts();
    set({ contexts, loading: false });
    // Also load system hosts
    try {
      const systemHosts = await api.getSystemHosts();
      set({ systemHosts });
    } catch {
      // ignore
    }
  },

  loadSystemHosts: async () => {
    try {
      const systemHosts = await api.getSystemHosts();
      set({ systemHosts });
    } catch {
      // ignore
    }
  },

  create: async (name: string) => {
    const ctx = await api.createContext(name);
    set((s) => ({
      contexts: [...s.contexts, ctx],
      selectedId: ctx.id,
    }));
  },

  importContext: async (name: string, content: string) => {
    const ctx = await api.createContext(name);
    const updated = await api.updateContext(ctx.id, undefined, content);
    set((s) => ({
      contexts: [...s.contexts, updated],
      selectedId: updated.id,
    }));
  },

  update: async (id: string, name?: string, content?: string) => {
    const updated = await api.updateContext(id, name, content);
    set((s) => ({
      contexts: s.contexts.map((c) => (c.id === id ? updated : c)),
    }));
  },

  remove: async (id: string) => {
    await api.deleteContext(id);
    set((s) => ({
      contexts: s.contexts.filter((c) => c.id !== id),
      selectedId: s.selectedId === id ? null : s.selectedId,
    }));
    refreshSystemHosts(set);
  },

  toggle: async (id: string) => {
    const updated = await api.toggleContext(id);
    set((s) => ({
      contexts: s.contexts.map((c) => (c.id === id ? updated : c)),
    }));
    refreshSystemHosts(set);
  },

  select: (id: string | null) => {
    set({ selectedId: id });
  },
}));
