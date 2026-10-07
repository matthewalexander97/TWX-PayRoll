import { invoke } from "@tauri-apps/api/core";
export type Row = {
  id: string;
  row_version: string;
  [key: string]: string | null;
};
export type ReferenceKind =
  | "driver_types"
  | "drivers"
  | "owners"
  | "trucks"
  | "twx_codes"
  | "plants"
  | "jobs"
  | "adjustment_types";
export type References = Record<ReferenceKind, Row[]>;
export const emptyReferences = (): References => ({
  driver_types: [],
  drivers: [],
  owners: [],
  trucks: [],
  twx_codes: [],
  plants: [],
  jobs: [],
  adjustment_types: [],
});
export interface Filter {
  periodStart: string | null;
  periodEnd: string | null;
  driverId: string | null;
  truckId: string | null;
}
export const emptyFilter = (): Filter => ({
  periodStart: null,
  periodEnd: null,
  driverId: null,
  truckId: null,
});
export interface LoadWrite {
  id: string | null;
  driverId: string;
  truckId: string;
  plantId: string | null;
  jobId: string | null;
  loadNumber: string;
  bolNumber: string;
  poNumber: string;
  tons: string;
  completionRate: string;
  mileage: string | null;
}
export interface AdjustmentWrite {
  id: string | null;
  adjustmentTypeId: string;
  twxCodeId: string | null;
  explanation: string;
  direction: "ADD" | "DEDUCT";
  amount: string;
}
export interface SettlementWrite {
  id: string | null;
  expectedVersion: string | null;
  driverId: string;
  truckId: string;
  periodStart: string;
  periodEnd: string;
  notes: string;
  payoutPercentage: string | null;
  loads: LoadWrite[];
  adjustments: AdjustmentWrite[];
}
export interface Aggregate {
  header: Row;
  loads: Row[];
  adjustments: Row[];
}
export interface Totals {
  loadTotalsCents: string[];
  payoutRatePpm: string;
  grossRouteCents: string;
  driverGrossCents: string;
  finalPayoutCents: string;
  negative: boolean;
}
export interface BusinessError {
  code: string;
  message: string;
  field: string | null;
}
export function errorOf(cause: unknown): BusinessError {
  if (typeof cause === "object" && cause !== null && "message" in cause)
    return {
      code: "code" in cause ? String(cause.code) : "ERROR",
      message: String(cause.message),
      field:
        "field" in cause && cause.field !== null ? String(cause.field) : null,
    };
  return { code: "ERROR", message: String(cause), field: null };
}
export interface ReferenceWrite {
  kind: ReferenceKind;
  id: string | null;
  expectedVersion: string | null;
  active: boolean;
  name?: string;
  payoutPercentage?: string;
  driverTypeId?: string;
  truckNumber?: string;
  ownerId?: string;
  defaultDriverId?: string | null;
  code?: string;
  label?: string;
  plantId?: string | null;
  defaultDirection?: string;
  twxCodeId?: string | null;
}
export interface OwnerWrite {
  id: string | null;
  expectedVersion: string | null;
  ownerId: string;
  truckId: string;
  twxCodeId: string | null;
  periodStart: string;
  periodEnd: string;
  amount: string;
  notes: string;
}
export const business = {
  references: () => invoke<References>("get_reference_data"),
  saveReference: (input: ReferenceWrite) =>
    invoke<Row>("save_reference_record", { input }),
  settlements: (filter: Filter) =>
    invoke<Row[]>("list_settlement_records", { filter }),
  settlement: (id: string) =>
    invoke<Aggregate>("get_settlement_record", { id }),
  preview: (input: SettlementWrite) =>
    invoke<Totals>("preview_settlement", { input }),
  saveSettlement: (input: SettlementWrite, confirmNegative: boolean) =>
    invoke<Aggregate>("save_settlement_record", { input, confirmNegative }),
  finalize: (id: string, expectedVersion: string, confirmNegative: boolean) =>
    invoke<Aggregate>("finalize_settlement_record", {
      id,
      expectedVersion,
      confirmNegative,
    }),
  payrolls: (filter: Filter) =>
    invoke<Row[]>("list_payroll_records", { filter }),
  saveDriver: (input: {
    id: string;
    expectedVersion: string;
    achCharge: string;
    twxCodeId: string | null;
    notes: string;
  }) => invoke<Row>("save_driver_payroll", { input }),
  saveOwner: (input: OwnerWrite) =>
    invoke<Row>("save_owner_payroll", { input }),
  history: (entity: "settlement" | "payroll", id: string) =>
    invoke<Row[]>("get_record_history", { entity, id }),
};
// Presentation only: manipulate digit strings, never convert financial values to Number.
export function decimal(
  integer: string | null | undefined,
  places: number,
): string {
  if (integer == null) return "";
  const negative = integer.startsWith("-");
  const digits = (negative ? integer.slice(1) : integer).padStart(
    places + 1,
    "0",
  );
  return `${negative ? "-" : ""}${digits.slice(0, -places)}.${digits.slice(-places)}`;
}
export function money(cents: string | null | undefined): string {
  const value = decimal(cents, 2);
  return value
    ? value.startsWith("-")
      ? `-$${value.slice(1)}`
      : `$${value}`
    : "—";
}
export function periodLabel(
  start: string | null | undefined,
  end: string | null | undefined,
): string {
  const display = (v: string | null | undefined) =>
    v ? `${v.slice(5, 7)}/${v.slice(8, 10)}/${v.slice(0, 4)}` : "—";
  return `${display(start)} – ${display(end)}`;
}
