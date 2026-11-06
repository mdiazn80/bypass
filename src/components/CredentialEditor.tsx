import { useCallback, useEffect, useRef, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { useCredentialStore, ACTIVE_VARS_ID } from "../stores/useCredentialStore";
import { useConfigStore } from "../stores/useConfigStore";
import FileDropZone from "./FileDropZone";
import ExportButton from "./ExportButton";
import { formatBytes, parseEnv, parsePasted, safeFileName, toEnvFile } from "../utils/envFormat";
import "./ContextEditor.css";
import "./CredentialEditor.css";

const PASTE_FEEDBACK_MS = 2000;

/** Editable copy of the saved values, keyed by variable name. */
function draftsFrom(vars: { key: string; value: string }[]): Record<string, string> {
  const drafts: Record<string, string> = {};
  for (const v of vars) drafts[v.key] = v.value;
  return drafts;
}

function FileIcon() {
  return (
    <svg className="cred-file-icon" width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
      <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
      <polyline points="14 2 14 8 20 8"/>
    </svg>
  );
}

/** Last component of a path chosen in the native file dialog. */
function baseName(path: string): string {
  return path.split(/[\\/]/).pop() ?? path;
}

/**
 * Read-only view of the merged variables every shell receives: all active
 * contexts layered by priority. The credential counterpart of System Hosts.
 */
function ActiveVarsSummary() {
  const { contexts, activeVars, selectContext } = useCredentialStore();
  // The fallback stays outside the selector: returning a new `[]` from it makes
  // the store look changed on every read, which loops until React gives up.
  const activeContexts = useConfigStore((s) => s.shellStatus?.active_contexts) ?? [];
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
            title="Export the merged variables as KEY=value. File variables export their path, not their content"
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
                    {v.file && <FileIcon />}
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
  const { contexts, selectedName, vars, updateContext, renameContext, setVar, setVars, setFile, deleteVar } =
    useCredentialStore();

  const selected = contexts.find((c) => c.name === selectedName);
  const [description, setDescription] = useState(selected?.description ?? "");
  const [name, setName] = useState(selected?.name ?? "");
  const [editingName, setEditingName] = useState(false);
  const [newKey, setNewKey] = useState("");
  const [newValue, setNewValue] = useState("");
  const [addKind, setAddKind] = useState<"text" | "file">("text");
  const [newFilePath, setNewFilePath] = useState<string | null>(null);
  // Per-row editable drafts of the values, keyed by variable name.
  const [drafts, setDrafts] = useState(() => draftsFrom(vars));
  const [pasteNotice, setPasteNotice] = useState<string | null>(null);
  const pasteTimer = useRef<number | undefined>(undefined);

  // Reset the header fields when another context is selected or its saved
  // description changes. Adjusting state during render (instead of in an
  // effect) avoids rendering the stale values first.
  const [syncedHeader, setSyncedHeader] = useState({
    name: selectedName,
    description: selected?.description,
  });
  if (syncedHeader.name !== selectedName || syncedHeader.description !== selected?.description) {
    setSyncedHeader({ name: selectedName, description: selected?.description });
    setDescription(selected?.description ?? "");
    setName(selected?.name ?? "");
    setEditingName(false);
    setNewFilePath(null);
  }

  // Reset drafts whenever the persisted values change (context switch or save).
  const [syncedVars, setSyncedVars] = useState(vars);
  if (syncedVars !== vars) {
    setSyncedVars(vars);
    setDrafts(draftsFrom(vars));
  }

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

  /** Asks for the file of a stored variable and replaces its content. */
  const replaceFile = async (key: string) => {
    const path = await open({ multiple: false });
    if (typeof path === "string") await setFile(key, path);
  };

  const handleChooseNewFile = async () => {
    const path = await open({ multiple: false });
    if (typeof path === "string") setNewFilePath(path);
  };

  const handleAddFile = async () => {
    const key = newKey.trim();
    if (!key || !newFilePath) return;
    await setFile(key, newFilePath);
    setNewKey("");
    setNewFilePath(null);
  };

  const handleAdd = () => {
    if (addKind === "file") void handleAddFile();
    else void handleAddVar();
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
              title="Export these variables as KEY=value. File variables export their path, not their content"
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
          Use <code>{"{$VAR}"}</code> to reuse another variable of this context &mdash; for
          example <code>{"{$APP_PATH}/config"}</code>. References resolve when the value reaches
          your shell, so editing the source updates everything derived from it.
        </p>

        <div className="cred-vars">
          {vars.map((v) => {
            if (v.file) {
              return (
                <div className="cred-var-item" key={v.key}>
                  <div className="cred-var-row">
                    <span className="cred-var-key">{v.key}</span>
                    <div className="cred-var-file" title={v.file.path}>
                      <FileIcon />
                      <span className="cred-var-file-name">{v.file.filename}</span>
                      <span className="cred-var-file-size">{formatBytes(v.file.size)}</span>
                    </div>
                    <button
                      className="cred-var-btn"
                      title="Replace the stored file"
                      onClick={() => void replaceFile(v.key)}
                    >
                      replace
                    </button>
                    <button
                      className="cred-var-btn danger"
                      title="Delete variable and its stored file"
                      onClick={() => deleteVar(v.key)}
                    >
                      ×
                    </button>
                  </div>
                </div>
              );
            }
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
          <div className="cred-add-kind" role="group" aria-label="Variable type">
            <button
              className={addKind === "text" ? "active" : ""}
              aria-pressed={addKind === "text"}
              onClick={() => setAddKind("text")}
            >
              Text
            </button>
            <button
              className={addKind === "file" ? "active" : ""}
              aria-pressed={addKind === "file"}
              onClick={() => setAddKind("file")}
            >
              File
            </button>
          </div>
          <input
            className="cred-add-key"
            value={newKey}
            placeholder="KEY"
            onChange={(e) => setNewKey(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter") handleAdd();
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
          {addKind === "text" ? (
            <input
              className="cred-add-value"
              value={newValue}
              placeholder="value"
              onChange={(e) => setNewValue(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") handleAdd();
              }}
            />
          ) : (
            <button
              className={`cred-add-file${newFilePath ? " chosen" : ""}`}
              title={newFilePath ?? undefined}
              onClick={() => void handleChooseNewFile()}
            >
              <FileIcon />
              <span className="cred-add-file-label">
                {newFilePath ? baseName(newFilePath) : "Choose a file…"}
              </span>
            </button>
          )}
          <button
            className="cred-add-btn"
            onClick={handleAdd}
            disabled={addKind === "file" && (!newKey.trim() || !newFilePath)}
          >
            Add
          </button>
          {addKind === "text" && (
            <button
              className="cred-paste-btn"
              onClick={handlePasteButton}
              title="Paste KEY=value lines from the clipboard into this context"
            >
              Paste
            </button>
          )}
        </div>
        {addKind === "file" && (
          <p className="cred-hint">
            Bypass keeps an encrypted copy, up to 1 MiB, and the variable holds its path. The file
            is on disk only while this context is active and Bypass is running.
          </p>
        )}
        {pasteNotice && <div className="cred-paste-notice">{pasteNotice}</div>}
      </div>
    </div>
  );
}
