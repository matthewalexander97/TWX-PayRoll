import { useEffect, useState } from "react";
import { ExportActions } from "../reports/ExportActions";
import { reportRequest } from "../../contracts/reports";
import {
  business,
  decimal,
  emptyFilter,
  errorOf,
  money,
  periodLabel,
  type Aggregate,
  type BusinessError,
  type Filter,
  type References,
  type Row,
  type SettlementWrite,
  type Totals,
} from "../../contracts/business";
import {
  AuditHistory,
  ErrorNotice,
  Field,
  Filters,
  SelectField,
  TextField,
} from "../../components/BusinessForms";
const blank = (): SettlementWrite => ({
  id: null,
  expectedVersion: null,
  driverId: "",
  truckId: "",
  periodStart: "",
  periodEnd: "",
  notes: "",
  payoutPercentage: null,
  loads: [],
  adjustments: [],
});
export function fromAggregate(a: Aggregate): SettlementWrite {
  return {
    id: a.header.id,
    expectedVersion: a.header.row_version,
    driverId: a.header.driver_id ?? "",
    truckId: a.header.truck_id ?? "",
    periodStart: a.header.period_start ?? "",
    periodEnd: a.header.period_end ?? "",
    notes: a.header.notes ?? "",
    payoutPercentage: decimal(a.header.payout_rate_ppm, 4),
    loads: a.loads.map((l) => ({
      id: l.id,
      driverId: l.driver_id ?? "",
      truckId: l.truck_id ?? "",
      plantId: l.plant_id ?? null,
      jobId: l.job_id ?? null,
      loadNumber: l.load_number ?? "",
      bolNumber: l.bol_number ?? "",
      poNumber: l.po_number ?? "",
      tons: decimal(l.tons_milli, 3),
      completionRate: decimal(l.completion_rate_scaled, 4),
      mileage: l.mileage ?? null,
    })),
    adjustments: a.adjustments.map((v) => ({
      id: v.id,
      adjustmentTypeId: v.adjustment_type_id ?? "",
      twxCodeId: v.twx_code_id ?? null,
      explanation: v.explanation ?? "",
      direction: v.direction === "ADD" ? "ADD" : "DEDUCT",
      amount: decimal(v.amount_cents, 2),
    })),
  };
}
export function Settlements({
  refs,
  enabled,
  onChanged,
}: {
  refs: References;
  enabled: boolean;
  onChanged: () => void;
}) {
  const [filter, setFilter] = useState<Filter>(emptyFilter);
  const [records, setRecords] = useState<Row[]>([]);
  const [form, setForm] = useState<SettlementWrite | null>(null);
  const [saved, setSaved] = useState<Aggregate | null>(null);
  const [totals, setTotals] = useState<Totals | null>(null);
  const [error, setError] = useState<BusinessError | null>(null);
  const [previewError, setPreviewError] = useState<BusinessError | null>(null);
  const [busy, setBusy] = useState(false);
  const [confirmed, setConfirmed] = useState(false);
  const [events, setEvents] = useState<Row[]>([]);
  const [message, setMessage] = useState("");
  const [refresh, setRefresh] = useState(0);
  useEffect(() => {
    if (!enabled) return;
    let active = true;
    void business
      .settlements(filter)
      .then((v) => {
        if (active) setRecords(v);
      })
      .catch((c) => {
        if (active) setError(errorOf(c));
      });
    return () => {
      active = false;
    };
  }, [enabled, filter, refresh]);
  useEffect(() => {
    if (!form || !enabled) return;
    let active = true;
    const timer = window.setTimeout(() => {
      void business
        .preview(form)
        .then((v) => {
          if (active) {
            setTotals(v);
            setPreviewError(null);
          }
        })
        .catch((c) => {
          if (active) {
            setTotals(null);
            if (form.driverId && form.truckId) setPreviewError(errorOf(c));
          }
        });
    }, 250);
    return () => {
      active = false;
      window.clearTimeout(timer);
    };
  }, [form, enabled]);
  useEffect(() => {
    if (!form?.id) return;
    let active = true;
    void business
      .history("settlement", form.id)
      .then((v) => {
        if (active) setEvents(v);
      })
      .catch((c) => {
        if (active) setError(errorOf(c));
      });
    return () => {
      active = false;
    };
  }, [form?.id, refresh]);
  function change(patch: Partial<SettlementWrite>) {
    setForm((f) => (f ? { ...f, ...patch } : f));
    setTotals(null);
    setConfirmed(false);
    setMessage("");
  }
  function accept(a: Aggregate) {
    setEvents([]);
    setSaved(a);
    setForm(fromAggregate(a));
    setConfirmed(false);
    setTotals(null);
    setPreviewError(null);
    setRefresh((v) => v + 1);
    onChanged();
  }
  async function open(id: string) {
    setBusy(true);
    setError(null);
    try {
      accept(await business.settlement(id));
      setMessage("");
    } catch (c) {
      setError(errorOf(c));
    } finally {
      setBusy(false);
    }
  }
  async function save(finalize: boolean) {
    if (!form) return;
    setBusy(true);
    setError(null);
    try {
      let a = await business.saveSettlement(form, confirmed);
      accept(a);
      if (finalize) {
        a = await business.finalize(
          a.header.id,
          a.header.row_version,
          confirmed,
        );
        accept(a);
      }
      setMessage(
        finalize
          ? "Settlement finalized. Linked driver payroll is updated."
          : a.header.status === "FINALIZED"
            ? "Finalized edit saved. Payroll and audit history updated."
            : "Draft saved.",
      );
    } catch (c) {
      setError(errorOf(c));
    } finally {
      setBusy(false);
    }
  }
  const problem = error ?? previewError;
  const err = (field: string) =>
    problem?.field === field ? problem.message : undefined;
  const status = saved?.header.status ?? "NEW";
  const typeName =
    saved?.header.driver_type_name_snapshot ??
    refs.driver_types.find(
      (t) =>
        t.id ===
        refs.drivers.find((d) => d.id === form?.driverId)?.driver_type_id,
    )?.name;
  return (
    <>
      <section className="panel business-panel">
        <div className="section-heading">
          <div>
            <h2>Settlement history</h2>
            <p>Open a saved period or start a new settlement.</p>
          </div>
          <button
            type="button"
            disabled={!enabled || busy}
            onClick={() => {
              setForm(blank());
              setEvents([]);
              setSaved(null);
              setError(null);
              setPreviewError(null);
              setTotals(null);
              setConfirmed(false);
              setMessage("");
            }}
          >
            New Settlement
          </button>
        </div>
        <Filters value={filter} onChange={setFilter} refs={refs} />
        <div className="record-list">
          {records.length ? (
            records.map((r) => (
              <button
                className="record-link"
                key={r.id}
                type="button"
                disabled={busy}
                onClick={() => {
                  void open(r.id);
                }}
              >
                <strong>
                  {r.driver_name_snapshot} · Truck {r.truck_number_snapshot}
                </strong>
                <span>
                  {periodLabel(r.period_start, r.period_end)} · {r.status} ·{" "}
                  {money(r.final_payout_cents)}
                </span>
              </button>
            ))
          ) : (
            <p>No settlements match this period.</p>
          )}
        </div>
      </section>
      {form && (
        <form
          className="settlement-editor"
          onSubmit={(e) => {
            e.preventDefault();
            void save(false);
          }}
        >
          <section className="panel business-panel">
            <div className="section-heading">
              <h2>Settlement detail</h2>
              <span className="status-pill">{status}</span>
            </div>
            <fieldset disabled={busy}>
              <legend className="sr-only">Settlement header</legend>
              <div className="form-grid">
                <TextField
                  label="Pay-period start"
                  type="date"
                  value={form.periodStart}
                  error={err("periodStart")}
                  onChange={(v) => change({ periodStart: v })}
                />
                <TextField
                  label="Pay-period end"
                  type="date"
                  value={form.periodEnd}
                  error={err("periodEnd")}
                  onChange={(v) => change({ periodEnd: v })}
                />
                <SelectField
                  label="Settlement Driver"
                  value={form.driverId}
                  rows={refs.drivers}
                  error={err("driverId")}
                  onChange={(v) => {
                    const type = refs.driver_types.find(
                      (t) =>
                        t.id ===
                        refs.drivers.find((d) => d.id === v)?.driver_type_id,
                    );
                    change({
                      driverId: v,
                      ...(!form.id
                        ? {
                            payoutPercentage:
                              decimal(type?.payout_rate_ppm, 4) || null,
                          }
                        : {}),
                    });
                  }}
                />
                <SelectField
                  label="Settlement Truck #"
                  value={form.truckId}
                  rows={refs.trucks}
                  error={err("truckId")}
                  onChange={(v) => {
                    const driver = refs.trucks.find(
                      (t) => t.id === v,
                    )?.default_driver_id;
                    const type = refs.driver_types.find(
                      (t) =>
                        t.id ===
                        refs.drivers.find((d) => d.id === driver)
                          ?.driver_type_id,
                    );
                    change({
                      truckId: v,
                      ...(!form.id && !form.driverId && driver
                        ? {
                            driverId: driver,
                            payoutPercentage:
                              decimal(type?.payout_rate_ppm, 4) || null,
                          }
                        : {}),
                    });
                  }}
                />
                <TextField
                  label="Driver Type"
                  readOnly
                  value={typeName ?? "Select a driver"}
                />
                <TextField
                  label="Payout Percentage"
                  value={form.payoutPercentage ?? ""}
                  error={err("payoutPercentage")}
                  onChange={(v) => change({ payoutPercentage: v || null })}
                />
              </div>
              <p className="form-help">
                The header driver receives the payout. Actual load
                drivers/trucks may differ. A saved percentage stays unchanged
                unless you explicitly edit it.
              </p>
              {saved && (
                <p className="form-help">
                  Saved labels: {saved.header.driver_name_snapshot} ·{" "}
                  {saved.header.driver_type_name_snapshot} · Truck{" "}
                  {saved.header.truck_number_snapshot}. Driver ID{" "}
                  {form.driverId}; Truck ID {form.truckId}.
                </p>
              )}
              <Field label="Settlement Notes">
                {(id) => (
                  <textarea
                    id={id}
                    value={form.notes}
                    onChange={(e) => change({ notes: e.target.value })}
                  />
                )}
              </Field>
            </fieldset>
          </section>
          <section className="panel business-panel">
            <div className="section-heading">
              <h2>Loads</h2>
              <button
                type="button"
                disabled={busy}
                onClick={() =>
                  change({
                    loads: [
                      ...form.loads,
                      {
                        id: null,
                        driverId: form.driverId,
                        truckId: form.truckId,
                        plantId: null,
                        jobId: null,
                        loadNumber: "",
                        bolNumber: "",
                        poNumber: "",
                        tons: "",
                        completionRate: "",
                        mileage: null,
                      },
                    ],
                  })
                }
              >
                Add Load
              </button>
            </div>
            {form.loads.map((l, i) => {
              const patch = (v: Partial<typeof l>) =>
                change({
                  loads: form.loads.map((r, index) =>
                    index === i ? { ...r, ...v } : r,
                  ),
                });
              return (
                <fieldset
                  key={l.id ?? `new-${i}`}
                  className="load-card"
                  disabled={busy}
                >
                  <legend>Load {i + 1}</legend>
                  <div className="form-grid load-grid">
                    <TextField
                      label="Load ID"
                      value={l.loadNumber}
                      onChange={(v) => patch({ loadNumber: v })}
                    />
                    <TextField
                      label="BOL Number"
                      value={l.bolNumber}
                      onChange={(v) => patch({ bolNumber: v })}
                    />
                    <TextField
                      label="PO Number"
                      value={l.poNumber}
                      onChange={(v) => patch({ poNumber: v })}
                    />
                    <SelectField
                      label="Plant"
                      value={l.plantId ?? ""}
                      rows={refs.plants}
                      optional
                      onChange={(v) => patch({ plantId: v || null })}
                    />
                    <SelectField
                      label="Job Name"
                      value={l.jobId ?? ""}
                      rows={refs.jobs}
                      optional
                      onChange={(v) => patch({ jobId: v || null })}
                    />
                    <SelectField
                      label="Truck #"
                      value={l.truckId}
                      rows={refs.trucks}
                      error={err(`loads.${i}.truckId`)}
                      onChange={(v) => patch({ truckId: v })}
                    />
                    <SelectField
                      label="Driver Name"
                      value={l.driverId}
                      rows={refs.drivers}
                      error={err(`loads.${i}.driverId`)}
                      onChange={(v) => patch({ driverId: v })}
                    />
                    <TextField
                      label="Net Weight / Tons"
                      value={l.tons}
                      error={err(`loads.${i}.tons`)}
                      onChange={(v) => patch({ tons: v })}
                    />
                    <TextField
                      label="Mileage"
                      value={l.mileage ?? ""}
                      error={err(`loads.${i}.mileage`)}
                      onChange={(v) => patch({ mileage: v || null })}
                    />
                    <TextField
                      label="Completion Rate"
                      value={l.completionRate}
                      error={err(`loads.${i}.completionRate`)}
                      onChange={(v) => patch({ completionRate: v })}
                    />
                    <div className="line-total">
                      <span>Load Total</span>
                      <output aria-label={`Load ${i + 1} Total`}>
                        {money(totals?.loadTotalsCents[i])}
                      </output>
                    </div>
                  </div>
                  <button
                    type="button"
                    onClick={() =>
                      change({
                        loads: form.loads.filter((_, index) => index !== i),
                      })
                    }
                  >
                    Remove Load {i + 1}
                  </button>
                </fieldset>
              );
            })}
            {!form.loads.length && (
              <p>No loads yet. Add a row to record a delivery.</p>
            )}
            <p className="form-help">
              Mileage is informational and does not affect payout. Tons allow 3
              decimal places; Completion Rate allows 4.
            </p>
          </section>
          <section className="panel business-panel">
            <div className="section-heading">
              <h2>Adjustments</h2>
              <button
                type="button"
                disabled={busy}
                onClick={() =>
                  change({
                    adjustments: [
                      ...form.adjustments,
                      {
                        id: null,
                        adjustmentTypeId: "",
                        twxCodeId: null,
                        explanation: "",
                        direction: "DEDUCT",
                        amount: "",
                      },
                    ],
                  })
                }
              >
                Add Adjustment
              </button>
            </div>
            {form.adjustments.map((a, i) => {
              const patch = (v: Partial<typeof a>) =>
                change({
                  adjustments: form.adjustments.map((r, index) =>
                    index === i ? { ...r, ...v } : r,
                  ),
                });
              return (
                <fieldset
                  className="load-card"
                  key={a.id ?? `new-${i}`}
                  disabled={busy}
                >
                  <legend>Adjustment {i + 1}</legend>
                  <div className="form-grid">
                    <SelectField
                      label="Adjustment Type"
                      value={a.adjustmentTypeId}
                      rows={refs.adjustment_types}
                      error={err(`adjustments.${i}.adjustmentTypeId`)}
                      onChange={(v) => {
                        const type = refs.adjustment_types.find(
                          (t) => t.id === v,
                        );
                        patch({
                          adjustmentTypeId: v,
                          direction:
                            type?.default_direction === "ADD"
                              ? "ADD"
                              : "DEDUCT",
                          twxCodeId: type?.twx_code_id ?? null,
                        });
                      }}
                    />
                    <SelectField
                      label="TWX Code"
                      value={a.twxCodeId ?? ""}
                      rows={refs.twx_codes}
                      optional
                      onChange={(v) => patch({ twxCodeId: v || null })}
                    />
                    <TextField
                      label="Explanation"
                      value={a.explanation}
                      onChange={(v) => patch({ explanation: v })}
                    />
                    <Field label="Direction">
                      {(id) => (
                        <select
                          id={id}
                          value={a.direction}
                          onChange={(e) =>
                            patch({
                              direction:
                                e.target.value === "ADD" ? "ADD" : "DEDUCT",
                            })
                          }
                        >
                          <option>ADD</option>
                          <option>DEDUCT</option>
                        </select>
                      )}
                    </Field>
                    <TextField
                      label="Adjustment Amount"
                      value={a.amount}
                      error={err(`adjustments.${i}.amount`)}
                      onChange={(v) => patch({ amount: v })}
                    />
                  </div>
                  <button
                    type="button"
                    onClick={() =>
                      change({
                        adjustments: form.adjustments.filter(
                          (_, index) => index !== i,
                        ),
                      })
                    }
                  >
                    Remove Adjustment {i + 1}
                  </button>
                </fieldset>
              );
            })}
          </section>
          <section className="panel business-panel calculation-panel">
            <h2>Settlement calculation</h2>
            <dl className="calculation-summary">
              <dt>Gross Route Total</dt>
              <dd>{money(totals?.grossRouteCents)}</dd>
              <dt>Driver Payout Percentage</dt>
              <dd>{totals ? `${decimal(totals.payoutRatePpm, 4)}%` : "—"}</dd>
              <dt>Driver Gross Payout</dt>
              <dd>{money(totals?.driverGrossCents)}</dd>
              {form.adjustments.map((a, i) => (
                <div className="summary-adjustment" key={i}>
                  <dt>
                    {refs.adjustment_types.find(
                      (t) => t.id === a.adjustmentTypeId,
                    )?.name ?? "Adjustment"}{" "}
                    · {a.direction}
                  </dt>
                  <dd>
                    {a.direction === "ADD" ? "+" : "−"}${a.amount || "—"}
                  </dd>
                </div>
              ))}
              <dt className="final-total">Final Settlement Payout</dt>
              <dd className="final-total" data-testid="final-payout">
                {money(totals?.finalPayoutCents)}
              </dd>
            </dl>
            {totals?.negative && (
              <div className="notice negative-warning">
                <p>
                  Final payout is negative. Review the deductions before saving
                  a finalized edit or finalizing.
                </p>
                <label className="backup-checkbox">
                  <input
                    type="checkbox"
                    checked={confirmed}
                    onChange={(e) => setConfirmed(e.target.checked)}
                  />
                  I confirm this negative final payout.
                </label>
              </div>
            )}
            <ErrorNotice
              error={problem}
              reload={
                form.id
                  ? () => {
                      void open(form.id!);
                    }
                  : undefined
              }
            />
            {message && <p role="status">{message}</p>}
            <div className="backup-actions">
              <button
                type="submit"
                disabled={
                  busy ||
                  !enabled ||
                  (status === "FINALIZED" && !!totals?.negative && !confirmed)
                }
              >
                {busy
                  ? "Saving…"
                  : status === "FINALIZED"
                    ? "Save Finalized Edit"
                    : "Save Draft"}
              </button>
              {status !== "FINALIZED" && (
                <button
                  type="button"
                  disabled={
                    busy ||
                    !enabled ||
                    !totals ||
                    (totals.negative && !confirmed)
                  }
                  onClick={() => {
                    void save(true);
                  }}
                >
                  Finalize Settlement
                </button>
              )}
            </div>
            <AuditHistory events={events} />
            {saved && (
              <ExportActions
                request={{
                  ...reportRequest("settlement_detail"),
                  settlementId: saved.header.id ?? null,
                }}
                enabled={enabled && !busy}
              />
            )}
          </section>
        </form>
      )}
      {!form && <ErrorNotice error={error} />}
    </>
  );
}
