import { useEffect, useState } from "react";
import { getVersion } from "@tauri-apps/api/app";

/**
 * Versión de `tauri.conf.json` / bundle en runtime. Fuera de Tauri (p. ej. vite solo), cae en "dev".
 */
export function useAppVersion(): string {
  const [version, setVersion] = useState("");

  useEffect(() => {
    let cancelled = false;
    void (async () => {
      try {
        const v = await getVersion();
        if (!cancelled) setVersion(v);
      } catch {
        if (!cancelled) setVersion("dev");
      }
    })();
    return () => {
      cancelled = true;
    };
  }, []);

  return version;
}
