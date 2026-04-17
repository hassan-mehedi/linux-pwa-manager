import type { BrowserChoice, DetectedBrowser } from "../lib/tauri";

interface BrowserSelectProps {
  browsers: DetectedBrowser[];
  value: BrowserChoice;
  onChange: (value: BrowserChoice) => void;
}

export function BrowserSelect({
  browsers,
  value,
  onChange,
}: BrowserSelectProps) {
  const options = [
    { id: "auto", label: "Automatic", description: "Choose the best detected browser when launching." },
    ...browsers.map((browser) => ({
      id: browser.id,
      label: browser.version ? `${browser.name} (${browser.version})` : browser.name,
      description: browser.supportsAppMode ? "Standalone app windows are supported." : "Available without full app-window support.",
    })),
    {
      id: "embedded",
      label: "Embedded (built-in)",
      description: "Open inside the built-in embedded browser window.",
    },
  ];

  const selected = options.find((option) => option.id === value) ?? options[0];

  return (
    <div className="mint-field-control">
      <select
        className="mint-input mint-select"
        value={value}
        aria-label="Browser"
        onChange={(event) => onChange(event.target.value as BrowserChoice)}
      >
        {options.map((option) => (
          <option key={option.id} value={option.id}>
            {option.label}
          </option>
        ))}
      </select>

      <p className="mint-field-help">{selected.description}</p>
    </div>
  );
}
