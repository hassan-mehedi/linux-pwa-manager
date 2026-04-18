import { render, screen, waitFor } from "@testing-library/react";
import { vi } from "vitest";

import { SettingsView } from "./SettingsView";

const { getAppInfo, getSettings } = vi.hoisted(() => ({
  getAppInfo: vi.fn(),
  getSettings: vi.fn(),
}));

vi.mock("../lib/tauri", () => ({
  getAppInfo,
  getSettings,
  saveSettings: vi.fn().mockResolvedValue(undefined),
  errorMessage: (error: unknown, fallback: string) =>
    error instanceof Error ? error.message : typeof error === "string" ? error : fallback,
}));

const mockInfo = {
  dataDir: "/home/mehedi/.local/share/linux-pwa-manager",
  configDir: "/home/mehedi/.config/linux-pwa-manager",
  stateDir: "/home/mehedi/.local/state/linux-pwa-manager",
  dbPath: "/home/mehedi/.local/share/linux-pwa-manager/webapps.db",
  desktopDir: "/home/mehedi/.local/share/applications",
  profilesDir: "/home/mehedi/.local/share/linux-pwa-manager/profiles",
  iconsDir: "/home/mehedi/.local/share/linux-pwa-manager/icons",
};

const mockSettings = {
  httpTimeoutSecs: 10,
  maxIconSizeMb: 5,
  historyLimit: 50,
  defaultBrowser: "auto",
  defaultWindowMode: "normal",
  launchOnLogin: false,
  theme: "light",
};

describe("SettingsView", () => {
  it("shows loading placeholders before data arrives", () => {
    getAppInfo.mockReturnValue(new Promise(() => {}));
    getSettings.mockReturnValue(new Promise(() => {}));

    render(<SettingsView />);

    expect(screen.getByText("Loading paths…")).toBeInTheDocument();
    expect(screen.getByText("Loading preferences…")).toBeInTheDocument();
  });

  it("renders discovered manager paths and preferences after loading", async () => {
    getAppInfo.mockResolvedValueOnce(mockInfo);
    getSettings.mockResolvedValueOnce(mockSettings);

    render(<SettingsView />);

    await waitFor(() => {
    expect(screen.getByText("/home/mehedi/.local/share/linux-pwa-manager/webapps.db")).toBeInTheDocument();
    });

    expect(screen.getByText("/home/mehedi/.config/linux-pwa-manager")).toBeInTheDocument();
    expect(screen.getByText("/home/mehedi/.local/state/linux-pwa-manager")).toBeInTheDocument();
    expect(screen.getByText("/home/mehedi/.local/share/linux-pwa-manager/icons")).toBeInTheDocument();
    expect(screen.getByText("/home/mehedi/.local/share/linux-pwa-manager/profiles")).toBeInTheDocument();
    expect(screen.getByText("/home/mehedi/.local/share/applications")).toBeInTheDocument();
    expect(screen.getByText("Save settings")).toBeInTheDocument();
    expect(screen.getByRole("combobox", { name: "Theme" })).toHaveValue("light");
  });

  it("shows an error when loading fails", async () => {
    getAppInfo.mockRejectedValueOnce(new Error("boom"));
    getSettings.mockRejectedValueOnce(new Error("boom"));

    render(<SettingsView />);

    await waitFor(() => {
      expect(screen.getByText("boom")).toBeInTheDocument();
    });
  });
});
