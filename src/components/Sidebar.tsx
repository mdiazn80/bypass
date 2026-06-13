import { useState } from "react";
import { save } from "@tauri-apps/plugin-dialog";
import { writeTextFile } from "@tauri-apps/plugin-fs";
import { useContextStore, SYSTEM_HOSTS_ID } from "../stores/useContextStore";
import "./Sidebar.css";

export default function Sidebar() {
  const { contexts, selectedId, select, create, remove, toggle } =
    useContextStore();
  const [newName, setNewName] = useState("");
  const [showInput, setShowInput] = useState(false);

  const handleCreate = async () => {
    const name = newName.trim();
    if (!name) return;
    await create(name);
    setNewName("");
    setShowInput(false);
  };

  const handleExport = async (ctx: { id: string; name: string; content: string }) => {
    try {
      const filePath = await save({
        defaultPath: `${ctx.name.replace(/[^a-zA-Z0-9_-]/g, "_")}.json`,
        filters: [{ name: "JSON", extensions: ["json"] }],
      });
      if (!filePath) return;

      const data = { name: ctx.name, content: ctx.content };
      await writeTextFile(filePath, JSON.stringify(data, null, 2));
    } catch {
      // user cancelled
    }
  };

  return (
    <div className="sidebar">
      <div className="sidebar-header">
        <span className="sidebar-label">Contexts</span>
        <button
          className="sidebar-add"
          onClick={() => setShowInput(true)}
          title="New context"
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
              if (e.key === "Escape") setShowInput(false);
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
          className={`sidebar-item sidebar-item-system ${selectedId === SYSTEM_HOSTS_ID ? "selected" : ""}`}
          onClick={() => select(SYSTEM_HOSTS_ID)}
        >
          <span className="sidebar-item-icon">⌘</span>
          <span className="sidebar-item-name">System Hosts</span>
        </div>

        {contexts.map((ctx) => (
          <div
            key={ctx.id}
            className={`sidebar-item ${selectedId === ctx.id ? "selected" : ""}`}
            onClick={() => select(ctx.id)}
          >
            <span className="sidebar-item-name">{ctx.name}</span>
            <button
              className="sidebar-export"
              onClick={(e) => {
                e.stopPropagation();
                handleExport(ctx);
              }}
              title="Export context"
            >
              ↓
            </button>
            <button
              className="sidebar-delete"
              onClick={(e) => {
                e.stopPropagation();
                remove(ctx.id);
              }}
              title="Delete context"
            >
              ×
            </button>
            <label className="sidebar-toggle" onClick={(e) => e.stopPropagation()}>
              <input
                type="checkbox"
                checked={ctx.enabled}
                onChange={() => toggle(ctx.id)}
              />
              <span className="toggle-slider" />
            </label>
          </div>
        ))}

        {contexts.length === 0 && (
          <div className="sidebar-empty">No contexts yet</div>
        )}
      </div>
    </div>
  );
}
