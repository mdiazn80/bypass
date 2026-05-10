import "./UpdateBanner.css";

export type UpdatePhase = "available" | "downloading" | "done";

export interface UpdateState {
  phase: UpdatePhase;
  version: string;
  downloaded: number;
  total: number | null;
}

interface UpdateBannerProps {
  state: UpdateState;
  onInstall: () => void;
  onDismiss: () => void;
}

export default function UpdateBanner({ state, onInstall, onDismiss }: UpdateBannerProps) {
  const progress =
    state.total != null ? Math.round((state.downloaded / state.total) * 100) : null;

  if (state.phase === "done") {
    return (
      <div className="update-banner update-banner--done">
        <span className="update-banner-message">Update installed. Restarting…</span>
      </div>
    );
  }

  if (state.phase === "downloading") {
    return (
      <div className="update-banner update-banner--downloading">
        <span className="update-banner-message">
          Downloading {state.version}…
        </span>
        <div className="update-banner-progress">
          <div
            className="update-banner-progress-bar"
            style={{ width: progress != null ? `${progress}%` : "0%" }}
          />
        </div>
        {progress != null && (
          <span className="update-banner-percent">{progress}%</span>
        )}
      </div>
    );
  }

  return (
    <div className="update-banner">
      <span className="update-banner-message">
        New version <strong>{state.version}</strong> available
      </span>
      <div className="update-banner-actions">
        <button
          className="update-banner-btn update-banner-btn--primary"
          onClick={onInstall}
        >
          Update &amp; Restart
        </button>
        <button
          className="update-banner-btn update-banner-btn--dismiss"
          onClick={onDismiss}
          title="Dismiss"
        >
          ✕
        </button>
      </div>
    </div>
  );
}
