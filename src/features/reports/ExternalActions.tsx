import { useState } from "react";
import {
  integrations,
  integrationError,
  type PreparedExternal,
  type EmailDetails,
  type IntegrationService,
} from "../../contracts/integrations";
import type { ReportRequest, ExportFormat } from "../../contracts/reports";
export function ExternalActions({
  request,
  enabled,
}: {
  request: ReportRequest;
  enabled: boolean;
}) {
  const [compose, setCompose] = useState(false);
  const [email, setEmail] = useState<EmailDetails>({
    recipient: "",
    subject: "TWX Payroll report",
    body: "",
    format: "pdf",
  });
  const [prepared, setPrepared] = useState<PreparedExternal | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [message, setMessage] = useState("");
  const [destination, setDestination] = useState<string | null>(null);
  async function prepare(service: IntegrationService) {
    setBusy(true);
    setError("");
    setMessage("");
    setDestination(null);
    try {
      setPrepared(
        await integrations.prepare(
          request,
          service,
          service === "email" ? email : null,
        ),
      );
    } catch (e) {
      setError(integrationError(e));
    } finally {
      setBusy(false);
    }
  }
  async function finish(send: boolean) {
    if (!prepared) return;
    setBusy(true);
    setError("");
    try {
      if (send) {
        const result = await integrations.confirm(prepared.token);
        setMessage(result.message);
        setDestination(result.destination);
      } else {
        await integrations.cancel(prepared.token);
        setMessage("Canceled. Nothing was sent.");
      }
    } catch (e) {
      setError(integrationError(e));
    } finally {
      setPrepared(null);
      setBusy(false);
    }
  }
  const disabled = !enabled || busy || prepared !== null;
  return (
    <section aria-label="External report actions" className="external-actions">
      <p className="form-help">
        Optional Google outputs require a connection in Settings and network
        access. Only saved report data is sent. Sheets is a separate snapshot;
        email addresses are not saved.
      </p>
      <div className="backup-actions">
        <button
          type="button"
          disabled={disabled}
          onClick={() => {
            void prepare("sheets");
          }}
        >
          Send to Google Sheets
        </button>
        <button
          type="button"
          disabled={disabled}
          onClick={() => setCompose(true)}
        >
          Email Report
        </button>
      </div>
      {compose && !prepared && (
        <form
          onSubmit={(e) => {
            e.preventDefault();
            void prepare("email");
          }}
          aria-label="Email report composition"
        >
          <div className="form-grid">
            <label className="field">
              Recipient
              <input
                type="email"
                required
                maxLength={254}
                disabled={disabled}
                value={email.recipient}
                onChange={(e) =>
                  setEmail((v) => ({ ...v, recipient: e.target.value }))
                }
              />
            </label>
            <label className="field">
              Subject
              <input
                required
                maxLength={200}
                disabled={disabled}
                value={email.subject}
                onChange={(e) =>
                  setEmail((v) => ({ ...v, subject: e.target.value }))
                }
              />
            </label>
            <label className="field">
              Attachment format
              <select
                disabled={disabled}
                value={email.format}
                onChange={(e) =>
                  setEmail((v) => ({
                    ...v,
                    format: e.target.value as ExportFormat,
                  }))
                }
              >
                <option value="pdf">PDF (recommended)</option>
                <option value="xlsx">Excel (.xlsx)</option>
                <option value="docx">Word (.docx)</option>
                <option value="csv">CSV</option>
              </select>
            </label>
            <label className="field">
              Optional message
              <textarea
                maxLength={20000}
                disabled={disabled}
                value={email.body}
                onChange={(e) =>
                  setEmail((v) => ({ ...v, body: e.target.value }))
                }
              />
            </label>
          </div>
          <div className="backup-actions">
            <button disabled={disabled} type="submit">
              Review Email
            </button>
            <button
              disabled={disabled}
              type="button"
              onClick={() => {
                setCompose(false);
                setEmail({
                  recipient: "",
                  subject: "TWX Payroll report",
                  body: "",
                  format: "pdf",
                });
              }}
            >
              Cancel composition
            </button>
          </div>
        </form>
      )}
      {prepared && (
        <section className="panel" aria-label="Confirm external report">
          <h3>
            {prepared.service === "sheets"
              ? "Create a new Google spreadsheet?"
              : "Send this report email?"}
          </h3>
          <p>
            {prepared.title} · {prepared.period}{" "}
            {prepared.draft
              ? "· DRAFT / NOT FINALIZED"
              : "· Saved finalized snapshot"}
          </p>
          <p>Google account: {prepared.account}</p>
          {prepared.recipient && (
            <p>
              To: {prepared.recipient} · Subject: {prepared.subject} ·
              Attachment: {prepared.fileName}
            </p>
          )}
          <p className="form-help">
            {prepared.service === "sheets"
              ? "A new spreadsheet will be created. No existing spreadsheet is overwritten."
              : "Confirm Send deliberately sends this attachment through Gmail. Delivery is not guaranteed."}{" "}
            The prepared saved revision is used even if records change later.
          </p>
          <div className="backup-actions">
            <button
              type="button"
              disabled={busy}
              onClick={() => {
                void finish(true);
              }}
            >
              {prepared.service === "sheets"
                ? "Confirm Create Sheet"
                : "Confirm Send Email"}
            </button>
            <button
              type="button"
              disabled={busy}
              onClick={() => {
                void finish(false);
              }}
            >
              Cancel Send
            </button>
          </div>
        </section>
      )}
      {busy && <p role="status">Working… Local workflows remain available.</p>}
      {error && (
        <p role="alert" className="error">
          {error}
        </p>
      )}
      {message && <p role="status">{message}</p>}
      {destination &&
        /^https:\/\/docs\.google\.com\/spreadsheets\/d\/[A-Za-z0-9_-]+\/edit$/u.test(
          destination,
        ) && (
          <p>
            Spreadsheet created:{" "}
            <span className="data-path">{destination}</span>
          </p>
        )}
    </section>
  );
}
