import { useCallback, useEffect, useState } from "react";
import { useCredentialStore } from "../stores/useCredentialStore";
import FileDropZone from "./FileDropZone";
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
        </div>
      </div>
    </div>
  );
}
