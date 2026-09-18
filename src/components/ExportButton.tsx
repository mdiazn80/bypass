import { useState } from "react";
import { save } from "@tauri-apps/plugin-dialog";
import { writeTextFile } from "@tauri-apps/plugin-fs";
import "./ContextEditor.css";

interface ExportButtonProps {
  /** Suggested file name, extension included. */
  defaultName: string;
  /** Label of the file-type filter in the save dialog. */
  filterName: string;
  extensions: string[];
  /** Produces the file contents; called once the user picks a location. */
  getContent: () => string;
  title?: string;
}

const FEEDBACK_MS = 1500;

/** Header button that saves the current screen's data to a text file. */
export default function ExportButton({
  defaultName,
  filterName,
  extensions,
  getContent,
  title = "Export to file",
}: ExportButtonProps) {
  const [saved, setSaved] = useState(false);

  const handleClick = async () => {
    try {
      const path = await save({
        defaultPath: defaultName,
        filters: [{ name: filterName, extensions }],
      });
      if (!path) return;
      await writeTextFile(path, getContent());
      setSaved(true);
      window.setTimeout(() => setSaved(false), FEEDBACK_MS);
    } catch {
      // Cancelled.
    }
  };

  return (
    <button className={`editor-export-btn ${saved ? "saved" : ""}`} onClick={handleClick} title={title}>
      {saved ? (
        <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
          <polyline points="20 6 9 17 4 12"/>
        </svg>
      ) : (
        <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
          <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
          <polyline points="7 10 12 15 17 10"/>
          <line x1="12" y1="15" x2="12" y2="3"/>
        </svg>
      )}
      {saved ? "Saved" : "Export"}
    </button>
  );
}
