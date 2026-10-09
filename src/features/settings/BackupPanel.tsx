import { useEffect, useState } from "react";
import {
  recovery,
  type BackupStatus,
  type RestorePreview,
} from "../../contracts/desktop";
export function BackupPanel({
  enabled,
  onRestored,
}: {
  enabled: boolean;
  onRestored: () => Promise<void>;
}) {
  const [status, setStatus] = useState<BackupStatus | null>(null);
  const [preview, setPreview] = useState<RestorePreview | null>(null);
  const [confirmed, setConfirmed] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [message, setMessage] = useState("");
  useEffect(() => {
    if (!enabled) return;
    let active = true;
    const refresh = () => {
      void recovery
        .status()
        .then((value) => {
          if (active) setStatus(value);
        })
        .catch((cause) => {
          if (active) setError(String(cause));
        });
    };
    refresh();
    const timer = window.setInterval(refresh, 60_000);
    return () => {
      active = false;
      window.clearInterval(timer);
    };
  }, [enabled]);
  async function run(operation: () => Promise<void>) {
    setBusy(true);
    setError("");
    setMessage("");
    try {
      await operation();
    } catch (cause) {
      setError(String(cause));
      try {
        setStatus(await recovery.status());
      } catch {
        /* Keep the actionable operation error. */
      }
    } finally {
      setBusy(false);
    }
  }
  const disabled = !enabled || busy;
  return (
    <section className="panel backup-panel" aria-labelledby="backup-heading">
      <div className="panel-heading">
        <h2 id="backup-heading">Backup & recovery</h2>
        <p>
          Keep complete copies of your workspace and return to a saved copy when
          needed.
        </p>
      </div>
      <div className="settings-body">
        {!enabled ? (
          <p>
            Backup and recovery are available in the desktop application after
            storage opens.
          </p>
        ) : (
          <>
            <p className="notice">
              {status?.notice ?? "Loading backup status…"}
            </p>
            <dl className="storage-details">
              <dt>Backup folder</dt>
              <dd className="data-path">
                {status?.destination ?? "Not configured"}
              </dd>
              <dt>Last successful backup</dt>
              <dd>
                {status?.lastSuccessfulBackup ??
                  "No successful backup recorded"}
              </dd>
            </dl>
            <div className="backup-actions">
              <button
                disabled={disabled || preview !== null}
                onClick={() => {
                  void run(async () => {
                    const selected = await recovery.selectFolder();
                    if (selected) setStatus(selected);
                  });
                }}
              >
                Select Backup Folder
              </button>
              <button
                disabled={disabled || !status?.destination || preview !== null}
                onClick={() => {
                  void run(async () => {
                    setStatus(await recovery.backup());
                    setMessage("Backup created and validated.");
                  });
                }}
              >
                Backup Now
              </button>
              <button
                disabled={disabled || preview !== null}
                onClick={() => {
                  void run(async () => {
                    setPreview(await recovery.selectRestore());
                    setConfirmed(false);
                  });
                }}
              >
                Restore Backup
              </button>
            </div>
            <div className="settings-inset">
              <label className="backup-checkbox">
                <input
                  type="checkbox"
                  checked={status?.automaticEnabled ?? false}
                  disabled={disabled || !status || preview !== null}
                  onChange={(event) => {
                    const value = event.target.checked;
                    void run(async () => {
                      setStatus(await recovery.automatic(value));
                    });
                  }}
                />
                Automatic backups while the app is open
              </label>
              <ul className="backup-help">
                <li>
                  After business data changes, an automatic backup is made when
                  the previous successful backup is at least 24 hours old.
                </li>
                <li>
                  The latest {status?.retentionCount ?? 10} automatic copies are
                  retained. Manual backups and pre-restore safety copies are
                  kept.
                </li>
              </ul>
            </div>
            {preview && (
              <div
                className="restore-confirmation"
                role="group"
                aria-labelledby="restore-heading"
              >
                <h3 id="restore-heading">Restore this backup?</h3>
                <p className="data-path">{preview.filename}</p>
                {preview.createdAt && <p>Saved {preview.createdAt}</p>}
                <p>
                  Restoring replaces your active records and appearance with
                  this saved copy. Your backup folder stays configured. A safety
                  copy of your current data will be kept first.
                </p>
                <label className="backup-checkbox">
                  <input
                    type="checkbox"
                    checked={confirmed}
                    disabled={busy}
                    onChange={(event) => setConfirmed(event.target.checked)}
                  />
                  I understand that this replaces the active workspace.
                </label>
                <div className="backup-actions">
                  <button
                    disabled={busy || !confirmed}
                    onClick={() => {
                      void run(async () => {
                        setStatus(await recovery.restore(preview.token));
                        setPreview(null);
                        setConfirmed(false);
                        await onRestored();
                        setMessage(
                          "Backup restored. The workspace has been refreshed.",
                        );
                      });
                    }}
                  >
                    Confirm Restore
                  </button>
                  <button
                    disabled={busy}
                    onClick={() => {
                      void run(async () => {
                        await recovery.cancelRestore();
                        setPreview(null);
                        setConfirmed(false);
                      });
                    }}
                  >
                    Cancel Restore
                  </button>
                </div>
              </div>
            )}
            {busy && <p role="status">Working… keep the application open.</p>}
            {message && <p role="status">{message}</p>}
            {error && (
              <p role="alert" className="error">
                {error}
              </p>
            )}
            <h3>Backup history</h3>
            {status?.runs.length ? (
              <ol className="backup-history">
                {status.runs.map((row) => (
                  <li key={row.id}>
                    <strong>
                      {row.kind.replaceAll("_", " ")} ·{" "}
                      {row.status.replaceAll("_", " ")}
                    </strong>
                    <span>{row.createdAt}</span>
                    <span className="data-path">{row.path}</span>
                    <span>{row.validation}</span>
                    {row.errorSummary && <span>{row.errorSummary}</span>}
                  </li>
                ))}
              </ol>
            ) : (
              <p>No backup operations recorded yet.</p>
            )}
          </>
        )}
      </div>
    </section>
  );
}
