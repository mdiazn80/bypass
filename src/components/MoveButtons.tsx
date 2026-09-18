import "./Sidebar.css";

interface MoveButtonsProps {
  canUp: boolean;
  canDown: boolean;
  disabled?: boolean;
  /** Extra tooltip line explaining what the order means in this list. */
  hint?: string;
  onMove: (delta: -1 | 1) => void;
}

function Chevron({ up }: { up: boolean }) {
  return (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round">
      {up ? <polyline points="6 15 12 9 18 15" /> : <polyline points="6 9 12 15 18 9" />}
    </svg>
  );
}

/**
 * Up/down chevrons that reorder a sidebar item. Clicks do not bubble, so the
 * row's own click (select) is not triggered.
 */
export default function MoveButtons({ canUp, canDown, disabled, hint, onMove }: MoveButtonsProps) {
  const title = (dir: string) => (hint ? `${dir}\n${hint}` : dir);
  return (
    <span className="sidebar-move" onClick={(e) => e.stopPropagation()}>
      <button
        className="sidebar-move-btn"
        disabled={disabled || !canUp}
        onClick={() => onMove(-1)}
        title={title("Move up (higher priority)")}
        aria-label="Move up"
      >
        <Chevron up />
      </button>
      <button
        className="sidebar-move-btn"
        disabled={disabled || !canDown}
        onClick={() => onMove(1)}
        title={title("Move down (lower priority)")}
        aria-label="Move down"
      >
        <Chevron up={false} />
      </button>
    </span>
  );
}
