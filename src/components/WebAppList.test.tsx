import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { vi } from "vitest";

import { WebAppList } from "./WebAppList";
import type { WebApp } from "../lib/tauri";

vi.mock("../lib/tauri", () => ({
  toAssetUrl: vi.fn((path: string | null | undefined) => (path ? `asset://${path}` : null)),
}));

function webApp(id: string, name: string, iconPath: string | null = null): WebApp {
  return {
    id,
    name,
    url: `https://${id}.example.com`,
    iconPath,
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

describe("WebAppList", () => {
  it("shows an empty state when there are no saved web apps", () => {
    render(
      <WebAppList items={[]} selectedId={null} onSelect={vi.fn()} onLaunch={vi.fn()} />,
    );

    expect(screen.getByText("No web apps created yet")).toBeInTheDocument();
    expect(screen.getByText("Use the add button below to create your first launcher.")).toBeInTheDocument();
  });

  it("selects on click and launches on double click", async () => {
    const onSelect = vi.fn();
    const onLaunch = vi.fn();
    const user = userEvent.setup();

    render(
      <WebAppList
        items={[webApp("mail", "Mail"), webApp("docs", "Docs")]}
        selectedId="mail"
        onSelect={onSelect}
        onLaunch={onLaunch}
      />,
    );

    const docsRow = screen.getByText("Docs").closest("button");

    expect(docsRow).not.toBeNull();

    await user.click(docsRow!);
    await user.dblClick(docsRow!);

    expect(onSelect).toHaveBeenCalledWith("docs");
    expect(onLaunch).toHaveBeenCalledWith("docs");
  });

  it("renders the icon fallback initial when no icon is available", () => {
    render(
      <WebAppList
        items={[webApp("mail", "Mail")]}
        selectedId={null}
        onSelect={vi.fn()}
        onLaunch={vi.fn()}
      />,
    );

    expect(screen.getByText("M")).toBeInTheDocument();
  });
});
