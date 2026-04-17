import { renderHook, waitFor } from "@testing-library/react";
import { act } from "react";
import { vi } from "vitest";

const { listBrowsers } = vi.hoisted(() => ({
  listBrowsers: vi.fn(),
}));

vi.mock("../lib/tauri", () => ({
  listBrowsers,
  errorMessage: (error: unknown, fallback: string) =>
    error instanceof Error ? error.message : typeof error === "string" ? error : fallback,
}));

describe("useBrowsers", () => {
  afterEach(() => {
    vi.resetModules();
    listBrowsers.mockReset();
  });

  it("caches detected browsers and refreshes on demand", async () => {
    listBrowsers.mockResolvedValueOnce([
      {
        id: "chrome",
        name: "Google Chrome",
        path: "/usr/bin/google-chrome",
        version: "123",
        supportsAppMode: true,
      },
    ]);

    const { useBrowsers } = await import("./useBrowsers");
    const first = renderHook(() => useBrowsers());

    await waitFor(() => {
      expect(first.result.current.loading).toBe(false);
    });
    expect(listBrowsers).toHaveBeenCalledTimes(1);

    const second = renderHook(() => useBrowsers());
    await waitFor(() => {
      expect(second.result.current.loading).toBe(false);
    });
    expect(listBrowsers).toHaveBeenCalledTimes(1);

    listBrowsers.mockResolvedValueOnce([
      {
        id: "firefox",
        name: "Firefox",
        path: "/usr/bin/firefox",
        version: "124",
        supportsAppMode: false,
      },
    ]);

    await act(async () => {
      second.result.current.refresh();
    });

    await waitFor(() => {
      expect(second.result.current.browsers[0]?.id).toBe("firefox");
    });
    expect(listBrowsers).toHaveBeenCalledTimes(2);
  });
});
