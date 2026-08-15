import { create } from "zustand";
import type { CredentialContext } from "../types";
import * as api from "../services/tauri";

export interface VarRow {
  key: string;
  /**
   * Decrypted value as stored, `{$VAR}` references included. This is what the
   * editor shows and saves. Loaded eagerly when a context is selected.
   */
  value: string;
  /** Set when a reference is missing, cyclic or too deep. */
  issue: string | null;
}

interface CredentialStore {
  contexts: CredentialContext[];
  selectedName: string | null;
  vars: VarRow[];
  loading: boolean;
  error: string | null;

  load: () => Promise<void>;
  selectContext: (name: string | null) => Promise<void>;
  refreshVars: () => Promise<void>;
  createContext: (name: string, description: string) => Promise<void>;
  updateContext: (name: string, description: string) => Promise<void>;
  renameContext: (oldName: string, newName: string) => Promise<void>;
  deleteContext: (name: string) => Promise<void>;
  setVar: (key: string, value: string) => Promise<void>;
  setVars: (entries: { key: string; value: string }[]) => Promise<void>;
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
    await get().refreshVars();
  },

  /**
   * Reloads every variable of the selected context, templates and resolved
   * values together. Called after any write because a single edit can change
   * the resolved value of every variable that references it.
   */
  refreshVars: async () => {
    const name = get().selectedName;
    if (!name) return;
    try {
      const rows = await api.resolveCredentialVars(name);
      // Ignore the result if the user switched contexts while loading.
      if (get().selectedName !== name) return;
      set({
        vars: rows.map((r) => ({ key: r.key, value: r.raw, issue: r.issue })),
      });
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
    await get().setVars([{ key, value }]);
  },

  /**
   * Writes several variables and refreshes once. The writes are sequential
   * because each one is a read-modify-write of the whole encrypted store, so
   * issuing them concurrently would lose entries.
   */
  setVars: async (entries: { key: string; value: string }[]) => {
    const { selectedName } = get();
    if (!selectedName) return;
    try {
      for (const { key, value } of entries) {
        await api.setCredentialVar(selectedName, key, value);
      }
      await get().refreshVars();
    } catch (err) {
      set({ error: String(err) });
    }
  },

  deleteVar: async (key: string) => {
    const { selectedName } = get();
    if (!selectedName) return;
    try {
      await api.deleteCredentialVar(selectedName, key);
      await get().refreshVars();
    } catch (err) {
      set({ error: String(err) });
    }
  },

  clearError: () => set({ error: null }),
}));
