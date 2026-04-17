import { useEffect, useState } from "react";
import {
  createWebApp,
  deleteWebApp,
  errorMessage,
  launchWebApp,
  listWebApps,
  updateWebApp,
  type WebApp,
  type WebAppPayload,
} from "../lib/tauri";

export function useWebApps() {
  const [webApps, setWebApps] = useState<WebApp[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  async function refresh() {
    try {
      setError(null);
      const items = await listWebApps();
      setWebApps(items);
    } catch (err) {
      setError(errorMessage(err, "Failed to load web apps."));
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    void refresh();
  }, []);

  async function create(payload: WebAppPayload) {
    const item = await createWebApp(payload);
    setWebApps((current) => [...current, item].sort((a, b) => a.name.localeCompare(b.name)));
    return item;
  }

  async function update(payload: WebAppPayload) {
    const item = await updateWebApp(payload);
    setWebApps((current) =>
      current
        .map((entry) => (entry.id === item.id ? item : entry))
        .sort((a, b) => a.name.localeCompare(b.name)),
    );
    return item;
  }

  async function remove(target: string) {
    await deleteWebApp(target);
    setWebApps((current) => current.filter((entry) => entry.id !== target));
  }

  async function launch(target: string) {
    await launchWebApp(target);
  }

  return {
    webApps,
    loading,
    error,
    refresh,
    create,
    update,
    remove,
    launch,
  };
}
