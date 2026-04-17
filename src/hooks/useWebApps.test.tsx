import { renderHook, waitFor } from "@testing-library/react";
import { act } from "react";
import { vi } from "vitest";

import { useWebApps } from "./useWebApps";
import type { WebApp, WebAppPayload } from "../lib/tauri";

const {
  listWebApps,
  createWebApp,
  updateWebApp,
  deleteWebApp,
  launchWebApp,
} = vi.hoisted(() => ({
  listWebApps: vi.fn(),
  createWebApp: vi.fn(),
  updateWebApp: vi.fn(),
  deleteWebApp: vi.fn(),
  launchWebApp: vi.fn(),
}));

vi.mock("../lib/tauri", () => ({
  listWebApps,
  createWebApp,
  updateWebApp,
  deleteWebApp,
  launchWebApp,
  errorMessage: (error: unknown, fallback: string) =>
    error instanceof Error ? error.message : typeof error === "string" ? error : fallback,
}));

function webApp(id: string, name: string): WebApp {
  return {
    id,
    name,
    url: `https://${id}.example.com`,
    iconPath: null,
    category: "Internet",
    browser: "auto",
    navBar: false,
    isolated: true,
    tray: false,
    windowMode: "normal",
    createdAt: "2025-01-01T00:00:00Z",
    updatedAt: "2025-01-01T00:00:00Z",
  };
}

describe("useWebApps", () => {
  it("loads saved web apps on mount", async () => {
    listWebApps.mockResolvedValueOnce([webApp("mail", "Mail")]);

    const { result } = renderHook(() => useWebApps());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.error).toBeNull();
    expect(result.current.webApps.map((item) => item.name)).toEqual(["Mail"]);
  });

  it("keeps created and updated items sorted by name", async () => {
    listWebApps.mockResolvedValueOnce([webApp("zeta", "Zeta"), webApp("mail", "Mail")]);
    createWebApp.mockResolvedValueOnce(webApp("alpha", "Alpha"));
    updateWebApp.mockResolvedValueOnce({
      ...webApp("zeta", "Beta"),
      id: "zeta",
    });

    const { result } = renderHook(() => useWebApps());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    await act(async () => {
      await result.current.create({
        name: "Alpha",
        url: "https://alpha.example.com",
        category: "Internet",
        browser: "auto",
        navBar: false,
        isolated: true,
        tray: false,
        windowMode: "normal",
        uploadedIconDataUrl: null,
        clearIcon: false,
      } satisfies WebAppPayload);
    });

    expect(result.current.webApps.map((item) => item.name)).toEqual(["Alpha", "Mail", "Zeta"]);

    await act(async () => {
      await result.current.update({
        id: "zeta",
        name: "Beta",
        url: "https://zeta.example.com",
        category: "Internet",
        browser: "auto",
        navBar: false,
        isolated: true,
        tray: false,
        windowMode: "normal",
        uploadedIconDataUrl: null,
        clearIcon: false,
      } satisfies WebAppPayload);
    });

    expect(result.current.webApps.map((item) => item.name)).toEqual(["Alpha", "Beta", "Mail"]);
  });

  it("removes deleted items and forwards launches", async () => {
    listWebApps.mockResolvedValueOnce([webApp("mail", "Mail"), webApp("docs", "Docs")]);
    deleteWebApp.mockResolvedValueOnce(undefined);
    launchWebApp.mockResolvedValueOnce(undefined);

    const { result } = renderHook(() => useWebApps());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    await act(async () => {
      await result.current.remove("mail");
    });

    expect(result.current.webApps.map((item) => item.id)).toEqual(["docs"]);

    await act(async () => {
      await result.current.launch("docs");
    });

    expect(launchWebApp).toHaveBeenCalledWith("docs");
  });
});
