import { useState } from "react";
import {
  business,
  decimal,
  errorOf,
  type BusinessError,
  type References,
  type ReferenceKind,
  type ReferenceWrite,
  type Row,
} from "../../contracts/business";
import {
  ErrorNotice,
  SelectField,
  TextField,
} from "../../components/BusinessForms";
export const referenceLabels: Record<ReferenceKind, string> = {
  driver_types: "Driver Types",
  drivers: "Drivers",
  owners: "Owners",
  trucks: "Trucks",
  twx_codes: "TWX Codes",
  plants: "Plants",
  jobs: "Jobs",
  adjustment_types: "Adjustment Types",
};
export function ReferenceManager({
  refs,
  kinds,
  onChanged,
}: {
  refs: References;
  kinds: ReferenceKind[];
  onChanged: () => Promise<void>;
}) {
  const [kind, setKind] = useState<ReferenceKind>(kinds[0] ?? "owners");
  const [form, setForm] = useState<ReferenceWrite | null>(null);
  const [error, setError] = useState<BusinessError | null>(null);
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState("");
  const blank = (k: ReferenceKind): ReferenceWrite => ({
    kind: k,
    id: null,
    expectedVersion: null,
    active: true,
    name: "",
    payoutPercentage: "",
    driverTypeId: "",
    truckNumber: "",
    ownerId: "",
    defaultDriverId: null,
    code: "",
    label: "",
    plantId: null,
    defaultDirection: "DEDUCT",
    twxCodeId: null,
  });
  function edit(r: Row) {
    setForm({
      ...blank(kind),
      id: r.id,
      expectedVersion: r.row_version,
      active: r.is_active === "1",
      name: r.name ?? "",
      payoutPercentage: decimal(r.payout_rate_ppm, 4),
      driverTypeId: r.driver_type_id ?? "",
      truckNumber: r.truck_number ?? "",
      ownerId: r.owner_id ?? "",
      defaultDriverId: r.default_driver_id ?? null,
      code: r.code ?? "",
      label: r.label ?? "",
      plantId: r.plant_id ?? null,
      defaultDirection: r.default_direction ?? "DEDUCT",
      twxCodeId: r.twx_code_id ?? null,
    });
    setError(null);
    setMessage("");
  }
  const update = (patch: Partial<ReferenceWrite>) => {
    if (form) setForm({ ...form, ...patch });
  };
  const err = (field: string) =>
    error?.field === field ? error.message : undefined;
  return (
    <section className="panel business-panel">
      <div className="section-heading">
        <div>
          <h2>Reference records</h2>
          <p>Maintain the names and choices used by payroll and settlements.</p>
        </div>
      </div>
      <div className="section-tabs" aria-label="Reference category">
        {kinds.map((k) => (
          <button
            type="button"
            key={k}
            aria-pressed={kind === k}
            onClick={() => {
              setKind(k);
              setForm(null);
              setError(null);
              setMessage("");
            }}
          >
            {referenceLabels[k]}
          </button>
        ))}
      </div>
      <div className="section-heading">
        <h3>{referenceLabels[kind]}</h3>
        <button
          type="button"
          onClick={() => {
            setForm(blank(kind));
            setError(null);
            setMessage("");
          }}
        >
          Add {referenceLabels[kind].replace(/s$/, "")}
        </button>
      </div>
      <div className="record-list">
        {refs[kind].length ? (
          refs[kind].map((r) => (
            <button
              className="record-link"
              type="button"
              key={r.id}
              onClick={() => edit(r)}
            >
              <strong>
                {r.name ?? r.truck_number ?? `${r.code} · ${r.label}`}
              </strong>
              <span>
                {r.is_active === "1" ? "Active" : "Archived"}
                {kind === "driver_types"
                  ? ` · ${decimal(r.payout_rate_ppm, 4)}%`
                  : ""}
                {kind === "trucks"
                  ? ` · ${refs.owners.find((o) => o.id === r.owner_id)?.name ?? "Owner"}`
                  : ""}
              </span>
            </button>
          ))
        ) : (
          <p>No {referenceLabels[kind].toLowerCase()} yet.</p>
        )}
      </div>
      {form && (
        <form
          className="record-editor"
          onSubmit={(e) => {
            e.preventDefault();
            setBusy(true);
            setError(null);
            void business
              .saveReference(form)
              .then(async () => {
                await onChanged();
                setForm(null);
                setMessage(
                  "Reference saved. Existing financial snapshots keep their saved labels.",
                );
              })
              .catch((c) => setError(errorOf(c)))
              .finally(() => setBusy(false));
          }}
        >
          <h3>
            {form.id ? "Edit" : "Add"} {referenceLabels[kind].replace(/s$/, "")}
          </h3>
          <fieldset disabled={busy}>
            <legend className="sr-only">Reference details</legend>
            <div className="form-grid">
              {kind !== "trucks" && kind !== "twx_codes" && (
                <TextField
                  label="Name"
                  value={form.name ?? ""}
                  error={err("name")}
                  onChange={(v) => update({ name: v })}
                />
              )}
              {kind === "driver_types" && (
                <TextField
                  label="Payout Percentage"
                  value={form.payoutPercentage ?? ""}
                  error={err("payoutPercentage")}
                  onChange={(v) => update({ payoutPercentage: v })}
                />
              )}
              {kind === "drivers" && (
                <SelectField
                  label="Driver Type"
                  value={form.driverTypeId ?? ""}
                  rows={refs.driver_types}
                  error={err("driverTypeId")}
                  onChange={(v) => update({ driverTypeId: v })}
                />
              )}
              {kind === "trucks" && (
                <>
                  <TextField
                    label="Truck #"
                    value={form.truckNumber ?? ""}
                    error={err("truckNumber")}
                    onChange={(v) => update({ truckNumber: v })}
                  />
                  <SelectField
                    label="Owner"
                    value={form.ownerId ?? ""}
                    rows={refs.owners}
                    error={err("ownerId")}
                    onChange={(v) => update({ ownerId: v })}
                  />
                  <SelectField
                    label="Default Driver"
                    value={form.defaultDriverId ?? ""}
                    rows={refs.drivers}
                    optional
                    onChange={(v) => update({ defaultDriverId: v || null })}
                  />
                </>
              )}
              {kind === "twx_codes" && (
                <>
                  <TextField
                    label="TWX Code"
                    value={form.code ?? ""}
                    error={err("code")}
                    onChange={(v) => update({ code: v })}
                  />
                  <TextField
                    label="Code explanation"
                    value={form.label ?? ""}
                    error={err("label")}
                    onChange={(v) => update({ label: v })}
                  />
                </>
              )}
              {kind === "jobs" && (
                <SelectField
                  label="Plant"
                  value={form.plantId ?? ""}
                  rows={refs.plants}
                  optional
                  onChange={(v) => update({ plantId: v || null })}
                />
              )}
              {kind === "adjustment_types" && (
                <>
                  <label className="field">
                    Default Direction
                    <select
                      value={form.defaultDirection}
                      onChange={(e) =>
                        update({ defaultDirection: e.target.value })
                      }
                    >
                      <option>ADD</option>
                      <option>DEDUCT</option>
                    </select>
                  </label>
                  <SelectField
                    label="Default TWX Code"
                    value={form.twxCodeId ?? ""}
                    rows={refs.twx_codes}
                    optional
                    onChange={(v) => update({ twxCodeId: v || null })}
                  />
                </>
              )}
            </div>
            <label className="backup-checkbox">
              <input
                type="checkbox"
                checked={form.active}
                onChange={(e) => update({ active: e.target.checked })}
              />
              Active for new selection
            </label>
            {kind === "adjustment_types" && !form.id && (
              <div className="backup-actions">
                {[
                  "Insurance",
                  "Hopper",
                  "Reimbursement",
                  "Apportioned Plates",
                ].map((name) => (
                  <button
                    type="button"
                    key={name}
                    onClick={() =>
                      update({
                        name,
                        defaultDirection:
                          name === "Reimbursement" ? "ADD" : "DEDUCT",
                      })
                    }
                  >
                    Use {name}
                  </button>
                ))}
              </div>
            )}
            <ErrorNotice error={error} />
            <div className="backup-actions">
              <button type="submit">
                {busy ? "Saving…" : "Save Reference"}
              </button>
              <button type="button" onClick={() => setForm(null)}>
                Cancel
              </button>
            </div>
          </fieldset>
        </form>
      )}
      {message && <p role="status">{message}</p>}
    </section>
  );
}
