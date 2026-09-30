import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { DatabaseSync } from "node:sqlite";
const read = (path) => readFileSync(path, "utf8");
const config = JSON.parse(read("src-tauri/tauri.conf.json"));
const capability = JSON.parse(read("src-tauri/capabilities/main.json"));
assert.equal(config.identifier, "com.twx.payroll");
assert.equal(config.build.frontendDist, "../dist");
assert.equal(config.bundle.active, false);
assert.deepEqual(capability.windows, ["main"]);
assert.equal(capability.local, true);
assert.equal(capability.remote, undefined);
assert.deepEqual(capability.permissions, [
  "allow-get-app-info",
  "allow-get-preferences",
  "allow-set-theme-preference",
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
