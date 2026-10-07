// Isolated WebView diagnostic only. Never run financial fixtures against production data.
import { chromium } from "@playwright/test";
import assert from "node:assert/strict";
import AxeBuilder from "@axe-core/playwright";
import { readFile, writeFile, mkdir } from "node:fs/promises";
const browser = await chromium.connectOverCDP("http://127.0.0.1:9223");
const page = browser
  .contexts()[0]
  .pages()
  .find((p) => p.url().includes("tauri.localhost"));
assert.ok(page);
const invoke = (command, args = {}) =>
  page.evaluate(
    ({ command, args }) => window.__TAURI_INTERNALS__.invoke(command, args),
    { command, args },
  );
const info = await invoke("get_app_info");
assert.ok(info.databasePath.includes("com.twx.payroll.phase7-smoke"));
const ids = JSON.parse(await readFile("work/phase7-native-state.json", "utf8"));
const request = {
  kind: "settlement_detail",
  settlementId: ids.settlement,
  periodStart: null,
  periodEnd: null,
  driverId: null,
  ownerId: null,
  truckId: null,
};
const fingerprint = async () =>
  JSON.stringify({
    settlements: await invoke("list_settlement_records", { filter: {} }),
    payroll: await invoke("list_payroll_records", { filter: {} }),
    history: await invoke("get_record_history", {
      entity: "settlement",
      id: ids.settlement,
    }),
  });
const mode = process.argv[2] ?? "verify";
await mkdir("outputs/desktop-phase-7/native", { recursive: true });
try {
  if (["folder", "backup-folder", "restore-select", "save-as"].includes(mode)) {
    const command = {
      folder: "select_export_folder",
      "backup-folder": "select_backup_folder",
      "restore-select": "select_restore_backup",
      "save-as": "save_report_as",
    }[mode];
    console.log(`NATIVE PICKER ${command}`);
    const result = await invoke(
      command,
      mode === "save-as" ? { request, format: "pdf" } : {},
    );
    await writeFile(
      `work/phase7-${mode}-result.json`,
      JSON.stringify(result, null, 2),
    );
    console.log(
      "PASS native picker completed",
      result === null ? "canceled" : "selected",
    );
  } else if (mode === "manual") {
    const original = await invoke("get_preferences");
    for (const theme of ["light", "dark"]) {
      await invoke("set_theme_preference", { theme });
      await page.reload();
      for (const width of [1440, 800, 375]) {
        await page.setViewportSize({ width, height: 900 });
        const button = page.getByRole("button", {
          name: "User Manual",
          exact: true,
        });
        if (!(await button.isVisible()))
          await page.getByRole("button", { name: "Toggle navigation" }).click();
        await button.click();
        assert.equal(
          await page
            .getByRole("navigation", { name: "Manual sections" })
            .getByRole("link")
            .count(),
          14,
        );
        await page
          .getByRole("link", { name: "Reports and Exports", exact: true })
          .focus();
        await page.keyboard.press("Enter");
        assert.equal(
          await page.evaluate(
            () => document.documentElement.scrollWidth <= innerWidth,
          ),
          true,
        );
        assert.deepEqual(
          (
            await new AxeBuilder({ page })
              .withTags(["wcag2a", "wcag2aa", "wcag21aa"])
              .analyze()
          ).violations.map((v) => v.id),
          [],
        );
        await page.screenshot({
          path: `outputs/desktop-phase-7/native/manual-${theme}-${width}.png`,
        });
      }
    }
    await invoke("set_theme_preference", { theme: original.theme });
    await page.setViewportSize({ width: 1280, height: 850 });
    await page.reload();
    console.log(
      "PASS native manual sections, keyboard, light/dark axe and 1440/800/375 layout",
    );
  } else if (mode === "verify") {
    const before = await fingerprint();
    const backup = await invoke("get_backup_status");
    const folder = await invoke("get_export_settings");
    assert.equal(folder.usesDocuments, false);
    assert.match(folder.directory, /phase7-exports/);
    for (const format of ["pdf", "xlsx", "csv", "docx"]) {
      const result = await invoke("export_report", { request, format });
      assert.ok(result.fileName.endsWith(`.${format}`));
      console.log(`PASS ${format} configured export ${result.fileName}`);
    }
    const second = await invoke("export_report", { request, format: "pdf" });
    assert.match(second.fileName, /-2\.pdf$/);
    console.log("PASS numbered collision");
    await writeFile(
      "work/phase7-before-restart.json",
      JSON.stringify(
        { financial: before, folder, theme: await invoke("get_preferences") },
        null,
        2,
      ),
    );
    assert.equal(before, await fingerprint());
    assert.deepEqual(backup, await invoke("get_backup_status"));
    for (const command of [
      "plugin:fs|read_text_file",
      "plugin:shell|execute",
      "plugin:sql|execute",
    ])
      await assert.rejects(
        invoke(command, { path: "private", sql: "SELECT 1", program: "cmd" }),
      );
    console.log(
      "PASS reports/settings leave saved financial/audit/backup state unchanged; broad IPC denied",
    );
  } else if (mode === "restart") {
    const old = JSON.parse(
      await readFile("work/phase7-before-restart.json", "utf8"),
    );
    assert.equal(old.financial, await fingerprint());
    assert.deepEqual(old.folder, await invoke("get_export_settings"));
    assert.deepEqual(old.theme, await invoke("get_preferences"));
    console.log(
      "PASS actual restart: financial history, export preference and SQLite theme retained",
    );
  } else if (mode === "backup") {
    const before = await fingerprint();
    const result = await invoke("backup_now");
    assert.ok(
      result.runs.some((r) => r.kind === "MANUAL" && r.status === "SUCCESS"),
    );
    assert.equal(before, await fingerprint());
    await writeFile(
      "work/phase7-backup-status.json",
      JSON.stringify(result, null, 2),
    );
    console.log("PASS manual validated backup");
  } else if (mode === "restore") {
    const preview = JSON.parse(
      await readFile("work/phase7-restore-select-result.json", "utf8"),
    );
    const before = await fingerprint();
    const result = await invoke("restore_backup", {
      token: preview.token,
      confirmed: true,
    });
    assert.equal(before, await fingerprint());
    assert.ok(
      result.runs.some(
        (r) => r.kind === "PRE_RESTORE" && r.status === "SUCCESS",
      ),
    );
    console.log(
      "PASS restore and pre-restore safety copy preserve financial history",
    );
  } else if (mode === "reset") {
    const before = await fingerprint();
    const backup = await invoke("get_backup_status");
    assert.equal((await invoke("reset_export_folder")).usesDocuments, true);
    assert.equal(before, await fingerprint());
    assert.deepEqual(backup, await invoke("get_backup_status"));
    console.log("PASS Documents reset, financial/backup independence");
  } else if (mode === "missing") {
    const before = await fingerprint();
    await assert.rejects(
      invoke("export_report", { request, format: "pdf" }),
      /unavailable/,
    );
    assert.equal(before, await fingerprint());
    console.log("PASS missing folder fails with no fallback");
  } else if (mode === "open") {
    await invoke("open_export_folder");
    console.log("PASS Rust opens configured folder without frontend path");
  } else throw Error(`Unknown mode ${mode}`);
} finally {
  await browser.close();
}
