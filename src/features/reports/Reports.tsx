import { useState } from "react";
import {
  reports,
  reportRequest,
  reportCell,
  type Report,
  type ReportKind,
  type ReportRequest,
} from "../../contracts/reports";
import {
  business,
  emptyFilter,
  type References,
  type Row,
} from "../../contracts/business";
import { Filters, SelectField } from "../../components/BusinessForms";
import { ExportActions } from "./ExportActions";
export function Reports({
  refs,
  enabled,
}: {
  refs: References;
  enabled: boolean;
}) {
  const [request, setRequest] = useState<ReportRequest>(
    reportRequest("payroll_summary"),
  );
  const [report, setReport] = useState<Report | null>(null);
  const [settlements, setSettlements] = useState<Row[]>([]);
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  function change(patch: Partial<ReportRequest>) {
    setRequest((r) => ({ ...r, ...patch }));
    setReport(null);
    setError("");
  }
  async function preview() {
    setBusy(true);
    setError("");
    setReport(null);
    try {
      setReport(await reports.preview(request));
    } catch (e) {
      setError(
        typeof e === "string" ? e : "Saved report data could not be loaded.",
      );
    } finally {
      setBusy(false);
    }
  }
  return (
    <>
      <section className="panel business-panel">
        <h2>Choose a saved report</h2>
        <p>
          Local reports preserve saved labels, amounts and financial policy.
          Files are derived copies.
        </p>
        <fieldset disabled={!enabled || busy}>
          <legend className="sr-only">Report selection</legend>
          <label className="field">
            Report type
            <select
              value={request.kind}
              onChange={(e) =>
                change({
                  ...reportRequest(e.target.value as ReportKind),
                  periodStart: request.periodStart,
                  periodEnd: request.periodEnd,
                })
              }
            >
              <option value="payroll_summary">Payroll Summary</option>
              <option value="driver_payroll">Driver Payroll</option>
              <option value="owner_payroll">Owner Payroll</option>
              <option value="settlement_history">Settlement History</option>
              <option value="settlement_detail">
                Weekly Settlement Detail
              </option>
            </select>
          </label>
          <Filters
            refs={refs}
            value={{
              ...emptyFilter,
              periodStart: request.periodStart,
              periodEnd: request.periodEnd,
              driverId: request.driverId,
              truckId: request.truckId,
            }}
            drivers={request.kind !== "owner_payroll"}
            onChange={(f) => change(f)}
          />
          {(request.kind === "owner_payroll" ||
            request.kind === "payroll_summary") && (
            <SelectField
              label="Filter by owner"
              rows={refs.owners}
              value={request.ownerId ?? ""}
              optional
              includeArchived
              onChange={(v) => change({ ownerId: v || null })}
            />
          )}
          {request.kind === "settlement_detail" && (
            <>
              <button
                type="button"
                onClick={() => {
                  setBusy(true);
                  setError("");
                  void business
                    .settlements({
                      periodStart: request.periodStart,
                      periodEnd: request.periodEnd,
                      driverId: request.driverId,
                      truckId: request.truckId,
                    })
                    .then(setSettlements)
                    .catch(() =>
                      setError("Saved settlements could not be loaded."),
                    )
                    .finally(() => setBusy(false));
                }}
              >
                Find Saved Settlements
              </button>
              <SelectField
                label="Saved settlement"
                rows={settlements.map((s) => ({
                  ...s,
                  is_active: "1",
                  name: `${s.period_start} – ${s.period_end} · ${s.driver_name_snapshot} · Truck ${s.truck_number_snapshot} · ${s.status}`,
                }))}
                value={request.settlementId ?? ""}
                onChange={(v) => change({ settlementId: v || null })}
              />
            </>
          )}
          <p className="form-help">
            Date filters include saved periods that overlap the selected range.
            Driver/owner filters apply to their respective payroll areas.
          </p>
          <button
            type="button"
            onClick={() => {
              void preview();
            }}
          >
            Preview Saved Report
          </button>
        </fieldset>
        {!enabled && (
          <p>Reports are available after the desktop database opens.</p>
        )}
        {error && (
          <p role="alert" className="error">
            {error}
          </p>
        )}
      </section>
      {report && (
        <section className="panel business-panel report-preview">
          <h2>{report.title}</h2>
          <p>
            {report.period} · {report.currency}
          </p>
          {report.draft && (
            <p className="notice">
              <strong>DRAFT / NOT FINALIZED</strong> — saved inputs only.
              Unavailable totals are not calculated.
            </p>
          )}
          <ExportActions request={request} enabled={enabled} />
          {report.sections.map((section, index) => (
            <div key={index} className="report-section">
              <h3>{section.name}</h3>
              <div
                className="report-table-scroll"
                role="region"
                aria-label={`${section.name} report table`}
                tabIndex={0}
              >
                <table className="business-table">
                  <caption className="sr-only">{section.name}</caption>
                  <thead>
                    <tr>
                      {section.columns.map((c) => (
                        <th scope="col" key={c}>
                          {c}
                        </th>
                      ))}
                    </tr>
                  </thead>
                  <tbody>
                    {section.rows.map((row, i) => (
                      <tr key={i}>
                        {row.map((cell, j) => (
                          <td key={j}>{reportCell(cell)}</td>
                        ))}
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          ))}
        </section>
      )}
    </>
  );
}
