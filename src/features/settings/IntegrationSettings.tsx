import { useEffect, useState } from "react";
import {
  integrations,
  integrationError,
  type IntegrationStatus,
  type IntegrationService,
} from "../../contracts/integrations";
export function IntegrationSettings({ enabled }: { enabled: boolean }) {
  const [statuses, setStatuses] = useState<IntegrationStatus[]>([]);
  const [busy, setBusy] = useState<IntegrationService | null>(null);
  const [connecting, setConnecting] = useState(false);
  const [error, setError] = useState("");
  const [message, setMessage] = useState("");
  useEffect(() => {
    if (!enabled) return;
    let active = true;
    void integrations
      .status()
      .then((s) => {
        if (active) setStatuses(s);
      })
      .catch((e) => {
        if (active) setError(integrationError(e));
      });
    return () => {
      active = false;
    };
  }, [enabled]);
  async function act(service: IntegrationService, connect: boolean) {
    setBusy(service);
    setConnecting(connect);
    setError("");
    setMessage("");
    try {
      if (connect) {
        const result = await integrations.connect(service);
        if (result) {
          setStatuses(result);
          setMessage("Connection saved securely. No report was sent.");
        } else setMessage("Client-file selection canceled.");
      } else {
        const result = await integrations.disconnect(service);
        setMessage(result.message);
        setStatuses(await integrations.status());
      }
    } catch (e) {
      setError(integrationError(e));
    } finally {
      setBusy(null);
      setConnecting(false);
    }
  }
  return (
    <section className="panel" aria-label="Optional integrations">
      <div className="panel-heading">
        <h2>Optional integrations</h2>
        <p>
          Local payroll, backups and reports work without Google. Sending
          requires network access and your own Google Cloud Desktop OAuth
          client.
        </p>
      </div>
      <p className="form-help">
        Enable Sheets API or Gmail API in your project, configure consent/test
        users, then choose its downloaded Desktop client JSON in Connect.
        Authorization opens in your browser. Sheets and Gmail connect
        separately. No passwords or tokens are shown here.
      </p>
      {(["sheets", "email"] as const).map((service) => {
        const s = statuses.find((v) => v.service === service);
        return (
          <div className="integration-setting" key={service}>
            <h3>{service === "sheets" ? "Google Sheets" : "Email · Gmail"}</h3>
            <p>{s?.account ?? "Disconnected"}</p>
            <p className="form-help">
              {s?.message ??
                (enabled
                  ? "Loading connection status…"
                  : "Available in the desktop app")}
            </p>
            <div className="backup-actions">
              <button
                type="button"
                disabled={!enabled || busy !== null}
                onClick={() => {
                  void act(service, true);
                }}
              >
                {s?.connected ? "Change account / Reconnect" : "Connect"}
              </button>
              <button
                type="button"
                disabled={!enabled || busy !== null}
                onClick={() => {
                  void act(service, false);
                }}
              >
                Disconnect / Clear credential
              </button>
            </div>
          </div>
        );
      })}
      {connecting && (
        <p role="status">
          Complete Google authorization in your browser (up to 3 minutes).{" "}
          <button
            type="button"
            onClick={() => {
              void integrations
                .cancelConnection()
                .catch((e) => setError(integrationError(e)));
            }}
          >
            Cancel connection
          </button>
        </p>
      )}
      <p className="form-help">
        Disconnect removes local credentials first and attempts Google
        revocation. If offline, revoke in Google Account permissions. Google may
        revoke both services for this project. Reports and SQLite backups
        contain no credentials.
      </p>
      {error && (
        <p role="alert" className="error">
          {error}
        </p>
      )}
      {message && <p role="status">{message}</p>}
    </section>
  );
}
