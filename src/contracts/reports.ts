import { invoke } from "@tauri-apps/api/core";
import { decimal } from "./business";
export type ReportKind =
  | "settlement_detail"
  | "driver_payroll"
  | "owner_payroll"
  | "payroll_summary"
  | "settlement_history";
export type ExportFormat = "pdf" | "xlsx" | "csv" | "docx";
export interface ReportRequest {
  kind: ReportKind;
  settlementId: string | null;
  periodStart: string | null;
  periodEnd: string | null;
  driverId: string | null;
  ownerId: string | null;
  truckId: string | null;
}
export interface ReportCell {
  kind: "text" | "money" | "decimal" | "percentage";
  value: string;
  scale?: number;
}
export interface Report {
  kind: ReportKind;
  title: string;
  period: string;
  draft: boolean;
  currency: string;
  sections: { name: string; columns: string[]; rows: ReportCell[][] }[];
  sourceVersions: string[];
  defaultName: string;
}
export interface ExportResult {
  fileName: string;
  sizeBytes?: string;
  message: string;
}
export const reports = {
  preview: (request: ReportRequest) =>
    invoke<Report>("preview_report", { request }),
  export: (request: ReportRequest, format: ExportFormat) =>
    invoke<ExportResult | null>("export_report", { request, format }),
  saveAs: (request: ReportRequest, format: ExportFormat) =>
    invoke<ExportResult | null>("save_report_as", { request, format }),
  print: (request: ReportRequest) =>
    invoke<ExportResult>("preview_report_print", { request }),
};
export const reportRequest = (kind: ReportKind): ReportRequest => ({
  kind,
  settlementId: null,
  periodStart: null,
  periodEnd: null,
  driverId: null,
  ownerId: null,
  truckId: null,
});
export function reportCell(cell: ReportCell): string {
  if (cell.kind === "text") return cell.value;
  const raw = decimal(
    cell.value,
    cell.kind === "money"
      ? 2
      : cell.kind === "percentage"
        ? 4
        : (cell.scale ?? 0),
  );
  if (cell.kind === "percentage") return `${raw}%`;
  if (cell.kind !== "money") return raw;
  const negative = raw.startsWith("-");
  const [whole, fraction] = raw.replace(/^-/u, "").split(".");
  return `${negative ? "-" : ""}$${(whole ?? "0").replace(/\B(?=(\d{3})+(?!\d))/gu, ",")}.${fraction ?? "00"}`;
}
