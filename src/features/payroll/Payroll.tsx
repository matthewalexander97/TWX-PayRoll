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
  type BusinessError,
  type Filter,
  type OwnerWrite,
  type References,
  type Row,
} from "../../contracts/business";
import {
  AuditHistory,
  ErrorNotice,
  Field,
  Filters,
  SelectField,
  TextField,
} from "../../components/BusinessForms";
const blankOwner = (): OwnerWrite => ({
  id: null,
  expectedVersion: null,
  ownerId: "",
  truckId: "",
  twxCodeId: null,
  periodStart: "",
  periodEnd: "",
  amount: "",
  notes: "",
});
export function Payroll({
  refs,
  enabled,
  onChanged,
}: {
  refs: References;
  enabled: boolean;
  onChanged: () => void;
}) {
  const [filter, setFilter] = useState<Filter>(emptyFilter);
  const [rows, setRows] = useState<Row[]>([]);
  const [driver, setDriver] = useState<Row | null>(null);
  const [driverForm, setDriverForm] = useState({
    achCharge: "",
    twxCodeId: null as string | null,
    notes: "",
  });
  const [owner, setOwner] = useState<OwnerWrite | null>(null);
  const [events, setEvents] = useState<Row[]>([]);
  const [ownerEvents, setOwnerEvents] = useState<Row[]>([]);
  const [error, setError] = useState<BusinessError | null>(null);
  const [ownerError, setOwnerError] = useState<BusinessError | null>(null);
  const [message, setMessage] = useState("");
  const [ownerMessage, setOwnerMessage] = useState("");
  const [busy, setBusy] = useState(false);
  const [refresh, setRefresh] = useState(0);
  useEffect(() => {
    if (!enabled) return;
    let active = true;
    void business
      .payrolls({ ...filter, driverId: null })
      .then((v) => {
        if (active) setRows(v);
      })
      .catch((c) => {
        if (active) setError(errorOf(c));
      });
    return () => {
      active = false;
    };
  }, [filter, enabled, refresh]);
  useEffect(() => {
    if (!driver?.id) return;
    let active = true;
    void business
      .history("payroll", driver.id)
      .then((v) => {
        if (active) setEvents(v);
      })
      .catch((c) => {
        if (active) setError(errorOf(c));
      });
    return () => {
      active = false;
    };
  }, [driver?.id, refresh]);
  useEffect(() => {
    if (!owner?.id) return;
    let active = true;
    void business
      .history("payroll", owner.id)
      .then((v) => {
        if (active) setOwnerEvents(v);
      })
      .catch((c) => {
        if (active) setOwnerError(errorOf(c));
      });
    return () => {
      active = false;
    };
  }, [owner?.id, refresh]);
  function openDriver(r: Row) {
    setEvents([]);
    setDriver(r);
    setDriverForm({
      achCharge: decimal(r.ach_charge_cents, 2),
      twxCodeId: r.twx_code_id ?? null,
      notes: r.notes ?? "",
    });
    setError(null);
    setMessage("");
  }
  function openOwner(r: Row) {
    setOwnerEvents([]);
    setOwner({
      id: r.id,
      expectedVersion: r.row_version,
      ownerId: r.owner_id ?? "",
      truckId: r.truck_id ?? "",
      twxCodeId: r.twx_code_id ?? null,
      periodStart: r.period_start ?? "",
      periodEnd: r.period_end ?? "",
      amount: decimal(r.amount_cents, 2),
      notes: r.notes ?? "",
    });
    setOwnerError(null);
    setOwnerMessage("");
  }
  async function reload(kind: "DRIVER" | "OWNER", id: string) {
    try {
      const latest = await business.payrolls(emptyFilter());
      const r = latest.find((r) => r.id === id);
      if (!r)
        throw {
          code: "MISSING",
          message: "Record no longer exists",
          field: null,
        };
      if (kind === "DRIVER") openDriver(r);
      else openOwner(r);
    } catch (c) {
      if (kind === "DRIVER") setError(errorOf(c));
      else setOwnerError(errorOf(c));
    }
  }
  const err = (field: string) =>
    ownerError?.field === field ? ownerError.message : undefined;
  const driverRows = rows.filter(
    (r) =>
      r.entry_kind === "DRIVER" &&
      (!filter.driverId || r.driver_id === filter.driverId),
  );
  return (
    <>
      <section className="panel business-panel">
        <h2>Payroll history filters</h2>
        <Filters value={filter} onChange={setFilter} refs={refs} />
        <p className="form-help">
          Driver filtering applies to Driver Payroll. Dates and truck apply to
          both areas.
        </p>
      </section>
      <section className="panel business-panel">
        <div className="section-heading">
          <div>
            <h2>Driver Payroll</h2>
            <p>
              Amounts come from finalized weekly settlements. Edit ACH, code and
              notes here.
            </p>
          </div>
        </div>
        <div
          className="payroll-table"
          role="table"
          aria-label="Driver payroll history"
        >
          <div className="payroll-table-header" role="row">
            <span role="columnheader">Driver / period</span>
            <span role="columnheader">Truck #</span>
            <span role="columnheader">Amount</span>
            <span role="columnheader">ACH Charge</span>
            <span role="columnheader">Total</span>
          </div>
          {driverRows.map((r) => (
            <div className="payroll-table-row" role="row" key={r.id}>
              <div role="cell">
                <button
                  className="text-button"
                  type="button"
                  disabled={busy}
                  onClick={() => openDriver(r)}
                >
                  {r.payee_name_snapshot}
                </button>
                <small>{periodLabel(r.period_start, r.period_end)}</small>
              </div>
              <div role="cell" data-label="Truck #">
                {r.truck_number_snapshot}
              </div>
              <div role="cell" data-label="Amount">
                {money(r.amount_cents)}
              </div>
              <div role="cell" data-label="ACH Charge">
                {money(r.ach_charge_cents)}
              </div>
              <div role="cell" data-label="Total">
                <strong>{money(r.total_cents)}</strong>
              </div>
            </div>
          ))}
        </div>
        {!driverRows.length && (
          <p>
            No driver payroll matches. Finalize a settlement to generate its
            driver Amount.
          </p>
        )}
        {driver && (
          <form
            className="record-editor"
            onSubmit={(e) => {
              e.preventDefault();
              setBusy(true);
              setError(null);
              void business
                .saveDriver({
                  id: driver.id,
                  expectedVersion: driver.row_version,
                  ...driverForm,
                })
                .then((r) => {
                  openDriver(r);
                  setRefresh((v) => v + 1);
                  onChanged();
                  setMessage(
                    "Driver payroll saved. Amount remains linked to the settlement.",
                  );
                })
                .catch((c) => setError(errorOf(c)))
                .finally(() => setBusy(false));
            }}
          >
            <h3>Driver payroll detail</h3>
            <fieldset disabled={busy}>
              <legend className="sr-only">Driver payroll fields</legend>
              <div className="form-grid">
                <TextField
                  label="Driver Name"
                  readOnly
                  value={driver.payee_name_snapshot ?? ""}
                />
                <TextField
                  label="Truck #"
                  readOnly
                  value={driver.truck_number_snapshot ?? ""}
                />
                <TextField
                  label="Week Ending / Pay Period"
                  readOnly
                  value={periodLabel(driver.period_start, driver.period_end)}
                />
                <TextField
                  label="Settlement Amount"
                  readOnly
                  value={money(driver.amount_cents)}
                />
                <SelectField
                  label="TWX Code"
                  value={driverForm.twxCodeId ?? ""}
                  rows={refs.twx_codes}
                  optional
                  onChange={(v) =>
                    setDriverForm({ ...driverForm, twxCodeId: v || null })
                  }
                />
                <TextField
                  label="ACH Charge"
                  value={driverForm.achCharge}
                  error={
                    error?.field === "achCharge" ? error.message : undefined
                  }
                  onChange={(v) =>
                    setDriverForm({ ...driverForm, achCharge: v })
                  }
                />
              </div>
              <Field label="Driver Payroll Notes">
                {(id) => (
                  <textarea
                    id={id}
                    value={driverForm.notes}
                    onChange={(e) =>
                      setDriverForm({ ...driverForm, notes: e.target.value })
                    }
                  />
                )}
              </Field>
              <p className="saved-total">
                Saved Driver Total: <strong>{money(driver.total_cents)}</strong>
              </p>
              <p className="form-help">
                Total reflects the last saved ACH Charge. Linked settlement:{" "}
                {driver.settlement_id}. Saved TWX Code:{" "}
                {driver.twx_code_snapshot ?? "None"}.
              </p>
              <ErrorNotice
                error={error}
                reload={() => {
                  void reload("DRIVER", driver.id);
                }}
              />
              {message && <p role="status">{message}</p>}
              <div className="backup-actions">
                <button type="submit">Save Driver Payroll</button>
                <button type="button" onClick={() => setDriver(null)}>
                  Close Driver Detail
                </button>
              </div>
            </fieldset>
            <AuditHistory events={events} />
          </form>
        )}
        {!driver && <ErrorNotice error={error} />}
        <ExportActions
          request={{ ...reportRequest("driver_payroll"), ...filter }}
          enabled={enabled && !busy}
        />
      </section>
      <section className="panel business-panel">
        <div className="section-heading">
          <div>
            <h2>Owner Payroll</h2>
            <p>Enter the owner Amount manually for a custom pay period.</p>
          </div>
          <button
            type="button"
            disabled={!enabled || busy}
            onClick={() => {
              setOwner(blankOwner());
              setOwnerEvents([]);
              setOwnerError(null);
              setOwnerMessage("");
            }}
          >
            New Owner Payroll
          </button>
        </div>
        <div className="record-list">
          {rows
            .filter((r) => r.entry_kind === "OWNER")
            .map((r) => (
              <button
                className="record-link"
                type="button"
                key={r.id}
                disabled={busy}
                onClick={() => openOwner(r)}
              >
                <strong>
                  {r.payee_name_snapshot} · Truck {r.truck_number_snapshot}
                </strong>
                <span>
                  {periodLabel(r.period_start, r.period_end)} ·{" "}
                  {money(r.amount_cents)}
                </span>
              </button>
            ))}
        </div>
        {!rows.some((r) => r.entry_kind === "OWNER") && (
          <p>No owner payroll matches this period.</p>
        )}
        {owner && (
          <form
            className="record-editor"
            onSubmit={(e) => {
              e.preventDefault();
              setBusy(true);
              setOwnerError(null);
              void business
                .saveOwner(owner)
                .then((r) => {
                  openOwner(r);
                  setRefresh((v) => v + 1);
                  onChanged();
                  setOwnerMessage(
                    "Manual owner payroll saved with audit history.",
                  );
                })
                .catch((c) => setOwnerError(errorOf(c)))
                .finally(() => setBusy(false));
            }}
          >
            <h3>Owner payroll detail</h3>
            <fieldset disabled={busy}>
              <legend className="sr-only">Owner payroll fields</legend>
              <div className="form-grid">
                <SelectField
                  label="Owner Name"
                  value={owner.ownerId}
                  rows={refs.owners}
                  error={err("ownerId")}
                  onChange={(v) => setOwner({ ...owner, ownerId: v })}
                />
                <SelectField
                  label="Owner Truck #"
                  value={owner.truckId}
                  rows={refs.trucks}
                  error={err("truckId")}
                  onChange={(v) => setOwner({ ...owner, truckId: v })}
                />
                <SelectField
                  label="Owner TWX Code"
                  value={owner.twxCodeId ?? ""}
                  rows={refs.twx_codes}
                  optional
                  onChange={(v) => setOwner({ ...owner, twxCodeId: v || null })}
                />
                <TextField
                  label="Owner Amount"
                  value={owner.amount}
                  error={err("amount")}
                  onChange={(v) => setOwner({ ...owner, amount: v })}
                />
                <TextField
                  label="Owner Pay-period start"
                  type="date"
                  value={owner.periodStart}
                  onChange={(v) => setOwner({ ...owner, periodStart: v })}
                />
                <TextField
                  label="Owner Pay-period end"
                  type="date"
                  value={owner.periodEnd}
                  error={err("periodEnd")}
                  onChange={(v) => setOwner({ ...owner, periodEnd: v })}
                />
              </div>
              <Field label="Owner Payroll Notes">
                {(id) => (
                  <textarea
                    id={id}
                    value={owner.notes}
                    onChange={(e) =>
                      setOwner({ ...owner, notes: e.target.value })
                    }
                  />
                )}
              </Field>
              <ErrorNotice
                error={ownerError}
                reload={
                  owner.id
                    ? () => {
                        void reload("OWNER", owner.id!);
                      }
                    : undefined
                }
              />
              {ownerMessage && <p role="status">{ownerMessage}</p>}
              <div className="backup-actions">
                <button type="submit">Save Owner Payroll</button>
                <button type="button" onClick={() => setOwner(null)}>
                  Cancel Owner Payroll
                </button>
              </div>
            </fieldset>
            <AuditHistory events={ownerEvents} />
          </form>
        )}
        <ExportActions
          request={{
            ...reportRequest("owner_payroll"),
            periodStart: filter.periodStart,
            periodEnd: filter.periodEnd,
            truckId: filter.truckId,
          }}
          enabled={enabled && !busy}
        />
      </section>
    </>
  );
}
