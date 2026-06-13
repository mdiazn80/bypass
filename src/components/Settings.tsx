import { useEffect } from "react";
import { useConfigStore } from "../stores/useConfigStore";
import "./Settings.css";

export default function Settings() {
  const { config, autostart, update, setAutostart } = useConfigStore();
  const {
    shellStatus,
    loadShellStatus,
    setShellAgentEnabled,
    installShell,
    uninstallShell,
  } = useConfigStore();

  useEffect(() => {
    loadShellStatus();
  }, [loadShellStatus]);

  const agentEnabled = shellStatus?.enabled ?? false;
  const installed = shellStatus?.installed ?? false;

  return (
    <div className="settings">
      <div className="settings-header">
        <h2>Settings</h2>
      </div>
      <div className="settings-body">
        <div className="settings-group">
          <h3>General</h3>

          <label className="settings-row">
            <span className="settings-label">
              <strong>Launch at startup</strong>
              <small>Automatically start Bypass when you log in</small>
            </span>
            <span className="settings-toggle">
              <input
                type="checkbox"
                checked={autostart}
                onChange={(e) => setAutostart(e.target.checked)}
              />
              <span className="settings-toggle-slider" />
            </span>
          </label>

          <label className="settings-row">
            <span className="settings-label">
              <strong>Minimize to tray</strong>
              <small>Keep running in the system tray when window is closed</small>
            </span>
            <span className="settings-toggle">
              <input
                type="checkbox"
                checked={config.minimize_to_tray}
                onChange={(e) => update(e.target.checked, undefined)}
              />
              <span className="settings-toggle-slider" />
            </span>
          </label>

          <label className="settings-row">
            <span className="settings-label">
              <strong>Start minimized</strong>
              <small>Start the application minimized to the system tray</small>
            </span>
            <span className="settings-toggle">
              <input
                type="checkbox"
                checked={config.start_minimized}
                onChange={(e) => update(undefined, e.target.checked)}
              />
              <span className="settings-toggle-slider" />
            </span>
          </label>
        </div>

        <div className="settings-group">
          <h3>Shell Integration</h3>

          <label className="settings-row">
            <span className="settings-label">
              <strong>Enable shell agent</strong>
              <small>
                Serve the active context's variables to your terminals. Keeps
                Bypass running in the background and enables launch at startup.
              </small>
            </span>
            <span className="settings-toggle">
              <input
                type="checkbox"
                checked={agentEnabled}
                onChange={(e) => setShellAgentEnabled(e.target.checked)}
              />
              <span className="settings-toggle-slider" />
            </span>
          </label>

          <div className="settings-row">
            <span className="settings-label">
              <strong>Shell hook</strong>
              <small>
                {shellStatus?.detected_shell
                  ? `Detected ${shellStatus.detected_shell}` +
                    (shellStatus.rc_path ? ` (${shellStatus.rc_path})` : "")
                  : "No supported shell detected"}
              </small>
            </span>
            <span className="settings-actions">
              {installed ? (
                <button
                  className="settings-button"
                  onClick={() => uninstallShell()}
                >
                  Uninstall
                </button>
              ) : (
                <button
                  className="settings-button"
                  disabled={!shellStatus?.detected_shell}
                  onClick={() => installShell()}
                >
                  Install
                </button>
              )}
            </span>
          </div>

          <div className="settings-status">
            <span
              className={
                shellStatus?.socket_active
                  ? "settings-status-dot active"
                  : "settings-status-dot"
              }
            />
            {shellStatus?.socket_active
              ? "Agent running"
              : "Agent stopped"}
            {installed ? " · hook installed" : " · hook not installed"}
          </div>
        </div>
      </div>
    </div>
  );
}
