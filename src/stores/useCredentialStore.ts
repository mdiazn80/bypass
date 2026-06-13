import { create } from "zustand";
import type { CredentialContext } from "../types";
import * as api from "../services/tauri";

export interface VarRow {
  key: string;
  /** Decrypted value when revealed; null while masked. */
  value: string | null;
}

interface CredentialStore {
  contexts: CredentialContext[];
  selectedName: string | null;
  activeName: string | null;
  vars: VarRow[];
  loading: boolean;
  error: string | null;

  load: () => Promise<void>;
  selectContext: (name: string | null) => Promise<void>;
  createContext: (name: string, description: string) => Promise<void>;
  updateContext: (name: string, description: string) => Promise<void>;
  deleteContext: (name: string) => Promise<void>;
  setActive: (name: string | null) => Promise<void>;
  reveal: (key: string) => Promise<void>;
  hide: (key: string) => void;
  setVar: (key: string, value: string) => Promise<void>;
  deleteVar: (key: string) => Promise<void>;
  clearError: () => void;
}

export const useCredentialStore = create<CredentialStore>((set, get) => ({
  contexts: [],
  selectedName: null,
  activeName: null,
  vars: [],
  loading: false,
  error: null,

  load: async () => {
    set({ loading: true, error: null });
    try {
      const [contexts, activeName] = await Promise.all([
        api.listCredentialContexts(),
        api.getActiveCredentialContext(),
      ]);
      set({ contexts, activeName, loading: false });
    } catch (err) {
      set({ loading: false, error: String(err) });
    }
  },

  selectContext: async (name: string | null) => {
    set({ selectedName: name, vars: [] });
    if (!name) return;
    try {
      const keys = await api.listCredentialVars(name);
      set({ vars: keys.map((key) => ({ key, value: null })) });
    } catch (err) {
      set({ error: String(err) });
    }
  },

  createContext: async (name: string, description: string) => {
    try {
      await api.createCredentialContext(name, description);
      await get().load();
      await get().selectContext(name);
    } catch (err) {
      set({ error: String(err) });
    }
  },

  updateContext: async (name: string, description: string) => {
    try {
      await api.updateCredentialContext(name, description);
      await get().load();
    } catch (err) {
      set({ error: String(err) });
    }
  },

  deleteContext: async (name: string) => {
    try {
      await api.deleteCredentialContext(name);
      const wasSelected = get().selectedName === name;
      await get().load();
      if (wasSelected) {
        set({ selectedName: null, vars: [] });
      }
    } catch (err) {
      set({ error: String(err) });
    }
  },

  setActive: async (name: string | null) => {
    try {
      await api.setActiveCredentialContext(name);
      set({ activeName: name });
    } catch (err) {
      set({ error: String(err) });
    }
  },

  reveal: async (key: string) => {
    const { selectedName } = get();
    if (!selectedName) return;
    try {
      const value = await api.getCredentialVar(selectedName, key);
      set((s) => ({
        vars: s.vars.map((v) => (v.key === key ? { ...v, value } : v)),
      }));
    } catch (err) {
      set({ error: String(err) });
    }
  },

  hide: (key: string) => {
    set((s) => ({
      vars: s.vars.map((v) => (v.key === key ? { ...v, value: null } : v)),
    }));
  },

  setVar: async (key: string, value: string) => {
    const { selectedName } = get();
    if (!selectedName) return;
    try {
      await api.setCredentialVar(selectedName, key, value);
      set((s) => {
        const exists = s.vars.some((v) => v.key === key);
        const vars = exists
          ? s.vars.map((v) => (v.key === key ? { key, value: null } : v))
          : [...s.vars, { key, value: null }].sort((a, b) => a.key.localeCompare(b.key));
        return { vars };
      });
    } catch (err) {
      set({ error: String(err) });
    }
  },

  deleteVar: async (key: string) => {
    const { selectedName } = get();
    if (!selectedName) return;
    try {
      await api.deleteCredentialVar(selectedName, key);
      set((s) => ({ vars: s.vars.filter((v) => v.key !== key) }));
    } catch (err) {
      set({ error: String(err) });
    }
  },

  clearError: () => set({ error: null }),
}));
