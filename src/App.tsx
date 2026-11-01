import { useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import TopBar, { type View } from "./components/TopBar";
import Sidebar from "./components/Sidebar";
import Dashboard from "./components/Dashboard";
import ContextEditor from "./components/ContextEditor";
import CredentialSidebar from "./components/CredentialSidebar";
import CredentialEditor from "./components/CredentialEditor";
import Settings from "./components/Settings";
import AboutModal from "./components/AboutModal";
import Footer from "./components/Footer";
import UpdateBanner, { type UpdateState } from "./components/UpdateBanner";
import { useContextStore } from "./stores/useContextStore";
import { useConfigStore } from "./stores/useConfigStore";
import { useCredentialStore } from "./stores/useCredentialStore";
import "./App.css";

/** A newer version if there is one, `null` if up to date, `undefined` if the check failed. */
async function findUpdate(): Promise<Update | null | undefined> {
  try {
    const update = await check();
    return update?.available ? update : null;
  } catch {
    return undefined;
  }
}

function App() {
  const [view, setView] = useState<View>("home");
  const [showAbout, setShowAbout] = useState(false);
  const [updateState, setUpdateState] = useState<UpdateState | null>(null);
  const pendingUpdate = useRef<Update | null>(null);
  const loadContexts = useContextStore((s) => s.load);
  const contextError = useContextStore((s) => s.error);
  const clearContextError = useContextStore((s) => s.clearError);
  const hostsInSync = useContextStore((s) => s.hostsInSync);
  const reapplying = useContextStore((s) => s.reapplying);
  const reapplyHosts = useContextStore((s) => s.reapply);
  const hostsChanged = useContextStore((s) => s.hostsChanged);
  const loadConfig = useConfigStore((s) => s.load);
  const loadCredentials = useCredentialStore((s) => s.load);
  const credentialError = useCredentialStore((s) => s.error);
  const clearCredentialError = useCredentialStore((s) => s.clearError);

  /** A manual check reports every outcome; the startup one only a new version. */
  function showUpdateResult(update: Update | null | undefined, manual: boolean) {
    if (update) {
      pendingUpdate.current = update;
      setUpdateState({ phase: "available", version: update.version, downloaded: 0, total: null });
    } else if (manual && update === null) {
      setUpdateState({ phase: "up-to-date", version: "", downloaded: 0, total: null });
      setTimeout(() => setUpdateState(null), 4000);
    } else if (manual) {
      setUpdateState(null);
    }
  }

  useEffect(() => {
    loadContexts();
    loadConfig();
    loadCredentials();
    findUpdate().then((update) => showUpdateResult(update, false));

    const unlistenAbout = listen("show-about", () => setShowAbout(true));
    const unlistenHosts = listen<{ in_sync: boolean }>("hosts-changed", (event) =>
      hostsChanged(event.payload.in_sync)
    );
    const unlistenCheckUpdates = listen("check-for-updates", () => {
      setUpdateState({ phase: "checking", version: "", downloaded: 0, total: null });
      findUpdate().then((update) => showUpdateResult(update, true));
    });

    return () => {
      unlistenAbout.then((fn) => fn());
      unlistenCheckUpdates.then((fn) => fn());
      unlistenHosts.then((fn) => fn());
    };
    // The store actions are stable, so this still runs once on mount.
  }, [loadContexts, loadConfig, loadCredentials, hostsChanged]);

  async function handleInstallUpdate() {
    const update = pendingUpdate.current;
    if (!update) return;

    let totalBytes: number | null = null;
    let downloadedBytes = 0;

    setUpdateState({
      phase: "downloading",
      version: update.version,
      downloaded: 0,
      total: null,
    });

    try {
      await update.downloadAndInstall((event) => {
        if (event.event === "Started") {
          totalBytes = event.data.contentLength ?? null;
          setUpdateState({
            phase: "downloading",
            version: update.version,
            downloaded: 0,
            total: totalBytes,
          });
        } else if (event.event === "Progress") {
          downloadedBytes += event.data.chunkLength;
          setUpdateState({
            phase: "downloading",
            version: update.version,
            downloaded: downloadedBytes,
            total: totalBytes,
          });
        } else if (event.event === "Finished") {
          setUpdateState({
            phase: "done",
            version: update.version,
            downloaded: downloadedBytes,
            total: totalBytes,
          });
        }
      });
      await relaunch();
    } catch {
      setUpdateState(null);
    }
  }

  return (
    <div className="app">
      <TopBar view={view} onViewChange={setView} onAbout={() => setShowAbout(true)} />
      <div className="app-body">
        {view === "home" && <Dashboard />}
        {view === "contexts" && (
          <>
            <Sidebar />
            <ContextEditor />
          </>
        )}
        {view === "credentials" && (
          <>
            <CredentialSidebar />
            <CredentialEditor />
          </>
        )}
        {view === "settings" && <Settings />}
      </div>
      {!hostsInSync && (
        <div className="error-banner warning" role="alert">
          <span className="error-banner-message">
            Another program changed /etc/hosts and the entries of your active hosts contexts are
            missing or out of date.
          </span>
          <button className="error-banner-action" onClick={reapplyHosts} disabled={reapplying}>
            {reapplying ? "Re-applying…" : "Re-apply"}
          </button>
        </div>
      )}
      {contextError && (
        <div className="error-banner" role="alert">
          <span className="error-banner-message">{contextError}</span>
          <button className="error-banner-dismiss" onClick={clearContextError}>✕</button>
        </div>
      )}
      {credentialError && (
        <div className="error-banner" role="alert">
          <span className="error-banner-message">{credentialError}</span>
          <button className="error-banner-dismiss" onClick={clearCredentialError}>✕</button>
        </div>
      )}
      {updateState && (
        <UpdateBanner
          state={updateState}
          onInstall={handleInstallUpdate}
          onDismiss={() => setUpdateState(null)}
        />
      )}
      <Footer />
      {showAbout && <AboutModal onClose={() => setShowAbout(false)} />}
    </div>
  );
}

export default App;
