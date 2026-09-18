import { useCallback, useEffect, useRef, useState } from "react";
import { useCredentialStore, ACTIVE_VARS_ID } from "../stores/useCredentialStore";
import { useConfigStore } from "../stores/useConfigStore";
import FileDropZone from "./FileDropZone";
import ExportButton from "./ExportButton";
import { safeFileName, toEnvFile } from "../utils/envFormat";
import "./ContextEditor.css";
import "./CredentialEditor.css";

/**
 * Parses a `.env`-style file into key/value pairs. Lines without `=` yield a
 * key with an empty value so the user can fill it in manually.
 */
function parseEnv(text: string): { key: string; value: string }[] {
  const result: { key: string; value: string }[] = [];
  for (const rawLine of text.split(/\r?\n/)) {
    let line = rawLine.trim();
    if (!line || line.startsWith("#")) continue;
    if (line.startsWith("export ")) line = line.slice("export ".length).trim();

    const eq = line.indexOf("=");
    let key: string;
    let value: string;
    if (eq === -1) {
      key = line;
      value = "";
    } else {
      key = line.slice(0, eq).trim();
      value = line.slice(eq + 1).trim();
    }
    if (!key) continue;

    if (
      value.length >= 2 &&
      ((value.startsWith('"') && value.endsWith('"')) ||
        (value.startsWith("'") && value.endsWith("'")))
    ) {
      value = value.slice(1, -1);
    }
    result.push({ key, value });
  }
  return result;
}

/**
 * Entries from pasted text. Unlike a `.env` import, at least one `KEY=value`
 * line is required so pasting unrelated text never creates junk keys.
 */
function parsePasted(text: string): { key: string; value: string }[] {
  if (!/^\s*(export\s+)?[^#=\s][^=]*=/m.test(text)) return [];
  return parseEnv(text);
}

const PASTE_FEEDBACK_MS = 2000;

/**
 * Read-only view of the merged variables every shell receives: all active
 * contexts layered by priority. The credential counterpart of System Hosts.
 */
function ActiveVarsSummary() {
  const { contexts, activeVars, selectContext } = useCredentialStore();
  const activeContexts = useConfigStore((s) => s.shellStatus?.active_contexts ?? []);
  // Sidebar order is the priority order; keep only the active ones.
  const active = contexts.filter((c) => activeContexts.includes(c.name));

  return (
    <div className="editor">
      <div className="editor-header">
        <h2 className="editor-name">Active Variables</h2>
        <div className="editor-header-actions">
          <ExportButton
            defaultName="active.env"
            filterName="Env file"
            extensions={["env", "txt"]}
            getContent={() => toEnvFile(activeVars)}
            title="Export the merged variables as KEY=value"
          />
          <span className="editor-status">Read-only</span>
        </div>
      </div>
      <div className="cred-body">
        {active.length === 0 ? (
          <p className="cred-hint">
            No credential context is active. Turn one on in the sidebar and its
            variables will be served to your shells.
          </p>
        ) : (
          <>
            <div className="cred-summary-order">
              <span className="cred-summary-label">Priority</span>
              {active.map((c, i) => (
                <button
                  key={c.name}
                  className="cred-summary-chip"
                  onClick={() => selectContext(c.name)}
                  title={`Open ${c.name}`}
                >
                  <span className="cred-summary-rank">{i + 1}</span>
                  {c.name}
                </button>
              ))}
            </div>
            <p className="cred-hint">
              When several active contexts define the same variable, the one
              higher in the sidebar wins. Use the arrows next to a context to
              change its priority.
            </p>
            <div className="cred-vars">
              {activeVars.map((v) => (
                <div className="cred-var-item" key={v.key}>
                  <div className="cred-var-row">
                    <span className="cred-var-key">{v.key}</span>
                    <span className="cred-var-value" title={v.raw !== v.value ? v.raw : undefined}>
                      {v.value}
                    </span>
                    <button
                      className="cred-var-source"
                      onClick={() => selectContext(v.source)}
                      title={`Defined in ${v.source}`}
                    >
                      {v.source}
                    </button>
                  </div>
                  {v.shadowed.length > 0 && (
                    <div className="cred-var-shadowed">
                      overrides {v.shadowed.join(", ")}
                    </div>
                  )}
                  {v.issue && <div className="cred-var-issue">{v.issue}</div>}
                </div>
              ))}
              {activeVars.length === 0 && (
                <p className="cred-hint">The active contexts have no variables yet.</p>
              )}
            </div>
          </>
        )}
      </div>
    </div>
  );
}

export default function CredentialEditor() {
  const { contexts, selectedName, vars, updateContext, renameContext, setVar, setVars, deleteVar } =
    useCredentialStore();

  const selected = contexts.find((c) => c.name === selectedName);
  const [description, setDescription] = useState("");
  const [name, setName] = useState("");
  const [editingName, setEditingName] = useState(false);
  const [newKey, setNewKey] = useState("");
  const [newValue, setNewValue] = useState("");
  // Per-row editable drafts of the values, keyed by variable name.
  const [drafts, setDrafts] = useState<Record<string, string>>({});
  const [pasteNotice, setPasteNotice] = useState<string | null>(null);
  const pasteTimer = useRef<number | undefined>(undefined);

  useEffect(() => {
    setDescription(selected?.description ?? "");
    setName(selected?.name ?? "");
    setEditingName(false);
  }, [selectedName, selected?.description]);

  // Reset drafts whenever the persisted values change (context switch or save).
  useEffect(() => {
    const next: Record<string, string> = {};
    for (const v of vars) next[v.key] = v.value;
    setDrafts(next);
  }, [vars]);

  const handleImportEnv = useCallback(
    (content: string) => {
      void setVars(parseEnv(content));
    },
    [setVars]
  );

  const showPasteNotice = useCallback((message: string) => {
    setPasteNotice(message);
    window.clearTimeout(pasteTimer.current);
    pasteTimer.current = window.setTimeout(() => setPasteNotice(null), PASTE_FEEDBACK_MS);
  }, []);

  useEffect(() => () => window.clearTimeout(pasteTimer.current), []);

  /** Adds every `KEY=value` line of `text` to the selected context. */
  const importPasted = useCallback(
    async (text: string): Promise<boolean> => {
      const entries = parsePasted(text);
      if (entries.length === 0) {
        showPasteNotice("Clipboard has no KEY=value lines");
        return false;
      }
      await setVars(entries);
      showPasteNotice(
        entries.length === 1
          ? `Pasted ${entries[0].key}`
          : `Pasted ${entries.length} variables`
      );
      return true;
    },
    [setVars, showPasteNotice]
  );

  const handlePasteButton = async () => {
    let text: string;
    try {
      text = await navigator.clipboard.readText();
    } catch {
      showPasteNotice("Clipboard unavailable — press Cmd/Ctrl+V instead");
      return;
    }
    await importPasted(text);
  };

  // Cmd/Ctrl+V anywhere in the editor that is not a text field pastes into
  // the selected context. Fields keep their normal paste behaviour.
  useEffect(() => {
    if (!selectedName || selectedName === ACTIVE_VARS_ID) return;
    const onPaste = (e: ClipboardEvent) => {
      const target = e.target as HTMLElement | null;
      if (
        target instanceof HTMLInputElement ||
        target instanceof HTMLTextAreaElement ||
        target?.isContentEditable
      ) {
        return;
      }
      const text = e.clipboardData?.getData("text") ?? "";
      if (!text) return;
      e.preventDefault();
      void importPasted(text);
    };
    document.addEventListener("paste", onPaste);
    return () => document.removeEventListener("paste", onPaste);
  }, [selectedName, importPasted]);

  if (selectedName === ACTIVE_VARS_ID) {
    return <ActiveVarsSummary />;
  }

  if (!selectedName || !selected) {
    return (
      <div className="editor-empty">
        <p>Select a credential context from the sidebar, or create one to store environment variables securely.</p>
      </div>
    );
  }

  const handleNameConfirm = () => {
    setEditingName(false);
    const trimmed = name.trim();
    if (trimmed && trimmed !== selected.name) {
      void renameContext(selected.name, trimmed);
    }
  };

  const handleNameCancel = () => {
    setName(selected.name);
    setEditingName(false);
  };

  const handleDescriptionSave = () => {
    if (description !== selected.description) {
      updateContext(selected.name, description);
    }
  };

  const handleAddVar = async () => {
    const key = newKey.trim();
    if (!key) return;
    await setVar(key, newValue);
    setNewKey("");
    setNewValue("");
  };

  const handleSaveVar = (key: string) => {
    void setVar(key, drafts[key] ?? "");
  };

  return (
    <div className="editor">
      <div className="editor-header">
        {editingName ? (
          <div className="editor-name-edit">
            <input
              className="editor-name-input"
              autoFocus
              value={name}
              onChange={(e) => setName(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") handleNameConfirm();
                if (e.key === "Escape") handleNameCancel();
              }}
            />
            <button className="editor-name-confirm" onClick={handleNameConfirm} title="Confirm rename">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                <polyline points="20 6 9 17 4 12"/>
              </svg>
            </button>
            <button className="editor-name-cancel" onClick={handleNameCancel} title="Cancel">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
              </svg>
            </button>
          </div>
        ) : (
          <div className="editor-name-row">
            <h2 className="editor-name">{selected.name}</h2>
            <button className="editor-rename-btn" onClick={() => setEditingName(true)} title="Rename">
              <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/>
                <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/>
              </svg>
            </button>
          </div>
        )}
        {!editingName && (
          <div className="editor-header-actions">
            <ExportButton
              defaultName={`${safeFileName(selected.name)}.env`}
              filterName="Env file"
              extensions={["env", "txt"]}
              // Templates are exported as stored, `{$VAR}` references included,
              // so the file re-imports without freezing them.
              getContent={() => toEnvFile(vars)}
              title="Export these variables as KEY=value"
            />
          </div>
        )}
      </div>

      <div className="cred-body">
        <input
          className="cred-description"
          value={description}
          placeholder="Description (optional)"
          onChange={(e) => setDescription(e.target.value)}
          onBlur={handleDescriptionSave}
          onKeyDown={(e) => {
            if (e.key === "Enter") e.currentTarget.blur();
          }}
        />

        <p className="cred-hint">
          A <code>.bypass-context</code> file in a project directory overrides the global active context for the CLI.
        </p>

        <p className="cred-hint">
          Use <code>{"{$VAR}"}</code> to reuse another variable of this context &mdash; for
          example <code>{"{$APP_PATH}/config"}</code>. References resolve when the value reaches
          your shell, so editing the source updates everything derived from it.
        </p>

        <div className="cred-vars">
          {vars.map((v) => {
            const draft = drafts[v.key] ?? v.value;
            const dirty = draft !== v.value;
            // Reported against the saved template, so it is hidden while the row
            // is dirty rather than shown next to an edited value.
            const issue = dirty ? null : v.issue;
            return (
              <div className="cred-var-item" key={v.key}>
                <div className="cred-var-row">
                  <span className="cred-var-key">{v.key}</span>
                  <input
                    className="cred-var-value-input"
                    value={draft}
                    spellCheck={false}
                    autoComplete="off"
                    onChange={(e) =>
                      setDrafts((d) => ({ ...d, [v.key]: e.target.value }))
                    }
                    onKeyDown={(e) => {
                      if (e.key === "Enter" && dirty) handleSaveVar(v.key);
                    }}
                  />
                  <button
                    className="cred-var-btn save"
                    title="Save new value"
                    disabled={!dirty}
                    onClick={() => handleSaveVar(v.key)}
                  >
                    save
                  </button>
                  <button
                    className="cred-var-btn danger"
                    title="Delete variable"
                    onClick={() => deleteVar(v.key)}
                  >
                    ×
                  </button>
                </div>
                {issue && <div className="cred-var-issue">{issue}</div>}
              </div>
            );
          })}

          {vars.length === 0 && (
            <div className="editor-dropzone-wrap">
              <FileDropZone
                title="Drag & drop a .env file"
                hint="or click to browse — keys without a value can be filled in below"
                onImport={handleImportEnv}
              />
            </div>
          )}
        </div>

        <div className="cred-add">
          <input
            className="cred-add-key"
            value={newKey}
            placeholder="KEY"
            onChange={(e) => setNewKey(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter") handleAddVar();
            }}
            onPaste={(e) => {
              // A pasted `KEY=value` goes straight into the context; a bare
              // key is pasted into the field as usual.
              const text = e.clipboardData.getData("text");
              if (parsePasted(text).length === 0) return;
              e.preventDefault();
              void importPasted(text);
            }}
          />
          <input
            className="cred-add-value"
            value={newValue}
            placeholder="value"
            onChange={(e) => setNewValue(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter") handleAddVar();
            }}
          />
          <button className="cred-add-btn" onClick={handleAddVar}>
            Add
          </button>
          <button
            className="cred-paste-btn"
            onClick={handlePasteButton}
            title="Paste KEY=value lines from the clipboard into this context"
          >
            Paste
          </button>
        </div>
        {pasteNotice && <div className="cred-paste-notice">{pasteNotice}</div>}
      </div>
    </div>
  );
}
