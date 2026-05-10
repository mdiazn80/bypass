import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";

document.addEventListener("contextmenu", (e) => e.preventDefault(), { capture: true });

document.addEventListener(
  "keydown",
  (e) => {
    if (e.key === "F12") { e.preventDefault(); return; }
    if ((e.ctrlKey || e.metaKey) && e.shiftKey && (e.key === "I" || e.key === "i")) { e.preventDefault(); return; }
    if ((e.ctrlKey || e.metaKey) && e.shiftKey && (e.key === "J" || e.key === "j")) { e.preventDefault(); return; }
    if ((e.ctrlKey || e.metaKey) && e.shiftKey && (e.key === "C" || e.key === "c")) { e.preventDefault(); return; }
    if ((e.ctrlKey || e.metaKey) && (e.key === "u" || e.key === "U")) { e.preventDefault(); return; }
  },
  { capture: true },
);

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
