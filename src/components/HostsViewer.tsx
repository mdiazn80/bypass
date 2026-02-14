import { useEffect, useState } from "react";
import { getSystemHosts } from "../services/tauri";
import { highlightHosts } from "./ContextEditor";
import "./HostsViewer.css";

export default function HostsViewer() {
  const [content, setContent] = useState("");
  const [error, setError] = useState("");

  const load = async () => {
    try {
      const hosts = await getSystemHosts();
      setContent(hosts);
      setError("");
    } catch (e) {
      setError(String(e));
    }
  };

  useEffect(() => {
    load();
  }, []);

  return (
    <div className="hosts-viewer">
      <div className="hosts-viewer-header">
        <h2>System Hosts File</h2>
        <button className="hosts-refresh" onClick={load}>
          Refresh
        </button>
      </div>
      {error ? (
        <div className="hosts-error">{error}</div>
      ) : (
        <pre className="hosts-content">{highlightHosts(content)}</pre>
      )}
    </div>
  );
}
