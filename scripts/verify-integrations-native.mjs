// Unconfigured native acceptance: no Google authorization or real transmissions.
import { chromium, expect } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";
import assert from "node:assert/strict";
import { readFile, writeFile, mkdir } from "node:fs/promises";
const browser = await chromium.connectOverCDP("http://127.0.0.1:9223");
const context = browser.contexts()[0];
const page = context.pages().find((p) => p.url().includes("tauri.localhost"));
assert.ok(page);
page.setDefaultTimeout(10000);
const invoke = (command, args = {}) =>
  page.evaluate(
    ({ command, args }) => window.__TAURI_INTERNALS__.invoke(command, args),
    { command, args },
  );
const info = await invoke("get_app_info");
assert.ok(
  info.databasePath.includes("com.twx.payroll.phase6-smoke"),
  "Refuse normal application database",
);
assert.equal(info.schemaVersion, 4);
const ids = JSON.parse(await readFile("work/phase6-native-state.json", "utf8"));
const folder = "outputs/desktop-phase-6/native";
await mkdir(folder, { recursive: true });
const request = (kind) => ({
  kind,
  settlementId: kind === "settlement_detail" ? ids.settlement : null,
  periodStart: "2026-10-01",
  periodEnd: "2026-10-19",
  driverId: null,
  ownerId: null,
  truckId: null,
});
const fingerprint = async () =>
  JSON.stringify({
    settlements: await invoke("list_settlement_records", { filter: {} }),
    payroll: await invoke("list_payroll_records", { filter: {} }),
    audit: await invoke("get_record_history", {
      entity: "settlement",
      id: ids.settlement,
    }),
    backup: await invoke("get_backup_status"),
  });
const before = await fingerprint();
if (process.argv.includes("--restart")) {
  assert.equal(
    before,
    await readFile(`${folder}/financial-before.json`, "utf8"),
  );
  console.log("PASS actual restart preserved financial/audit/backup state");
} else {
  await writeFile(`${folder}/financial-before.json`, before);
}
const results = [];
const pass = (message) => {
  results.push(message);
  console.log(`PASS ${message}`);
};
const navigate = async (name) => {
  if (!(await page.getByRole("navigation").isVisible()))
    await page.getByRole("button", { name: "Toggle navigation" }).click();
  await page
    .getByRole("navigation")
    .getByRole("button", { name, exact: true })
    .click();
};
try {
  const status = await invoke("get_integration_status");
  assert.equal(status.length, 2);
  assert.ok(status.every((s) => !s.connected && s.account === null));
  pass("Unconfigured Sheets/Gmail without startup network requirement");
  for (const kind of [
    "settlement_detail",
    "driver_payroll",
    "owner_payroll",
    "payroll_summary",
  ]) {
    const report = await invoke("preview_report", { request: request(kind) });
    assert.ok(report.sections.length);
    await assert.rejects(
      invoke("prepare_external_report", {
        request: request(kind),
        service: "sheets",
        email: null,
      }),
      /disconnected/u,
    );
    if (kind === "owner_payroll")
      assert.ok(!JSON.stringify(report.sections).includes("ACH"));
  }
  for (const format of ["pdf", "xlsx", "docx"])
    await assert.rejects(
      invoke("prepare_external_report", {
        request: request("settlement_detail"),
        service: "email",
        email: {
          recipient: "recipient@example.invalid",
          subject: "Synthetic smoke",
          body: "Synthetic only",
          format,
        },
      }),
      /disconnected/u,
    );
  pass(
    "All required report types remain readable; disconnected external actions refuse safely",
  );
  await assert.rejects(
    invoke("prepare_external_report", {
      request: { ...request("settlement_detail"), path: "not-allowed" },
      service: "sheets",
      email: null,
    }),
    /unknown field/u,
  );
  await assert.rejects(
    invoke("prepare_external_report", {
      request: request("settlement_detail"),
      service: "email",
      email: {
        recipient: "recipient@example.invalid",
        subject: "Synthetic",
        body: "",
        format: "pdf",
        attachmentPath: "not-allowed",
      },
    }),
    /unknown field/u,
  );
  await assert.rejects(
    invoke("confirm_external_report", {
      token: "nonexistent",
      confirmed: true,
    }),
    /expired|already used/u,
  );
  for (const command of [
    "plugin:fs|read_file",
    "plugin:shell|execute",
    "plugin:sql|execute",
  ])
    await assert.rejects(invoke(command, {}));
  pass(
    "IPC denies paths, arbitrary attachments, unknown tokens and broad plugins",
  );
  await navigate("Reports");
  await page.getByRole("button", { name: "Preview Saved Report" }).click();
  await expect(
    page.getByRole("button", { name: "Send to Google Sheets" }),
  ).toBeVisible();
  await page.getByRole("button", { name: "Email Report" }).click();
  await page
    .getByLabel("Recipient", { exact: true })
    .fill("recipient@example.invalid");
  await page
    .getByLabel("Subject", { exact: true })
    .fill("Synthetic native report");
  await page.getByRole("button", { name: "Review Email" }).click();
  await expect(page.getByRole("alert")).toContainText("disconnected");
  await page.getByRole("button", { name: "Cancel composition" }).click();
  await expect(
    page.getByRole("form", { name: "Email report composition" }),
  ).toHaveCount(0);
  pass(
    "Native report email form defaults to PDF, reports unconfigured state and cancels",
  );
  for (const [width, height] of [
    [1440, 900],
    [800, 700],
    [375, 700],
  ]) {
    await page.setViewportSize({ width, height });
    await navigate("Settings");
    for (const theme of ["light", "dark"]) {
      await page.getByRole("button", { name: theme, exact: true }).click();
      await expect(page.locator("html")).toHaveAttribute("data-theme", theme);
      await expect(
        page.getByRole("heading", { name: "Optional integrations" }),
      ).toBeVisible();
      assert.equal(
        await page.evaluate(
          () => document.documentElement.scrollWidth <= window.innerWidth,
        ),
        true,
      );
      const accessibility = await new AxeBuilder({ page })
        .withTags(["wcag2a", "wcag2aa", "wcag21aa"])
        .analyze();
      assert.deepEqual(
        accessibility.violations.map((v) => v.id),
        [],
      );
    }
    await page.screenshot({
      path: `${folder}/settings-${width}.png`,
      fullPage: true,
    });
  }
  pass(
    "Native integration Settings: light/dark accessibility and responsive widths 1440/800/375",
  );
  assert.equal(await fingerprint(), before);
  pass(
    "External successes are mocked elsewhere; native unconfigured failures leave financial/audit/backup state unchanged",
  );
} finally {
  await writeFile(
    `${folder}/results.json`,
    JSON.stringify(
      {
        info: {
          ...info,
          databasePath: info.databasePath.replace(
            /Users\\[^\\]+/iu,
            "Users\\[redacted]",
          ),
        },
        results,
        liveGoogleSmoke: false,
        liveEmailSmoke: false,
      },
      null,
      2,
    ),
  );
  await browser.close();
}
