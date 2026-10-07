import { invoke, isTauri } from "@tauri-apps/api/core";
export type Theme = "system" | "light" | "dark";
export interface Preferences {
  theme: Theme;
}
export interface AppInfo {
  version: string;
  databasePath: string;
  schemaVersion: number;
  sqliteVersion: string;
}
export const desktop = {
  available: isTauri,
  preferences: () => invoke<Preferences>("get_preferences"),
  saveTheme: (theme: Theme) =>
    invoke<Preferences>("set_theme_preference", { theme }),
  info: () => invoke<AppInfo>("get_app_info"),
};

export interface BackupRun {
  id: string;
  startedAt: number;
  createdAt: string;
  kind: string;
  path: string;
  status: string;
  schemaVersion: number | null;
  sizeBytes: string | null;
  sha256: string | null;
  validation: string;
  errorSummary: string | null;
}
export interface BackupStatus {
  destination: string | null;
  automaticEnabled: boolean;
  retentionCount: number;
  notice: string;
  lastSuccessfulBackup: string | null;
  runs: BackupRun[];
}
export interface RestorePreview {
  token: string;
  filename: string;
  schemaVersion: number;
  createdAt: string | null;
}
export const recovery = {
  status: () => invoke<BackupStatus>("get_backup_status"),
  selectFolder: () => invoke<BackupStatus | null>("select_backup_folder"),
  backup: () => invoke<BackupStatus>("backup_now"),
  automatic: (enabled: boolean) =>
    invoke<BackupStatus>("set_automatic_backup", { enabled }),
  selectRestore: () => invoke<RestorePreview | null>("select_restore_backup"),
  cancelRestore: () => invoke<void>("cancel_restore"),
  restore: (token: string) =>
    invoke<BackupStatus>("restore_backup", { token, confirmed: true }),
};
