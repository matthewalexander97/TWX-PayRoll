import { useId, type ReactNode } from "react";
import {
  type BusinessError,
  type Filter,
  type References,
  type Row,
} from "../contracts/business";
export function Field({
  label,
  error,
  children,
}: {
  label: string;
  error?: string | undefined;
  children: (id: string) => ReactNode;
}) {
  const id = useId();
  return (
    <div className="field">
      <label htmlFor={id}>{label}</label>
      {children(id)}
      {error && (
        <span className="field-error" id={`${id}-error`}>
          {error}
        </span>
      )}
    </div>
  );
}
export function TextField({
  label,
  value,
  onChange,
  error,
  type = "text",
  readOnly = false,
}: {
  label: string;
  value: string;
  onChange?: (v: string) => void;
  error?: string | undefined;
  type?: string;
  readOnly?: boolean;
}) {
  return (
    <Field label={label} error={error}>
      {(id) => (
        <input
          id={id}
          type={type}
          value={value}
          readOnly={readOnly}
          aria-invalid={!!error}
          aria-describedby={error ? `${id}-error` : undefined}
          onChange={(e) => onChange?.(e.target.value)}
          inputMode={
            /Amount|Charge|Tons|Rate|Percentage|Mileage/.test(label)
              ? "decimal"
              : undefined
          }
        />
      )}
    </Field>
  );
}
export function SelectField({
  label,
  value,
  rows,
  onChange,
  error,
  optional = false,
  savedLabel,
  includeArchived = false,
}: {
  label: string;
  value: string;
  rows: Row[];
  onChange: (v: string) => void;
  error?: string | undefined;
  optional?: boolean;
  savedLabel?: string | null | undefined;
  includeArchived?: boolean;
}) {
  return (
    <Field label={label} error={error}>
      {(id) => (
        <select
          id={id}
          value={value}
          aria-invalid={!!error}
          aria-describedby={error ? `${id}-error` : undefined}
          onChange={(e) => onChange(e.target.value)}
        >
          <option value="">{optional ? "None" : "Select…"}</option>
          {rows
            .filter(
              (r) => includeArchived || r.is_active === "1" || r.id === value,
            )
            .map((r) => (
              <option key={r.id} value={r.id}>
                {r.name ?? r.truck_number ?? `${r.code} · ${r.label}`}
                {r.is_active === "0" ? " (archived)" : ""}
              </option>
            ))}
          {value && !rows.some((r) => r.id === value) && (
            <option value={value}>
              {savedLabel ?? value} (saved reference)
            </option>
          )}
        </select>
      )}
    </Field>
  );
}
export function ErrorNotice({
  error,
  reload,
}: {
  error: BusinessError | null;
  reload?: (() => void) | undefined;
}) {
  return error ? (
    <div role="alert" className="error">
      {error.message}
      {error.code === "CONFLICT" && (
        <>
          <p>
            Your entered values are still here. Reload the latest record to
            resolve the conflict.
          </p>
          {reload && (
            <button type="button" onClick={reload}>
              Reload latest record
            </button>
          )}
        </>
      )}
    </div>
  ) : null;
}
export function Filters({
  value,
  onChange,
  refs,
  drivers = true,
}: {
  value: Filter;
  onChange: (v: Filter) => void;
  refs: References;
  drivers?: boolean;
}) {
  return (
    <div className="form-grid filter-grid" aria-label="History filters">
      <TextField
        label="History from"
        type="date"
        value={value.periodStart ?? ""}
        onChange={(v) => onChange({ ...value, periodStart: v || null })}
      />
      <TextField
        label="History through"
        type="date"
        value={value.periodEnd ?? ""}
        onChange={(v) => onChange({ ...value, periodEnd: v || null })}
      />
      {drivers && (
        <SelectField
          label="Filter by driver"
          value={value.driverId ?? ""}
          rows={refs.drivers}
          optional
          includeArchived
          onChange={(v) => onChange({ ...value, driverId: v || null })}
        />
      )}
      <SelectField
        label="Filter by truck"
        value={value.truckId ?? ""}
        rows={refs.trucks}
        optional
        includeArchived
        onChange={(v) => onChange({ ...value, truckId: v || null })}
      />
    </div>
  );
}
export function AuditHistory({ events }: { events: Row[] }) {
  return (
    <details className="audit-history">
      <summary>Record history ({events.length} events)</summary>
      {events.length ? (
        <ol>
          {events.map((e) => (
            <li key={e.id}>
              <strong>{e.category}</strong> · {e.occurred_at}
              <p>{e.reason}</p>
              <details>
                <summary>Saved before / after</summary>
                <pre>
                  {JSON.stringify(
                    {
                      before: JSON.parse(e.before_json ?? "null") as unknown,
                      after: JSON.parse(e.after_json ?? "null") as unknown,
                    },
                    null,
                    2,
                  )}
                </pre>
              </details>
            </li>
          ))}
        </ol>
      ) : (
        <p>No saved events yet.</p>
      )}
    </details>
  );
}
