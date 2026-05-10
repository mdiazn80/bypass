import { useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import TopBar, { type View } from "./components/TopBar";
import Sidebar from "./components/Sidebar";
import ContextEditor from "./components/ContextEditor";
import Settings from "./components/Settings";
import AboutModal from "./components/AboutModal";
import Footer from "./components/Footer";
import UpdateBanner, { type UpdateState } from "./components/UpdateBanner";
import { useContextStore } from "./stores/useContextStore";
import { useConfigStore } from "./stores/useConfigStore";
import "./App.css";

function App() {
  const [view, setView] = useState<View>("contexts");
  const [showAbout, setShowAbout] = useState(false);
  const [updateState, setUpdateState] = useState<UpdateState | null>(null);
  const pendingUpdate = useRef<Update | null>(null);
  const loadContexts = useContextStore((s) => s.load);
  const loadConfig = useConfigStore((s) => s.load);

  async function runUpdateCheck(manual: boolean) {
    if (manual) {
      setUpdateState({ phase: "checking", version: "", downloaded: 0, total: null });
    }
    try {
      const update = await check();
      if (update?.available) {
        pendingUpdate.current = update;
        setUpdateState({ phase: "available", version: update.version, downloaded: 0, total: null });
      } else if (manual) {
        setUpdateState({ phase: "up-to-date", version: "", downloaded: 0, total: null });
        setTimeout(() => setUpdateState(null), 4000);
      }
    } catch {
      if (manual) setUpdateState(null);
    }
  }

  useEffect(() => {
    loadContexts();
    loadConfig();
    runUpdateCheck(false);

    const unlistenContexts = listen("contexts-changed", () => loadContexts());
    const unlistenAbout = listen("show-about", () => setShowAbout(true));
    const unlistenCheckUpdates = listen("check-for-updates", () => runUpdateCheck(true));

    return () => {
      unlistenContexts.then((fn) => fn());
      unlistenAbout.then((fn) => fn());
      unlistenCheckUpdates.then((fn) => fn());
    };
  }, []);

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
        {view === "contexts" && (
          <>
            <Sidebar />
            <ContextEditor />
          </>
        )}
        {view === "settings" && <Settings />}
      </div>
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
