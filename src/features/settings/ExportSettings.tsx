import { useEffect, useState } from "react";
import { exportSettings, type ExportPreference } from "../../contracts/exports";
export function ExportSettings({ enabled }: { enabled: boolean }) {
  const [preference, setPreference] = useState<ExportPreference | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [message, setMessage] = useState("");
  const failure = (e: unknown) =>
    typeof e === "string"
      ? e
      : "Export folder unavailable. Change Folder or Reset to Documents.";
  useEffect(() => {
    if (!enabled) return;
    let active = true;
    void exportSettings
      .status()
      .then((p) => {
        if (active) setPreference(p);
      })
      .catch((e) => {
        if (active) setError(failure(e));
      });
    return () => {
      active = false;
    };
  }, [enabled]);
  async function run(action: "select" | "reset" | "open") {
    setBusy(true);
    setError("");
    setMessage("");
    try {
      const result = await exportSettings[action]();
      if (result) setPreference(result);
      setMessage(
        action === "open"
          ? "Export folder opened."
          : result
            ? "Export destination saved."
            : "Folder selection canceled. Previous destination retained.",
      );
    } catch (e) {
      setError(failure(e));
    } finally {
      setBusy(false);
    }
  }
  return (
    <section className="panel" aria-labelledby="export-settings-heading">
      <div className="panel-heading">
        <h2 id="export-settings-heading">Export Settings</h2>
        <p>
          PDF, Excel, CSV and Word copies use this folder. Exports are not
          backups; backup protection has its own destination below.
        </p>
      </div>
      <div className="settings-body">
        <dl className="storage-details">
          <dt>Default Export Folder</dt>
          <dd className="data-path">
            {preference?.directory ??
              (enabled
                ? "Choose a folder if Documents is unavailable."
                : "Available in the desktop application")}
          </dd>
          <dt>Source</dt>
          <dd>
            {preference
              ? preference.usesDocuments
                ? "Windows Documents"
                : "Your selected folder"
              : "Not loaded"}
          </dd>
        </dl>
        <div className="backup-actions">
          {(
            [
              ["select", "Change Folder"],
              ["reset", "Reset to Documents"],
              ["open", "Open Export Folder"],
            ] as const
          ).map(([action, label]) => (
            <button
              key={action}
              disabled={!enabled || busy}
              onClick={() => {
                void run(action);
              }}
            >
              {label}
            </button>
          ))}
        </div>
        {error && (
          <p className="error" role="alert">
            {error}
          </p>
        )}
        {message && <p role="status">{message}</p>}
      </div>
    </section>
  );
}
