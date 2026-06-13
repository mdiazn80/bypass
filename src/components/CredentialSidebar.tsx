import { useEffect, useState } from "react";
import { useCredentialStore } from "../stores/useCredentialStore";
import { useConfigStore } from "../stores/useConfigStore";
import "./Sidebar.css";

export default function CredentialSidebar() {
  const { contexts, selectedName, selectContext, createContext, deleteContext } =
    useCredentialStore();
  const activeContext = useConfigStore((s) => s.shellStatus?.active_context ?? null);
  const setActiveContext = useConfigStore((s) => s.setActiveContext);
  const loadShellStatus = useConfigStore((s) => s.loadShellStatus);
  const [newName, setNewName] = useState("");
  const [showInput, setShowInput] = useState(false);

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

  // Only one context can be active. Toggling the active one off clears it.
  const handleToggleActive = (name: string) => {
    setActiveContext(activeContext === name ? null : name);
  };

  const handleDelete = async (name: string) => {
    if (activeContext === name) {
      await setActiveContext(null);
    }
    await deleteContext(name);
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
        {contexts.map((ctx) => (
          <div
            key={ctx.name}
            className={`sidebar-item ${selectedName === ctx.name ? "selected" : ""}`}
            onClick={() => selectContext(ctx.name)}
          >
            <span className="sidebar-item-name">{ctx.name}</span>
            {activeContext === ctx.name && (
              <span className="sidebar-item-badge" title="Active in shells">
                active
              </span>
            )}
            <button
              className="sidebar-delete"
              onClick={(e) => {
                e.stopPropagation();
                handleDelete(ctx.name);
              }}
              title="Delete context"
            >
              ×
            </button>
            <label
              className="sidebar-toggle"
              onClick={(e) => e.stopPropagation()}
              title={
                activeContext === ctx.name
                  ? "Active context — variables served to your shells"
                  : "Activate this context for your shells"
              }
            >
              <input
                type="checkbox"
                checked={activeContext === ctx.name}
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
    </div>
  );
}
