// Explicitly isolated native WebView harness. Windows picker actions are performed separately.
import { chromium, expect } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";
import assert from "node:assert/strict";
import { readFile, writeFile, mkdir } from "node:fs/promises";
const browser = await chromium.connectOverCDP("http://127.0.0.1:9223");
const context = browser.contexts()[0];
const page = context.pages().find((p) => p.url().includes("tauri.localhost"));
assert.ok(page);
page.setDefaultTimeout(15000);
const invoke = (command, args = {}) =>
  page.evaluate(
    ({ command, args }) => window.__TAURI_INTERNALS__.invoke(command, args),
    { command, args },
  );
const info = await invoke("get_app_info");
assert.ok(
  info.databasePath.includes("com.twx.payroll.phase5-smoke"),
  "Refuse normal application data",
);
const output = "outputs/desktop-phase-5/native";
await mkdir(output, { recursive: true });
const ids = JSON.parse(await readFile("work/phase5-native-state.json", "utf8"));
const request = (kind, settlementId = null) => ({
  kind,
  settlementId,
  periodStart: "2026-10-01",
  periodEnd: "2026-10-19",
  driverId: null,
  ownerId: null,
  truckId: null,
});
const navigate = async (area) => {
  const button = page
    .getByRole("navigation")
    .getByRole("button", { name: area, exact: true });
  if (!(await button.isVisible()))
    await page.getByRole("button", { name: "Toggle navigation" }).click();
  await button.click();
  await page
    .getByRole("heading", { name: area, level: 1, exact: true })
    .waitFor();
};
const fingerprint = async () =>
  JSON.stringify({
    settlements: await invoke("list_settlement_records", { filter: {} }),
    payroll: await invoke("list_payroll_records", { filter: {} }),
    history: await invoke("get_record_history", {
      entity: "settlement",
      id: ids.settlement,
    }),
    backup: await invoke("get_backup_status"),
  });
try {
  const [mode = "verify", format = "pdf", name = "settlement"] =
    process.argv.slice(2);
  if (mode === "prepare") {
    const a = await invoke("get_settlement_record", { id: ids.settlement });
    const input = {
      id: null,
      expectedVersion: null,
      driverId: a.header.driver_id,
      truckId: a.header.truck_id,
      periodStart: "2026-10-01",
      periodEnd: "2026-10-19",
      notes: 'Synthetic draft, quotes "test"\nand Unicode José',
      payoutPercentage: "85",
      loads: a.loads.map((l) => ({
        id: null,
        driverId: l.driver_id,
        truckId: l.truck_id,
        plantId: l.plant_id,
        jobId: l.job_id,
        loadNumber: l.load_number,
        bolNumber: l.bol_number,
        poNumber: l.po_number,
        tons: l.tons_milli === "10000" ? "10" : "20",
        completionRate: l.completion_rate_scaled === "50000" ? "5" : "4",
        mileage: l.mileage,
      })),
      adjustments: a.adjustments.map((v) => ({
        id: null,
        adjustmentTypeId: v.adjustment_type_id,
        twxCodeId: v.twx_code_id,
        explanation: v.explanation,
        direction: v.direction,
        amount: v.direction === "DEDUCT" ? "162.50" : "2",
      })),
    };
    const draft = await invoke("save_settlement_record", {
      input,
      confirmNegative: true,
    });
    ids.draft = draft.header.id;
    const negative = await invoke("save_settlement_record", {
      input: { ...input, notes: "Synthetic negative payout" },
      confirmNegative: true,
    });
    await invoke("finalize_settlement_record", {
      id: negative.header.id,
      expectedVersion: negative.header.row_version,
      confirmNegative: true,
    });
    ids.negative = negative.header.id;
    await writeFile("work/phase5-native-state.json", JSON.stringify(ids));
    console.log(
      "PASS persisted draft and negative settlement fixtures through existing audited commands",
    );
  } else if (mode === "verify" || mode === "restart") {
    const before = await fingerprint();
    if (mode === "restart")
      assert.equal(
        before,
        await readFile("work/phase5-before-exports.json", "utf8"),
      );
    else await writeFile("work/phase5-before-exports.json", before);
    for (const [name, r] of [
      ["settlement", request("settlement_detail", ids.settlement)],
      ["draft", request("settlement_detail", ids.draft)],
      ["negative", request("settlement_detail", ids.negative)],
      ["driver", request("driver_payroll")],
      ["owner", request("owner_payroll")],
      ["summary", request("payroll_summary")],
      ["history", request("settlement_history")],
    ]) {
      const report = await invoke("preview_report", { request: r });
      await writeFile(
        `${output}/${name}.json`,
        JSON.stringify(report, null, 2),
      );
      if (name === "settlement")
        assert.ok(JSON.stringify(report).includes('"value":"10375"'));
      if (name === "negative")
        assert.ok(JSON.stringify(report).includes('"value":"-5000"'));
      if (name === "draft") assert.ok(report.draft);
      if (name === "owner") assert.ok(!JSON.stringify(report).includes("ACH"));
    }
    for (const args of [
      { request: { ...request("driver_payroll"), path: "C:\\unexpected" } },
      { request: request("driver_payroll"), format: "exe" },
    ])
      await assert.rejects(() =>
        invoke(args.format ? "export_report" : "preview_report", args),
      );
    for (const command of [
      "plugin:fs|read_file",
      "plugin:shell|execute",
      "plugin:sql|execute",
    ])
      await assert.rejects(() =>
        invoke(command, { path: "C:\\Windows", query: "SELECT 1" }),
      );
    await navigate("Reports");
    await page
      .getByRole("combobox", { name: "Report type", exact: true })
      .selectOption("payroll_summary");
    await page.getByRole("button", { name: "Preview Saved Report" }).click();
    await expect(page.locator(".report-preview")).toContainText("$103.75");
    const session = await context.newCDPSession(page);
    for (const [width, height] of [
      [1440, 900],
      [800, 600],
      [375, 700],
    ]) {
      await session.send("Emulation.setDeviceMetricsOverride", {
        width,
        height,
        deviceScaleFactor: 1,
        mobile: false,
      });
      for (const theme of ["light", "dark"]) {
        await page.evaluate(
          (v) => (document.documentElement.dataset.theme = v),
          theme,
        );
        assert.ok(
          await page.evaluate(
            () => document.documentElement.scrollWidth <= innerWidth,
          ),
        );
        const axe = await new AxeBuilder({ page })
          .withTags(["wcag2a", "wcag2aa", "wcag21aa"])
          .analyze();
        assert.deepEqual(
          axe.violations.map((v) => ({
            id: v.id,
            nodes: v.nodes.map((n) => n.failureSummary),
          })),
          [],
        );
      }
      await page.screenshot({
        path: `${output}/reports-${width}.png`,
        fullPage: true,
      });
    }
    await session.send("Emulation.clearDeviceMetricsOverride");
    assert.equal(await fingerprint(), before);
    console.log(
      `PASS ${mode}: seven canonical saved reports, negative/draft/owner semantics, IPC denials, responsive light/dark axe, unchanged financial/audit/backup state`,
    );
  } else {
    let r;
    if (["settlement", "draft", "negative"].includes(name))
      r = request("settlement_detail", ids[name]);
    else
      r = request(
        name === "driver"
          ? "driver_payroll"
          : name === "owner"
            ? "owner_payroll"
            : name === "history"
              ? "settlement_history"
              : "payroll_summary",
      );
    await navigate("Reports");
    await page
      .getByRole("combobox", { name: "Report type", exact: true })
      .selectOption(r.kind);
    await page.getByLabel("History from", { exact: true }).fill(r.periodStart);
    await page.getByLabel("History through", { exact: true }).fill(r.periodEnd);
    if (r.settlementId) {
      await page
        .getByRole("button", { name: "Find Saved Settlements" })
        .click();
      await expect(
        page
          .getByLabel("Saved settlement", { exact: true })
          .locator(`option[value="${r.settlementId}"]`),
      ).toHaveCount(1);
      await page
        .getByLabel("Saved settlement", { exact: true })
        .selectOption(r.settlementId);
    }
    if (name === "driver" || name === "history") {
      const refs = await invoke("get_reference_data");
      r.driverId = refs.drivers[0].id;
      r.truckId = refs.trucks[0].id;
      await page
        .getByLabel("Filter by driver", { exact: true })
        .selectOption(r.driverId);
      await page
        .getByLabel("Filter by truck", { exact: true })
        .selectOption(r.truckId);
    }
    await page.getByRole("button", { name: "Preview Saved Report" }).click();
    await expect(page.locator(".report-preview")).toBeVisible();
    const before = await fingerprint();
    await page
      .getByRole("combobox", { name: "Export format", exact: true })
      .selectOption(format);
    await page
      .getByRole("button", {
        name: mode === "print" ? "Print / Preview PDF" : "Export Saved Report",
      })
      .click();
    console.log(`WAIT native picker ${name}.${format}`);
    const actions = page.getByRole("region", { name: "Report export actions" });
    await expect(
      actions.locator('[role="status"], [role="alert"]'),
    ).toBeVisible({ timeout: 240000 });
    const message = await actions
      .locator('[role="status"], [role="alert"]')
      .innerText();
    if (mode === "cancel") assert.ok(message.includes("canceled"));
    else if (mode === "collision")
      assert.ok(message.includes("already exists"));
    else assert.ok(!(await actions.locator('[role="alert"]').count()), message);
    assert.equal(await fingerprint(), before);
    console.log(
      `PASS ${mode} ${name}.${format}: ${message}; database/audit/backup unchanged`,
    );
  }
} finally {
  await browser.close();
}
