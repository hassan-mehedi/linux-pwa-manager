import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { vi } from "vitest";

const {
  listenMock,
  onResizedMock,
  getWebApp,
  getEmbeddedState,
  embeddedGoBack,
  embeddedGoForward,
  embeddedNavigate,
  embeddedOpenInBrowser,
  embeddedReload,
  embeddedResize,
  toAssetUrl,
} = vi.hoisted(() => ({
  listenMock: vi.fn(),
  onResizedMock: vi.fn(),
  getWebApp: vi.fn(),
  getEmbeddedState: vi.fn(),
  embeddedGoBack: vi.fn(),
  embeddedGoForward: vi.fn(),
  embeddedNavigate: vi.fn(),
  embeddedOpenInBrowser: vi.fn(),
  embeddedReload: vi.fn(),
  embeddedResize: vi.fn(),
  toAssetUrl: vi.fn((path: string | null | undefined) => (path ? `asset://${path}` : null)),
}));

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({
    onResized: onResizedMock,
  }),
}));

vi.mock("@tauri-apps/api/webviewWindow", () => ({
  getCurrentWebviewWindow: () => ({
    listen: listenMock,
  }),
}));

vi.mock("../lib/tauri", () => ({
  EMBEDDED_PAGE_STATE_EVENT: "embedded://page-state",
  getWebApp,
  getEmbeddedState,
  embeddedGoBack,
  embeddedGoForward,
  embeddedNavigate,
  embeddedOpenInBrowser,
  embeddedReload,
  embeddedResize,
  toAssetUrl,
  errorMessage: (error: unknown, fallback: string) =>
    error instanceof Error ? error.message : typeof error === "string" ? error : fallback,
}));

import { EmbeddedShell } from "./EmbeddedShell";

describe("EmbeddedShell", () => {
  beforeEach(() => {
    window.history.replaceState({}, "", "/?mode=embedded&webapp=mail");
    onResizedMock.mockResolvedValue(vi.fn());
    embeddedResize.mockResolvedValue(undefined);
    embeddedNavigate.mockResolvedValue("https://mail.example.com/next");
    embeddedGoBack.mockResolvedValue(undefined);
    embeddedGoForward.mockResolvedValue(undefined);
    embeddedReload.mockResolvedValue(undefined);
    embeddedOpenInBrowser.mockResolvedValue(undefined);
  });

  it("loads the embedded web app state and reacts to toolbar actions", async () => {
    const user = userEvent.setup();

    getWebApp.mockResolvedValueOnce({
      id: "mail",
      name: "Mail",
      url: "https://mail.example.com",
      iconPath: null,
      category: "Internet",
      browser: "embedded",
      navBar: true,
      isolated: true,
      tray: false,
      windowMode: "normal",
      createdAt: "2025-01-01T00:00:00Z",
      updatedAt: "2025-01-01T00:00:00Z",
    });
    getEmbeddedState.mockResolvedValueOnce({
      url: "https://mail.example.com/inbox",
      loading: false,
      canGoBack: true,
      canGoForward: false,
    });
    listenMock.mockResolvedValueOnce(vi.fn());

    render(<EmbeddedShell />);

    await waitFor(() => {
      expect(screen.getByRole("button", { name: "Back" })).toBeEnabled();
    });

    expect(screen.getByDisplayValue("https://mail.example.com/inbox")).toBeInTheDocument();
    expect(screen.getByText("https://mail.example.com/inbox")).toBeInTheDocument();
    expect(embeddedResize).toHaveBeenCalledWith("mail", 0);

    await user.click(screen.getByRole("button", { name: "Back" }));
    await user.click(screen.getByRole("button", { name: "Reload" }));
    await user.click(screen.getByRole("button", { name: "Browser" }));

    const urlField = screen.getByDisplayValue("https://mail.example.com/inbox");
    await user.clear(urlField);
    await user.type(urlField, "example.org");
    await user.click(screen.getByRole("button", { name: "Go" }));

    expect(embeddedGoBack).toHaveBeenCalledWith("mail");
    expect(embeddedReload).toHaveBeenCalledWith("mail");
    expect(embeddedOpenInBrowser).toHaveBeenCalledWith("mail");
    expect(embeddedNavigate).toHaveBeenCalledWith("mail", "example.org");

    await waitFor(() => {
      expect(screen.getByDisplayValue("https://mail.example.com/next")).toBeInTheDocument();
    });
  });

  it("updates page state from embedded events", async () => {
    let pageStateListener:
      | ((event: {
          payload: {
            url: string;
            loading: boolean;
            canGoBack: boolean;
            canGoForward: boolean;
          };
        }) => void)
      | undefined;

    getWebApp.mockResolvedValueOnce({
      id: "mail",
      name: "Mail",
      url: "https://mail.example.com",
      iconPath: null,
      category: "Internet",
      browser: "embedded",
      navBar: true,
      isolated: true,
      tray: false,
      windowMode: "normal",
      createdAt: "2025-01-01T00:00:00Z",
      updatedAt: "2025-01-01T00:00:00Z",
    });
    getEmbeddedState.mockResolvedValueOnce({
      url: "https://mail.example.com",
      loading: true,
      canGoBack: false,
      canGoForward: false,
    });
    listenMock.mockImplementationOnce(async (_eventName, callback) => {
      pageStateListener = callback;
      return vi.fn();
    });

    render(<EmbeddedShell />);

    await waitFor(() => {
      expect(screen.getByText("Loading Mail…")).toBeInTheDocument();
    });

    pageStateListener?.({
      payload: {
        url: "https://mail.example.com/inbox",
        loading: false,
        canGoBack: true,
        canGoForward: true,
      },
    });

    await waitFor(() => {
      expect(screen.getByDisplayValue("https://mail.example.com/inbox")).toBeInTheDocument();
    });

    expect(screen.getByRole("button", { name: "Back" })).toBeEnabled();
    expect(screen.getByRole("button", { name: "Forward" })).toBeEnabled();
    expect(screen.queryByText("Loading Mail…")).not.toBeInTheDocument();
  });

  it("shows a missing target message when no web app id is present", () => {
    window.history.replaceState({}, "", "/?mode=embedded");

    render(<EmbeddedShell />);

    expect(screen.getByText("Missing embedded target.")).toBeInTheDocument();
  });
});
