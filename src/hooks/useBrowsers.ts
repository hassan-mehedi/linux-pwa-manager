import { useState, useEffect, useCallback } from "react";
import { errorMessage, listBrowsers, type DetectedBrowser } from "../lib/tauri";

// Module-level cache: persists for the lifetime of the app session,
// resets to null when the user explicitly refreshes.
let cachedBrowsers: DetectedBrowser[] | null = null;

export function useBrowsers() {
  const [browsers, setBrowsers] = useState<DetectedBrowser[]>(() => cachedBrowsers ?? []);
  const [loading, setLoading] = useState(cachedBrowsers === null);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(async (force = false) => {
    if (!force && cachedBrowsers !== null) {
      setBrowsers(cachedBrowsers);
      setLoading(false);
      return;
    }

    setLoading(true);
    setError(null);

    try {
      const items = await listBrowsers();
      cachedBrowsers = items;
      setBrowsers(items);
    } catch (err) {
      setError(errorMessage(err, "Failed to detect browsers."));
    } finally {
      setLoading(false);
    }
  }, []);

  // Run initial load on first mount (only if cache is empty)
  useEffect(() => {
    void load(false);
  }, [load]);

  const refresh = useCallback(() => {
    cachedBrowsers = null;
    void load(true);
  }, [load]);

  return { browsers, loading, error, refresh };
}
