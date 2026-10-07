import { invoke } from "@tauri-apps/api/core";
export interface ExportPreference {
  directory: string;
  usesDocuments: boolean;
}
export const exportSettings = {
  status: () => invoke<ExportPreference>("get_export_settings"),
  select: () => invoke<ExportPreference | null>("select_export_folder"),
  reset: () => invoke<ExportPreference>("reset_export_folder"),
  open: () => invoke<void>("open_export_folder"),
};
