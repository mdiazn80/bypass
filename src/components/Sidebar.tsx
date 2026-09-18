import { useState } from "react";
import { useContextStore, SYSTEM_HOSTS_ID } from "../stores/useContextStore";
import MoveButtons from "./MoveButtons";
import ConfirmModal from "./ConfirmModal";
import "./Sidebar.css";

export default function Sidebar() {
  const { contexts, selectedId, togglingId, reordering, select, create, remove, toggle, move } =
    useContextStore();
  // Toggling and reordering both rewrite the hosts file; one at a time.
  const busy = togglingId !== null || reordering;
  const [newName, setNewName] = useState("");
  const [showInput, setShowInput] = useState(false);
  // Context awaiting delete confirmation.
  const [pendingDelete, setPendingDelete] = useState<{ id: string; name: string; enabled: boolean } | null>(null);

  const handleCreate = async () => {
    const name = newName.trim();
    if (!name) return;
    await create(name);
    setNewName("");
    setShowInput(false);
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

        {contexts.map((ctx, i) => (
          <div
            key={ctx.id}
            className={`sidebar-item ${selectedId === ctx.id ? "selected" : ""}`}
            onClick={() => select(ctx.id)}
          >
            <span className="sidebar-item-name">{ctx.name}</span>
            <MoveButtons
              canUp={i > 0}
              canDown={i < contexts.length - 1}
              disabled={busy}
              onMove={(delta) => move(ctx.id, delta)}
              hint="Entries of higher contexts come first in the hosts file"
            />
            <button
              className="sidebar-delete"
              disabled={busy}
              onClick={(e) => {
                e.stopPropagation();
                setPendingDelete({ id: ctx.id, name: ctx.name, enabled: ctx.enabled });
              }}
              title="Delete context"
            >
              ×
            </button>
            {togglingId === ctx.id ? (
              <span
                className="sidebar-toggle-spinner"
                title="Waiting for administrator authorization…"
                aria-label="Applying"
              />
            ) : (
              <label className="sidebar-toggle" onClick={(e) => e.stopPropagation()}>
                <input
                  type="checkbox"
                  checked={ctx.enabled}
                  // One hosts write at a time: a second toggle would queue up
                  // another administrator prompt behind the first.
                  disabled={busy}
                  onChange={() => toggle(ctx.id)}
                />
                <span className="toggle-slider" />
              </label>
            )}
          </div>
        ))}

        {contexts.length === 0 && (
          <div className="sidebar-empty">No contexts yet</div>
        )}
      </div>

      {pendingDelete && (
        <ConfirmModal
          title="Delete context?"
          message={
            <>
              <strong>{pendingDelete.name}</strong> and its hosts entries will be
              deleted. This cannot be undone.
              {pendingDelete.enabled && (
                <> It is active, so its entries will also be removed from the
                system hosts file (administrator credentials required).</>
              )}
            </>
          }
          onConfirm={() => {
            void remove(pendingDelete.id);
            setPendingDelete(null);
          }}
          onCancel={() => setPendingDelete(null)}
        />
      )}
    </div>
  );
}
