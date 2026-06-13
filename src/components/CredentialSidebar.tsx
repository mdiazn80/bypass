import { useState } from "react";
import { useCredentialStore } from "../stores/useCredentialStore";
import "./Sidebar.css";

export default function CredentialSidebar() {
  const {
    contexts,
    selectedName,
    activeName,
    selectContext,
    createContext,
    deleteContext,
    setActive,
  } = useCredentialStore();
  const [newName, setNewName] = useState("");
  const [showInput, setShowInput] = useState(false);

  const handleCreate = async () => {
    const name = newName.trim();
    if (!name) return;
    await createContext(name, "");
    setNewName("");
    setShowInput(false);
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
            placeholder="Context name..."
          />
          <button onClick={handleCreate}>Add</button>
        </div>
      )}

      <div className="sidebar-list">
        {contexts.map((ctx) => {
          const isActive = ctx.name === activeName;
          return (
            <div
              key={ctx.name}
              className={`sidebar-item ${selectedName === ctx.name ? "selected" : ""}`}
              onClick={() => selectContext(ctx.name)}
            >
              <span className="sidebar-item-name">{ctx.name}</span>
              <button
                className="sidebar-delete"
                onClick={(e) => {
                  e.stopPropagation();
                  deleteContext(ctx.name);
                }}
                title="Delete context"
              >
                ×
              </button>
              <label
                className="sidebar-toggle"
                onClick={(e) => e.stopPropagation()}
                title={isActive ? "Active context (click to deactivate)" : "Set as active context"}
              >
                <input
                  type="checkbox"
                  checked={isActive}
                  onChange={() => setActive(isActive ? null : ctx.name)}
                />
                <span className="toggle-slider" />
              </label>
            </div>
          );
        })}

        {contexts.length === 0 && (
          <div className="sidebar-empty">No credential contexts yet</div>
        )}
      </div>
    </div>
  );
}
