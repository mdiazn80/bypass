import logo from "../assets/logo.png";
import "./TopBar.css";

export type View = "contexts" | "credentials" | "settings";

interface TopBarProps {
  view: View;
  onViewChange: (view: View) => void;
  onAbout: () => void;
}

export default function TopBar({ view, onViewChange, onAbout }: TopBarProps) {
  return (
    <div className="topbar">
      <div className="topbar-title">
        <img src={logo} alt="Bypass" className="topbar-logo" />
        Bypass
      </div>
      <nav className="topbar-nav">
        <button
          className={view === "contexts" ? "active" : ""}
          onClick={() => onViewChange("contexts")}
        >
          Hosts
        </button>
        <button
          className={view === "credentials" ? "active" : ""}
          onClick={() => onViewChange("credentials")}
        >
          Credentials
        </button>
        <button
          className={view === "settings" ? "active" : ""}
          onClick={() => onViewChange("settings")}
        >
          Settings
        </button>
      </nav>
      <div className="topbar-spacer" />
      <button className="topbar-about" onClick={onAbout}>
        About
      </button>
    </div>
  );
}
