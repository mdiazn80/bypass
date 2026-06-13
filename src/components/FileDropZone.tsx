import { useEffect, useRef, useState } from "react";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { open } from "@tauri-apps/plugin-dialog";
import { readFileText } from "../services/tauri";
import "./FileDropZone.css";

interface FileDropZoneProps {
  title: string;
  hint: string;
  /** Called with the file's text content and its file name. */
  onImport: (content: string, fileName: string) => void;
}

/**
 * A drop target that imports a single text file, either by dragging it from the
 * OS or by clicking to browse. File contents are read through the backend so
 * dropped paths (which are outside the fs plugin scope) can be read reliably.
 */
export default function FileDropZone({ title, hint, onImport }: FileDropZoneProps) {
  const boxRef = useRef<HTMLDivElement>(null);
  const [hovering, setHovering] = useState(false);

  useEffect(() => {
    let unlisten: (() => void) | undefined;

    const isInside = (position: { x: number; y: number }) => {
      const el = boxRef.current;
      if (!el) return false;
      const rect = el.getBoundingClientRect();
      const dpr = window.devicePixelRatio || 1;
      const x = position.x / dpr;
      const y = position.y / dpr;
      return x >= rect.left && x <= rect.right && y >= rect.top && y <= rect.bottom;
    };

    const importPath = async (path: string) => {
      try {
        const content = await readFileText(path);
        const name = path.split(/[\\/]/).pop() ?? path;
        onImport(content, name);
      } catch {
        // Unreadable file; ignored.
      }
    };

    getCurrentWebview()
      .onDragDropEvent((event) => {
        const payload = event.payload;
        if (payload.type === "enter" || payload.type === "over") {
          setHovering(isInside(payload.position));
        } else if (payload.type === "drop") {
          const inside = isInside(payload.position);
          setHovering(false);
          if (inside && payload.paths.length > 0) {
            void importPath(payload.paths[0]);
          }
        } else {
          setHovering(false);
        }
      })
      .then((fn) => {
        unlisten = fn;
      });

    return () => {
      if (unlisten) unlisten();
    };
  }, [onImport]);

  const handleClick = async () => {
    try {
      const selected = await open({ multiple: false });
      if (typeof selected === "string") {
        const content = await readFileText(selected);
        const name = selected.split(/[\\/]/).pop() ?? selected;
        onImport(content, name);
      }
    } catch {
      // Cancelled.
    }
  };

  return (
    <div
      ref={boxRef}
      className={`dropzone ${hovering ? "hovering" : ""}`}
      onClick={handleClick}
    >
      <div className="dropzone-icon">⬇</div>
      <div className="dropzone-title">{title}</div>
      <div className="dropzone-hint">{hint}</div>
    </div>
  );
}
