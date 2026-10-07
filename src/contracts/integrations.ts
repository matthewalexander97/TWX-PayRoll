import { invoke } from "@tauri-apps/api/core";
import type { ExportFormat, ReportRequest } from "./reports";
export type IntegrationService = "sheets" | "email";
export interface IntegrationStatus {
  service: IntegrationService;
  connected: boolean;
  account: string | null;
  message: string;
}
export interface EmailDetails {
  recipient: string;
  subject: string;
  body: string;
  format: ExportFormat;
}
export interface PreparedExternal {
  token: string;
  service: IntegrationService;
  title: string;
  period: string;
  draft: boolean;
  account: string;
  recipient: string | null;
  subject: string | null;
  fileName: string | null;
  format: ExportFormat | null;
}
export interface IntegrationReceipt {
  message: string;
  destination: string | null;
}
export const integrations = {
  status: () => invoke<IntegrationStatus[]>("get_integration_status"),
  connect: (service: IntegrationService) =>
    invoke<IntegrationStatus[] | null>("connect_integration", { service }),
  cancelConnection: () => invoke<void>("cancel_integration_connection"),
  disconnect: (service: IntegrationService) =>
    invoke<IntegrationReceipt>("disconnect_integration", { service }),
  prepare: (
    request: ReportRequest,
    service: IntegrationService,
    email: EmailDetails | null,
  ) =>
    invoke<PreparedExternal>("prepare_external_report", {
      request,
      service,
      email,
    }),
  confirm: (token: string) =>
    invoke<IntegrationReceipt>("confirm_external_report", {
      token,
      confirmed: true,
    }),
  cancel: (token: string) => invoke<void>("cancel_external_report", { token }),
};
export const integrationError = (error: unknown) =>
  typeof error === "string"
    ? error
    : "Integration unavailable. Local reports and exports remain available.";
