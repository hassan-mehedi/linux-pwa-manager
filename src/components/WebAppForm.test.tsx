import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { vi } from "vitest";

import { WebAppForm } from "./WebAppForm";
import type { DetectedBrowser, WebApp } from "../lib/tauri";

vi.mock("../lib/tauri", () => ({
  toAssetUrl: vi.fn(() => null),
  errorMessage: (error: unknown, fallback: string) =>
    error instanceof Error ? error.message : typeof error === "string" ? error : fallback,
}));

const browsers: DetectedBrowser[] = [
  {
    id: "chrome",
    name: "Google Chrome",
    path: "/usr/bin/google-chrome",
    version: "123.0.0",
    supportsAppMode: true,
  },
];

const defaults = {
  defaultBrowser: "auto",
  defaultWindowMode: "normal",
} as const;

function sampleWebApp(): WebApp {
  return {
    id: "mail",
    name: "Mail",
    url: "https://mail.example.com",
    iconPath: null,
    category: "Internet",
    browser: "chrome",
    navBar: true,
    isolated: true,
    tray: false,
    windowMode: "maximized",
    createdAt: "2025-01-01T00:00:00Z",
    updatedAt: "2025-01-01T00:00:00Z",
  };
}

describe("WebAppForm", () => {
  it("shows a validation error when required fields are empty", async () => {
    const onSubmit = vi.fn();
    const user = userEvent.setup();

    render(
      <WebAppForm browsers={browsers} defaults={defaults} onClose={vi.fn()} onSubmit={onSubmit} />,
    );

    await user.click(screen.getByRole("button", { name: "Save web app" }));

    expect(screen.getByText("Name and address are required.")).toBeInTheDocument();
    expect(onSubmit).not.toHaveBeenCalled();
  });

  it("trims values before submitting a new web app", async () => {
    const onSubmit = vi.fn().mockResolvedValue(undefined);
    const user = userEvent.setup();

    render(
      <WebAppForm browsers={browsers} defaults={defaults} onClose={vi.fn()} onSubmit={onSubmit} />,
    );

    await user.type(screen.getByLabelText("Name"), "  YouTube  ");
    await user.type(screen.getByLabelText("Address"), "  youtube.com  ");
    await user.click(screen.getByRole("button", { name: "Show advanced settings" }));
    await user.click(screen.getByLabelText("Embedded navigation bar"));
    await user.click(screen.getByRole("button", { name: "Save web app" }));

    expect(onSubmit).toHaveBeenCalledWith({
      name: "YouTube",
      url: "youtube.com",
      category: "Internet",
      browser: "auto",
      navBar: true,
      isolated: true,
      tray: false,
      windowMode: "normal",
      uploadedIconDataUrl: null,
      iconSourceUrl: "https://www.google.com/s2/favicons?domain=youtube.com&sz=128",
      clearIcon: false,
    });
  });

  it("prefills the form when editing an existing web app", () => {
    render(
      <WebAppForm
        browsers={browsers}
        defaults={defaults}
        initialValue={sampleWebApp()}
        onClose={vi.fn()}
        onSubmit={vi.fn()}
      />,
    );

    expect(screen.getByText("Edit Web App")).toBeInTheDocument();
    expect(screen.getByLabelText("Name")).toHaveValue("Mail");
    expect(screen.getByLabelText("Address")).toHaveValue("https://mail.example.com");
    expect(screen.getByLabelText("Browser")).toHaveValue("chrome");
    expect(screen.getByLabelText("Embedded navigation bar")).toBeChecked();
    expect(screen.getByLabelText("Startup window")).toHaveValue("maximized");
  });
});
