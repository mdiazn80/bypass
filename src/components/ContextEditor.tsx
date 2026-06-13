import React, { useEffect, useRef, useState, useCallback } from "react";
import { useContextStore, SYSTEM_HOSTS_ID } from "../stores/useContextStore";
import FileDropZone from "./FileDropZone";
import "./ContextEditor.css";

export function highlightHosts(text: string): React.ReactNode[] {
  return text.split("\n").map((line, i, arr) => {
    const trimmed = line.trimStart();
    const last = i === arr.length - 1;
    const nl = last ? "" : "\n";

    // Comment line
    if (trimmed.startsWith("#")) {
      return (
        <span key={i}>
          <span className="hl-comment">{line}</span>
          {nl}
        </span>
      );
    }

    // Empty line
    if (trimmed === "") {
      return <span key={i}>{nl}</span>;
    }

    // Host entry: IP followed by one or more hostnames
    const match = line.match(/^(\s*)([\d.:a-fA-F]+)([\s]+)(.+)$/);
    if (match) {
      const [, indent, ip, sep, rest] = match;
      const hosts = rest.split(/(\s+)/);
      return (
        <span key={i}>
          {indent}
          <span className="hl-ip">{ip}</span>
          {sep}
          {hosts.map((part, j) =>
            /\s+/.test(part) ? (
              part
            ) : (
              <span key={j} className="hl-host">
                {part}
              </span>
            )
          )}
          {nl}
        </span>
      );
    }

    // Fallback
    return (
      <span key={i}>
        {line}
        {nl}
      </span>
    );
  });
}

export default function ContextEditor() {
  const { contexts, selectedId, update, systemHosts } = useContextStore();
  const selected = contexts.find((c) => c.id === selectedId);
  const [content, setContent] = useState("");
  const [name, setName] = useState("");
  const [editingName, setEditingName] = useState(false);
  const [manualMode, setManualMode] = useState(false);
  const saveTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const highlightRef = useRef<HTMLPreElement>(null);

  useEffect(() => {
    if (selected) {
      setContent(selected.content);
      setName(selected.name);
    }
    setManualMode(false);
    return () => {
      if (saveTimer.current) {
        clearTimeout(saveTimer.current);
      }
    };
  }, [selectedId]);

  const syncScroll = useCallback(() => {
    if (textareaRef.current && highlightRef.current) {
      highlightRef.current.scrollTop = textareaRef.current.scrollTop;
      highlightRef.current.scrollLeft = textareaRef.current.scrollLeft;
    }
  }, []);

  const handleImportFile = useCallback(
    (fileContent: string) => {
      setContent(fileContent);
      if (selectedId && selectedId !== SYSTEM_HOSTS_ID) {
        update(selectedId, undefined, fileContent);
      }
    },
    [selectedId, update]
  );

  if (selectedId === SYSTEM_HOSTS_ID) {
    return (
      <div className="editor">
        <div className="editor-header">
          <h2 className="editor-name">System Hosts</h2>
          <span className="editor-status">Read-only</span>
        </div>
        <pre className="editor-readonly">{highlightHosts(systemHosts)}</pre>
      </div>
    );
  }

  if (!selected) {
    return (
      <div className="editor-empty">
        <p>Select a context from the sidebar to edit its hosts entries</p>
      </div>
    );
  }

  const handleContentChange = (value: string) => {
    setContent(value);
    if (saveTimer.current) clearTimeout(saveTimer.current);
    saveTimer.current = setTimeout(() => {
      update(selected.id, undefined, value);
    }, 500);
  };

  const handleNameSave = () => {
    setEditingName(false);
    const trimmed = name.trim();
    if (trimmed && trimmed !== selected.name) {
      update(selected.id, trimmed);
    }
  };

  // Content for the highlight layer (always ends with newline so cursor space matches)
  const displayContent = content + (content.endsWith("\n") ? " " : "\n ");

  return (
    <div className="editor">
      <div className="editor-header">
        {editingName ? (
          <input
            className="editor-name-input"
            autoFocus
            value={name}
            onChange={(e) => setName(e.target.value)}
            onBlur={handleNameSave}
            onKeyDown={(e) => {
              if (e.key === "Enter") handleNameSave();
              if (e.key === "Escape") {
                setName(selected.name);
                setEditingName(false);
              }
            }}
          />
        ) : (
          <h2
            className="editor-name"
            onDoubleClick={() => setEditingName(true)}
            title="Double-click to rename"
          >
            {selected.name}
          </h2>
        )}
        <span className={`editor-status ${selected.enabled ? "enabled" : ""}`}>
          {selected.enabled ? "Active" : "Inactive"}
        </span>
      </div>
      {!content && !manualMode ? (
        <div className="editor-dropzone-wrap">
          <FileDropZone
            title="Drag & drop a hosts file"
            hint="or click to browse"
            onImport={handleImportFile}
          />
          <button className="editor-manual-link" onClick={() => setManualMode(true)}>
            or edit manually
          </button>
        </div>
      ) : (
      <div className="editor-code-wrap">
        <pre ref={highlightRef} className="editor-highlight" aria-hidden="true">
          {content
            ? highlightHosts(displayContent)
            : <span className="hl-placeholder">{"# Add hosts entries, one per line\n# IPv4 example:\n# 192.168.1.1     mysite.local\n# IPv6 examples:\n# ::1             localhost\n# 2001:db8::1     myhost.local"}</span>
          }
        </pre>
        <textarea
          ref={textareaRef}
          className="editor-textarea"
          value={content}
          onChange={(e) => handleContentChange(e.target.value)}
          onScroll={syncScroll}
          onKeyDown={(e) => {
            if (e.key === "Tab") {
              e.preventDefault();
              const ta = textareaRef.current;
              if (!ta) return;
              const start = ta.selectionStart;
              const end = ta.selectionEnd;
              const newValue = content.substring(0, start) + "\t" + content.substring(end);
              handleContentChange(newValue);
              // Restore cursor position after React re-render
              requestAnimationFrame(() => {
                ta.selectionStart = ta.selectionEnd = start + 1;
              });
            }
          }}
          spellCheck={false}
        />
      </div>
      )}
    </div>
  );
}
