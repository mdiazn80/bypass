import { useConfigStore } from "../stores/useConfigStore";
import "./Settings.css";

export default function Settings() {
  const { config, autostart, update, setAutostart } = useConfigStore();

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
      </div>
    </div>
  );
}
