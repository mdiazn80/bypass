import { create } from "zustand";
import type { CredentialContext } from "../types";
import * as api from "../services/tauri";

export interface VarRow {
  key: string;
  /** Decrypted value. Loaded eagerly when a context is selected. */
  value: string;
}

interface CredentialStore {
  contexts: CredentialContext[];
  selectedName: string | null;
  vars: VarRow[];
  loading: boolean;
  error: string | null;

  load: () => Promise<void>;
  selectContext: (name: string | null) => Promise<void>;
  createContext: (name: string, description: string) => Promise<void>;
  updateContext: (name: string, description: string) => Promise<void>;
  renameContext: (oldName: string, newName: string) => Promise<void>;
  deleteContext: (name: string) => Promise<void>;
  setVar: (key: string, value: string) => Promise<void>;
  deleteVar: (key: string) => Promise<void>;
  clearError: () => void;
}

export const useCredentialStore = create<CredentialStore>((set, get) => ({
  contexts: [],
  selectedName: null,
  vars: [],
  loading: false,
  error: null,

  load: async () => {
    set({ loading: true, error: null });
    try {
      const contexts = await api.listCredentialContexts();
      set({ contexts, loading: false });
    } catch (err) {
      set({ loading: false, error: String(err) });
    }
  },

  selectContext: async (name: string | null) => {
    set({ selectedName: name, vars: [] });
    if (!name) return;
    try {
      const keys = await api.listCredentialVars(name);
      const vars = await Promise.all(
        keys.map(async (key) => ({
          key,
          value: await api.getCredentialVar(name, key),
        }))
      );
      // Ignore the result if the user switched contexts while loading.
      if (get().selectedName === name) {
        set({ vars });
      }
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

  renameContext: async (oldName: string, newName: string) => {
    const trimmed = newName.trim();
    if (!trimmed || trimmed === oldName) return;
    try {
      await api.renameCredentialContext(oldName, trimmed);
      const wasSelected = get().selectedName === oldName;
      await get().load();
      if (wasSelected) {
        set({ selectedName: trimmed });
      }
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

  setVar: async (key: string, value: string) => {
    const { selectedName } = get();
    if (!selectedName) return;
    try {
      await api.setCredentialVar(selectedName, key, value);
      set((s) => {
        const exists = s.vars.some((v) => v.key === key);
        const vars = exists
          ? s.vars.map((v) => (v.key === key ? { key, value } : v))
          : [...s.vars, { key, value }].sort((a, b) => a.key.localeCompare(b.key));
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
