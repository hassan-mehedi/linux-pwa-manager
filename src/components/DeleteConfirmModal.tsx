import { useEffect, useRef } from "react";

interface DeleteConfirmModalProps {
  appName: string;
  onConfirm: () => void;
  onCancel: () => void;
}

export function DeleteConfirmModal({ appName, onConfirm, onCancel }: DeleteConfirmModalProps) {
  const dialogRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const el = dialogRef.current;
    if (!el) return;

    const focusable = Array.from(
      el.querySelectorAll<HTMLElement>(
        'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
      )
    );
    const first = focusable[0];
    const last = focusable[focusable.length - 1];

    function onKeyDown(e: KeyboardEvent) {
      if (e.key === "Escape") {
        e.preventDefault();
        e.stopPropagation();
        onCancel();
        return;
      }
      if (e.key === "Tab") {
        if (e.shiftKey) {
          if (document.activeElement === first) {
            e.preventDefault();
            last?.focus();
          }
        } else {
          if (document.activeElement === last) {
            e.preventDefault();
            first?.focus();
          }
        }
      }
    }

    el.addEventListener("keydown", onKeyDown);
    return () => el.removeEventListener("keydown", onKeyDown);
  }, [onCancel]);

  return (
    <div className="mint-dialog-backdrop" role="presentation">
      <div
        ref={dialogRef}
        className="mint-dialog"
        role="alertdialog"
        aria-modal="true"
        aria-labelledby="delete-confirm-title"
        aria-describedby="delete-confirm-desc"
        style={{ maxWidth: "420px" }}
      >
        <header className="mint-dialog-titlebar">
          <div className="mint-dialog-copy">
            <p className="app-eyebrow">Confirm deletion</p>
            <h2 id="delete-confirm-title">Delete {appName}?</h2>
            <p id="delete-confirm-desc">
              This will permanently remove the desktop entry and browser profile for this web app.
              This action cannot be undone.
            </p>
          </div>
        </header>

        <footer className="mint-dialog-actions">
          <button className="mint-secondary-button" type="button" onClick={onCancel}>
            Cancel
          </button>
          <button
            className="mint-primary-button"
            type="button"
            style={{ background: "var(--danger)" }}
            onClick={onConfirm}
            autoFocus
          >
            Delete
          </button>
        </footer>
      </div>
    </div>
  );
}
