import { useEffect, useState } from "react";
import { useCredentialStore, ACTIVE_VARS_ID } from "../stores/useCredentialStore";
import MoveButtons from "./MoveButtons";
import ConfirmModal from "./ConfirmModal";
import { useConfigStore } from "../stores/useConfigStore";
import "./Sidebar.css";

export default function CredentialSidebar() {
  const {
    contexts,
    selectedName,
    activeVars,
    selectContext,
    createContext,
    deleteContext,
    moveContext,
    refreshActiveVars,
  } = useCredentialStore();
  const activeContexts = useConfigStore((s) => s.shellStatus?.active_contexts ?? []);
  const setContextActive = useConfigStore((s) => s.setContextActive);
  const loadShellStatus = useConfigStore((s) => s.loadShellStatus);
  const [newName, setNewName] = useState("");
  const [showInput, setShowInput] = useState(false);
  // Context awaiting delete confirmation.
  const [pendingDelete, setPendingDelete] = useState<string | null>(null);

  useEffect(() => {
    loadShellStatus();
  }, [loadShellStatus]);

  const handleCreate = async () => {
    const name = newName.trim();
    if (!name) return;
    await createContext(name, "");
    setNewName("");
    setShowInput(false);
  };

  const isActive = (name: string) => activeContexts.includes(name);

  // Several contexts can be active at once; the merged view must follow.
  const handleToggleActive = async (name: string) => {
    await setContextActive(name, !isActive(name));
    await refreshActiveVars();
  };

  const handleDelete = async (name: string) => {
    // The backend drops the context from the active list; refresh our copy.
    await deleteContext(name);
    await loadShellStatus();
  };

  return (
    <div className="sidebar">
      <div className="sidebar-header">
        <span className="sidebar-label">Credentials</span>
        <button
          className="sidebar-add"
          onClick={() => setShowInput((s) => !s)}
          title="New credential context"
        >
          +
        </button>
      </div>

      {showInput && (
        <div className="sidebar-new">
          <input
            autoFocus
            value={newName}
            onChange={(e) => setNewName(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter") handleCreate();
              if (e.key === "Escape") {
                setShowInput(false);
                setNewName("");
              }
            }}
            onBlur={() => {
              if (!newName.trim()) {
                setShowInput(false);
                setNewName("");
              }
            }}
            placeholder="Context name..."
          />
          <button onClick={handleCreate}>Add</button>
        </div>
      )}

      <div className="sidebar-list">
        <div
          className={`sidebar-item sidebar-item-system ${selectedName === ACTIVE_VARS_ID ? "selected" : ""}`}
          onClick={() => selectContext(ACTIVE_VARS_ID)}
          title="Every variable your shells receive, merged from the active contexts"
        >
          <span className="sidebar-item-icon">⌘</span>
          <span className="sidebar-item-name">Active Variables</span>
          {activeVars.length > 0 && (
            <span className="sidebar-item-count">{activeVars.length}</span>
          )}
        </div>

        {contexts.map((ctx, i) => (
          <div
            key={ctx.name}
            className={`sidebar-item ${selectedName === ctx.name ? "selected" : ""}`}
            onClick={() => selectContext(ctx.name)}
          >
            <span className="sidebar-item-name">{ctx.name}</span>
            {isActive(ctx.name) && (
              <span className="sidebar-item-badge" title="Active in shells">
                active
              </span>
            )}
            <MoveButtons
              canUp={i > 0}
              canDown={i < contexts.length - 1}
              onMove={(delta) => moveContext(ctx.name, delta)}
              hint="Higher contexts win when two define the same variable"
            />
            <button
              className="sidebar-delete"
              onClick={(e) => {
                e.stopPropagation();
                setPendingDelete(ctx.name);
              }}
              title="Delete context"
            >
              ×
            </button>
            <label
              className="sidebar-toggle"
              onClick={(e) => e.stopPropagation()}
              title={
                isActive(ctx.name)
                  ? "Active — its variables are served to your shells"
                  : "Activate this context for your shells"
              }
            >
              <input
                type="checkbox"
                checked={isActive(ctx.name)}
                onChange={() => handleToggleActive(ctx.name)}
              />
              <span className="toggle-slider" />
            </label>
          </div>
        ))}

        {contexts.length === 0 && (
          <div className="sidebar-empty">No credential contexts yet</div>
        )}
      </div>

      {pendingDelete && (
        <ConfirmModal
          title="Delete credential context?"
          message={
            <>
              <strong>{pendingDelete}</strong> and all of its variables will be
              permanently deleted from the vault. This cannot be undone.
              {isActive(pendingDelete) && (
                <> It is active, so your shells will stop receiving its variables.</>
              )}
            </>
          }
          onConfirm={() => {
            void handleDelete(pendingDelete);
            setPendingDelete(null);
          }}
          onCancel={() => setPendingDelete(null)}
        />
      )}
    </div>
  );
}
