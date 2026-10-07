import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { DatabaseSync } from "node:sqlite";
const read = (path) => readFileSync(path, "utf8");
const config = JSON.parse(read("src-tauri/tauri.conf.json"));
const capability = JSON.parse(read("src-tauri/capabilities/main.json"));
assert.equal(config.identifier, "com.twx.payroll");
assert.equal(config.build.frontendDist, "../dist");
assert.equal(config.bundle.active, true);
assert.deepEqual(config.bundle.targets, ["nsis"]);
assert.equal(config.bundle.windows.nsis.installMode, "currentUser");
assert.equal(config.bundle.windows.webviewInstallMode.type, "offlineInstaller");
assert.equal(config.app.windows[0].additionalBrowserArgs, undefined);
assert.deepEqual(capability.windows, ["main"]);
assert.equal(capability.local, true);
assert.equal(capability.remote, undefined);
assert.deepEqual(capability.permissions, [
  "allow-get-app-info",
  "allow-get-preferences",
  "allow-set-theme-preference",
  "allow-get-backup-status",
  "allow-select-backup-folder",
  "allow-backup-now",
  "allow-set-automatic-backup",
  "allow-select-restore-backup",
  "allow-cancel-restore",
  "allow-restore-backup",
  "allow-get-reference-data",
  "allow-save-reference-record",
  "allow-list-settlement-records",
  "allow-get-settlement-record",
  "allow-preview-settlement",
  "allow-save-settlement-record",
  "allow-finalize-settlement-record",
  "allow-list-payroll-records",
  "allow-save-driver-payroll",
  "allow-save-owner-payroll",
  "allow-get-record-history",
  "allow-preview-report",
  "allow-export-report",
  "allow-preview-report-print",
  "allow-get-integration-status",
  "allow-connect-integration",
  "allow-cancel-integration-connection",
  "allow-disconnect-integration",
  "allow-prepare-external-report",
  "allow-confirm-external-report",
  "allow-cancel-external-report",
  "allow-save-report-as",
  "allow-get-export-settings",
  "allow-select-export-folder",
  "allow-reset-export-folder",
  "allow-open-export-folder",
]);
assert.equal(config.app.security.csp.includes("https:"), false);
assert.equal(read("tsconfig.json").includes("legacy"), false);
console.log(
  "PASS foundation build boundary, capabilities, local CSP, stable identifier",
);
const database = new DatabaseSync(":memory:");
try {
  database.exec("PRAGMA trusted_schema=OFF; PRAGMA foreign_keys=ON;");
  database.exec(read("src-tauri/migrations/001_settings.sql"));
  assert.deepEqual(
    database
      .prepare("SELECT name FROM sqlite_schema WHERE type='table'")
      .all()
      .map((row) => row.name),
    ["app_settings"],
  );
  assert.equal(
    database.prepare("SELECT value FROM app_settings WHERE key='theme'").get()
      .value,
    "system",
  );
  assert.throws(() => database.exec("UPDATE app_settings SET value='invalid'"));
  assert.throws(() =>
    database.exec(
      "INSERT INTO app_settings(key,value) VALUES('payroll','dark')",
    ),
  );
  database.exec("UPDATE app_settings SET value='dark'");
  assert.equal(
    database.prepare("SELECT value FROM app_settings WHERE key='theme'").get()
      .value,
    "dark",
  );
  console.log(
    "PASS actual Phase 1 SQL migration and settings constraints (Node SQLite, not Rust runner)",
  );
} finally {
  database.close();
}
