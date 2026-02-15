import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { check } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import TopBar, { type View } from "./components/TopBar";
import Sidebar from "./components/Sidebar";
import ContextEditor from "./components/ContextEditor";
import Settings from "./components/Settings";
import AboutModal from "./components/AboutModal";
import Footer from "./components/Footer";
import { useContextStore } from "./stores/useContextStore";
import { useConfigStore } from "./stores/useConfigStore";
import "./App.css";

async function checkForUpdates() {
  try {
    const update = await check();
    if (update?.available) {
      const confirmed = window.confirm(
        `A new version (${update.version}) is available. Update now?`
      );
      if (confirmed) {
        await update.downloadAndInstall();
        await relaunch();
      }
    }
  } catch {
    // silently ignore update check failures
  }
}

function App() {
  const [view, setView] = useState<View>("contexts");
  const [showAbout, setShowAbout] = useState(false);
  const loadContexts = useContextStore((s) => s.load);
  const loadConfig = useConfigStore((s) => s.load);

  useEffect(() => {
    loadContexts();
    loadConfig();
    checkForUpdates();

    const unlistenContexts = listen("contexts-changed", () => {
      loadContexts();
    });

    const unlistenAbout = listen("show-about", () => {
      setShowAbout(true);
    });

    return () => {
      unlistenContexts.then((fn) => fn());
      unlistenAbout.then((fn) => fn());
    };
  }, []);

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
      <Footer />
      {showAbout && <AboutModal onClose={() => setShowAbout(false)} />}
    </div>
  );
}

export default App;
