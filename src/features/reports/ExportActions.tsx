import { useState } from "react";
import { ExternalActions } from "./ExternalActions";
import {
  reports,
  type ReportRequest,
  type ExportFormat,
} from "../../contracts/reports";
export function ExportActions({
  request,
  enabled = true,
}: {
  request: ReportRequest;
  enabled?: boolean;
}) {
  const [format, setFormat] = useState<ExportFormat>("pdf");
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState("");
  const [error, setError] = useState("");
  async function run(action: "export" | "saveAs" | "print") {
    setBusy(true);
    setError("");
    setMessage("");
    try {
      const result =
        action === "print"
          ? await reports.print(request)
          : await reports[action](request, format);
      setMessage(
        result
          ? `${result.message} ${result.fileName}`
          : "Save canceled. No file was published.",
      );
    } catch (e) {
      setError(
        typeof e === "string"
          ? e
          : "The report could not be generated. Review saved records and choose a writable destination.",
      );
    } finally {
      setBusy(false);
    }
  }
  return (
    <section className="export-actions" aria-label="Report export actions">
      <p className="form-help">
        Reports use saved database records. Save your edits before exporting.
        Export uses the folder in Settings. Save As chooses a one-off
        destination. Print opens a stable PDF in your local viewer.
      </p>
      <div className="backup-actions">
        <label className="field">
          Export format
          <select
            value={format}
            disabled={busy}
            onChange={(e) => setFormat(e.target.value as ExportFormat)}
          >
            <option value="pdf">PDF</option>
            <option value="xlsx">Excel (.xlsx)</option>
            <option value="csv">CSV</option>
            <option value="docx">Word (.docx)</option>
          </select>
        </label>
        <button
          type="button"
          disabled={!enabled || busy}
          onClick={() => {
            void run("export");
          }}
        >
          {busy ? "Working…" : "Export Saved Report"}
        </button>
        <button
          type="button"
          disabled={!enabled || busy}
          onClick={() => {
            void run("print");
          }}
        >
          Print / Preview PDF
        </button>
        <button
          type="button"
          disabled={!enabled || busy}
          onClick={() => {
            void run("saveAs");
          }}
        >
          Save As…
        </button>
      </div>
      {error && (
        <p role="alert" className="error">
          {error}
        </p>
      )}
      {message && <p role="status">{message}</p>}
      <ExternalActions request={request} enabled={enabled && !busy} />
    </section>
  );
}
