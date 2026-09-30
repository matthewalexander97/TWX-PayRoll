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
