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
  const { contexts, selectedName, vars, updateContext, reveal, hide, setVar, deleteVar } =
    useCredentialStore();

  const selected = contexts.find((c) => c.name === selectedName);
  const [description, setDescription] = useState("");
  const [newKey, setNewKey] = useState("");
  const [newValue, setNewValue] = useState("");

  useEffect(() => {
    setDescription(selected?.description ?? "");
  }, [selectedName, selected?.description]);

  const handleImportEnv = useCallback(
    (content: string) => {
      for (const { key, value } of parseEnv(content)) {
        void setVar(key, value);
      }
    },
    [setVar]
  );

  if (!selectedName || !selected) {
    return (
      <div className="editor-empty">
        <p>Select a credential context from the sidebar, or create one to store environment variables securely.</p>
      </div>
    );
  }

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

  return (
    <div className="editor">
      <div className="editor-header">
        <h2 className="editor-name">{selected.name}</h2>
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

        <div className="cred-vars">
          {vars.map((v) => (
            <div className="cred-var-row" key={v.key}>
              <span className="cred-var-key">{v.key}</span>
              <span className="cred-var-value">
                {v.value === null ? "••••••••" : v.value}
              </span>
              <button
                className="cred-var-btn"
                title={v.value === null ? "Reveal" : "Hide"}
                onClick={() => (v.value === null ? reveal(v.key) : hide(v.key))}
              >
                {v.value === null ? "show" : "hide"}
              </button>
              <button
                className="cred-var-btn danger"
                title="Delete variable"
                onClick={() => deleteVar(v.key)}
              >
                ×
              </button>
            </div>
          ))}

          {vars.length === 0 && (
            <FileDropZone
              title="Drag & drop a .env file"
              hint="or click to browse — keys without a value can be filled in below"
              onImport={handleImportEnv}
            />
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
