// Run only against an explicitly isolated Tauri/WebView2 diagnostic build.
// The ordinary application config never exposes a debugging port.
import { chromium, expect } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";
import assert from "node:assert/strict";
import { mkdir, readFile, writeFile } from "node:fs/promises";
const browser = await chromium.connectOverCDP("http://127.0.0.1:9223");
const context = browser.contexts()[0];
const page = context.pages().find((p) => p.url().includes("tauri.localhost"));
assert.ok(page, "Native Tauri page must exist");
page.setDefaultTimeout(10000);
const invoke = (command, args = {}) =>
  page.evaluate(
    ({ command, args }) => window.__TAURI_INTERNALS__.invoke(command, args),
    { command, args },
  );
const info = await invoke("get_app_info");
assert.ok(
  info.databasePath.includes("com.twx.payroll.phase4-smoke"),
  "Refuse writes to the normal database",
);
assert.equal(info.schemaVersion, 4);
const output = "outputs/desktop-phase-4";
await mkdir(output, { recursive: true });
const report = {
  info: {
    ...info,
    databasePath: info.databasePath.replace(
      /Users\\[^\\]+/i,
      "Users\\[redacted]",
    ),
  },
  results: [],
};
const pass = (step) => {
  report.results.push(step);
  console.log(`PASS ${step}`);
};
const navigate = async (area) => {
  const toggle = page.getByRole("button", { name: "Toggle navigation" });
  const button = page
    .getByRole("navigation")
    .getByRole("button", { name: area, exact: true });
  if (!(await button.isVisible())) await toggle.click();
  await button.click();
  await page
    .getByRole("heading", { name: area, exact: true, level: 1 })
    .waitFor();
};
const fill = async (label, value, scope = page) =>
  scope.getByLabel(label, { exact: true }).fill(value);
const select = async (label, name, scope = page) =>
  scope.getByLabel(label, { exact: true }).selectOption({ label: name });
const saveReference = async () => {
  await page
    .getByRole("button", { name: "Save Reference", exact: true })
    .click();
  await expect(
    page.getByText(
      "Reference saved. Existing financial snapshots keep their saved labels.",
    ),
  ).toBeVisible();
};
const refs = () => invoke("get_reference_data");
try {
  if (process.argv.includes("--restart")) {
    const ids = JSON.parse(
      await readFile("work/phase4-native-state.json", "utf8"),
    );
    const list = await invoke("list_settlement_records", { filter: {} });
    assert.ok(list.some((r) => r.id === ids.settlement));
    let payroll = (await invoke("list_payroll_records", { filter: {} })).find(
      (r) => r.settlement_id === ids.settlement,
    );
    assert.equal(
      payroll.total_cents,
      process.argv.includes("--post-edit") ? "11225" : "10375",
    );
    assert.equal(payroll.notes, "Synthetic ACH note");
    assert.equal(payroll.twx_code_snapshot, "SYN");
    if (!process.argv.includes("--post-edit"))
      assert.equal((await invoke("get_preferences")).theme, "dark");
    pass("Restart preserved settlement, payroll, ACH, notes and SQLite theme");
    await navigate("Weekly Settlements / Loads");
    await page
      .getByRole("button", { name: /Synthetic Driver · Truck FAKE-001/ })
      .click();
    await expect(
      page.getByLabel("Mileage", { exact: true }).first(),
    ).toHaveValue("12.3456789");
    if (!process.argv.includes("--post-edit")) {
      await fill(
        "Completion Rate",
        "6",
        page.getByRole("group", { name: "Load 1", exact: true }),
      );
      await expect(page.getByTestId("final-payout")).toHaveText("$111.00");
      await page
        .getByRole("button", { name: "Save Finalized Edit", exact: true })
        .click();
      await expect(
        page.getByText(
          "Finalized edit saved. Payroll and audit history updated.",
        ),
      ).toBeVisible();
    }
    payroll = (await invoke("list_payroll_records", { filter: {} })).find(
      (r) => r.settlement_id === ids.settlement,
    );
    assert.equal(payroll.amount_cents, "11100");
    assert.equal(payroll.ach_charge_cents, "125");
    assert.equal(payroll.total_cents, "11225");
    const history = await invoke("get_record_history", {
      entity: "settlement",
      id: ids.settlement,
    });
    assert.ok(history.some((r) => r.category === "CORRECTION"));
    const before = history.find((r) => r.category === "CORRECTION");
    assert.ok(before.before_json.includes("50000"));
    assert.ok(before.after_json.includes("60000"));
    pass(
      "Finalized edit saved, linked payroll refreshed atomically and audit retained before/after facts",
    );
    await navigate("TWX Payroll");
    await page
      .getByRole("button", { name: "Synthetic Driver", exact: true })
      .click();
    await expect(
      page.getByLabel("Settlement Amount", { exact: true }),
    ).toHaveValue("$111.00");
    await expect(
      page.getByText("$112.25", { exact: true }).first(),
    ).toBeVisible();
    pass("Primary payroll displays updated driver Amount and Total");
    await assert.rejects(() =>
      invoke("save_driver_payroll", {
        input: {
          id: payroll.id,
          expectedVersion: payroll.row_version,
          achCharge: 1.25,
          twxCodeId: null,
          notes: "",
        },
      }),
    );
    await assert.rejects(() =>
      invoke("save_driver_payroll", {
        input: {
          id: payroll.id,
          expectedVersion: payroll.row_version,
          achCharge: "1.25",
          twxCodeId: null,
          notes: "",
          amount: "500",
        },
      }),
    );
    pass(
      "Native IPC rejects Number financial input and manual driver Amount injection",
    );
  } else {
    if (!process.argv.includes("--resume")) {
      assert.equal(
        (await refs()).owners.length,
        0,
        "Initial run needs an empty isolated database",
      );
      pass(
        "Native startup created schema 4 in isolated per-user application-local-data path",
      );
      await navigate("Owners");
      await page
        .getByRole("button", { name: "Add Owner", exact: true })
        .click();
      await fill("Name", "Synthetic Owner");
      await saveReference();
      pass("Created synthetic owner through native UI");
      await navigate("Drivers");
      await page
        .getByRole("button", { name: "Driver Types", exact: true })
        .click();
      await page
        .getByRole("button", { name: "Add Driver Type", exact: true })
        .click();
      await fill("Name", "Synthetic Type");
      await fill("Payout Percentage", "85");
      await saveReference();
      await page
        .getByRole("button", { name: "Drivers", exact: true })
        .last()
        .click();
      await page
        .getByRole("button", { name: "Add Driver", exact: true })
        .click();
      await fill("Name", "Synthetic Driver");
      await select("Driver Type", "Synthetic Type");
      await saveReference();
      pass("Created synthetic driver type at 85% and driver");
      await navigate("Trucks");
      await page
        .getByRole("button", { name: "Add Truck", exact: true })
        .click();
      await fill("Truck #", "FAKE-001");
      await select("Owner", "Synthetic Owner");
      await select("Default Driver", "Synthetic Driver");
      await saveReference();
      pass("Created truck linked to owner and default driver");
      await navigate("Settings");
      await page
        .getByRole("button", { name: "TWX Codes", exact: true })
        .click();
      await page
        .getByRole("button", { name: "Add TWX Code", exact: true })
        .click();
      await fill("TWX Code", "SYN");
      await fill("Code explanation", "Synthetic code");
      await saveReference();
      await page.getByRole("button", { name: "Plants", exact: true }).click();
      await page
        .getByRole("button", { name: "Add Plant", exact: true })
        .click();
      await fill("Name", "Synthetic Plant");
      await saveReference();
      await page.getByRole("button", { name: "Jobs", exact: true }).click();
      await page.getByRole("button", { name: "Add Job", exact: true }).click();
      await fill("Name", "Synthetic Job");
      await select("Plant", "Synthetic Plant");
      await saveReference();
      await page
        .getByRole("button", { name: "Adjustment Types", exact: true })
        .click();
      for (const name of ["Insurance", "Reimbursement"]) {
        await page
          .getByRole("button", { name: "Add Adjustment Type", exact: true })
          .click();
        await page
          .getByRole("button", { name: `Use ${name}`, exact: true })
          .click();
        await select("Default TWX Code", "SYN · Synthetic code");
        await saveReference();
      }
      pass(
        "Created synthetic plant/job/code and ADD/DEDUCT adjustment configuration",
      );
    }
    await navigate("Weekly Settlements / Loads");
    await page
      .getByRole("button", { name: "New Settlement", exact: true })
      .click();
    await fill("Pay-period start", "2026-10-01");
    await fill("Pay-period end", "2026-10-19");
    await select("Settlement Truck #", "FAKE-001");
    await expect(
      page.getByLabel("Settlement Driver", { exact: true }),
    ).toHaveValue((await refs()).drivers[0].id);
    await expect(
      page.getByLabel("Payout Percentage", { exact: true }),
    ).toHaveValue("85.0000");
    await fill("Settlement Notes", "Synthetic native workflow");
    pass(
      "Custom pay period and explicit driver/type/percentage created from truck default",
    );
    for (const [index, tons, rate] of [
      [1, "10", "5"],
      [2, "20", "4"],
    ]) {
      await page.getByRole("button", { name: "Add Load", exact: true }).click();
      const group = page.getByRole("group", {
        name: `Load ${index}`,
        exact: true,
      });
      await fill("Load ID", `FAKE-LOAD-${index}`, group);
      await fill("BOL Number", `FAKE-BOL-${index}`, group);
      await fill("PO Number", `FAKE-PO-${index}`, group);
      await select("Plant", "Synthetic Plant", group);
      await select("Job Name", "Synthetic Job", group);
      await fill("Net Weight / Tons", tons, group);
      await fill("Completion Rate", rate, group);
      await fill("Mileage", "12.3456789", group);
    }
    await expect(page.getByLabel("Load 1 Total", { exact: true })).toHaveText(
      "$50.00",
    );
    await expect(page.getByLabel("Load 2 Total", { exact: true })).toHaveText(
      "$80.00",
    );
    pass("Two exact loads entered; Rust returned $50/$80 load totals");
    for (const [index, type, amount] of [
      [1, "Insurance", "10"],
      [2, "Reimbursement", "2"],
    ]) {
      await page
        .getByRole("button", { name: "Add Adjustment", exact: true })
        .click();
      const group = page.getByRole("group", {
        name: `Adjustment ${index}`,
        exact: true,
      });
      await select("Adjustment Type", type, group);
      await fill("Explanation", `Synthetic ${type}`, group);
      await fill("Adjustment Amount", amount, group);
    }
    await expect(page.getByTestId("final-payout")).toHaveText("$102.50");
    pass("Rust summary: gross $130, driver gross $110.50, -$10+$2 = $102.50");
    await page.getByRole("button", { name: "Save Draft", exact: true }).click();
    await expect(page.getByText("Draft saved.", { exact: true })).toBeVisible();
    pass("Draft persisted with load rows, mileage, adjustments and ISO dates");
    await page
      .getByRole("button", { name: "Finalize Settlement", exact: true })
      .click();
    await expect(
      page.locator(".status-pill").filter({ hasText: "FINALIZED" }),
    ).toBeVisible();
    const settlement = (
      await invoke("list_settlement_records", { filter: {} })
    )[0].id;
    await writeFile(
      "work/phase4-native-state.json",
      JSON.stringify({ settlement, databasePath: info.databasePath }),
    );
    pass("Finalization created linked driver payroll and audit");
    await navigate("TWX Payroll");
    await page
      .getByRole("button", { name: "Synthetic Driver", exact: true })
      .click();
    await expect(
      page.getByLabel("Settlement Amount", { exact: true }),
    ).toHaveValue("$102.50");
    assert.equal(
      await page
        .getByLabel("Settlement Amount", { exact: true })
        .getAttribute("readonly"),
      "",
    );
    await fill("ACH Charge", "1.25");
    await select("TWX Code", "SYN · Synthetic code");
    await fill("Driver Payroll Notes", "Synthetic ACH note");
    await page
      .getByRole("button", { name: "Save Driver Payroll", exact: true })
      .click();
    await expect(
      page.getByText("$103.75", { exact: true }).first(),
    ).toBeVisible();
    pass(
      "Primary payroll readonly Amount; saved driver ACH/code/notes and $103.75 total",
    );
    await page
      .getByRole("button", { name: "New Owner Payroll", exact: true })
      .click();
    await select("Owner Name", "Synthetic Owner");
    await select("Owner Truck #", "FAKE-001");
    await fill("Owner Amount", "123.45");
    await fill("Owner Pay-period start", "2026-10-01");
    await fill("Owner Pay-period end", "2026-10-19");
    assert.equal(
      await page
        .getByRole("group", { name: "Owner payroll fields" })
        .getByLabel(/ACH/)
        .count(),
      0,
    );
    await page
      .getByRole("button", { name: "Save Owner Payroll", exact: true })
      .click();
    await expect(
      page.getByText("Manual owner payroll saved with audit history."),
    ).toBeVisible();
    pass("Manual owner payroll saved separately with no ACH control");
    await navigate("Settings");
    await page.getByRole("button", { name: "dark", exact: true }).click();
    assert.equal((await invoke("get_preferences")).theme, "dark");
    pass("Saved dark theme in real SQLite before restart");
  }
  for (const command of [
    "plugin:fs|read_file",
    "plugin:shell|execute",
    "plugin:sql|execute",
    "plugin:window|set_title",
  ]) {
    await assert.rejects(() =>
      invoke(command, { path: "C:/", query: "SELECT 1" }),
    );
  }
  pass("Filesystem, shell, arbitrary SQL and ungranted window IPC denied");
  if (process.argv.includes("--restart")) {
    const session = await context.newCDPSession(page);
    for (const [width, height, dpr] of [
      [1440, 900, 1],
      [1024, 768, 1.25],
      [800, 600, 1.5],
      [375, 700, 2],
    ]) {
      await session.send("Emulation.setDeviceMetricsOverride", {
        width,
        height,
        deviceScaleFactor: dpr,
        mobile: false,
      });
      for (const area of [
        "TWX Payroll",
        "Weekly Settlements / Loads",
        "Drivers",
        "Settings",
      ]) {
        await navigate(area);
        if (area === "TWX Payroll") {
          await page
            .getByRole("button", { name: "Synthetic Driver", exact: true })
            .click();
          await page
            .getByRole("button", { name: "New Owner Payroll", exact: true })
            .click();
          const input = page.getByLabel("Owner Amount", { exact: true });
          await input.focus();
          await page.keyboard.type("45.67");
          await expect(input).toHaveValue("45.67");
          await page.keyboard.press("Tab");
          await expect(
            page.getByLabel("Owner Pay-period start", { exact: true }),
          ).toBeFocused();
        }
        if (area === "Weekly Settlements / Loads")
          await page
            .getByRole("button", { name: /Synthetic Driver · Truck FAKE-001/ })
            .click();
        if (area === "Drivers")
          await page
            .getByRole("button", { name: "Add Driver", exact: true })
            .click();
        for (const theme of ["light", "dark"]) {
          await invoke("set_theme_preference", { theme });
          await page.evaluate(
            (v) => (document.documentElement.dataset.theme = v),
            theme,
          );
          assert.ok(
            await page.evaluate(
              () => document.documentElement.scrollWidth <= innerWidth,
            ),
            `Overflow ${area} ${width}`,
          );
          const result = await new AxeBuilder({ page })
            .withTags(["wcag2a", "wcag2aa", "wcag21aa"])
            .analyze();
          assert.deepEqual(
            result.violations.map((r) => ({
              id: r.id,
              nodes: r.nodes.map((n) => ({
                html: n.html,
                summary: n.failureSummary,
              })),
            })),
            [],
            `${area} ${width} ${theme}`,
          );
        }
        if (
          (width === 1440 || width === 375) &&
          area === "Weekly Settlements / Loads"
        )
          await page.screenshot({
            path: `${output}/settlement-${width}.png`,
            fullPage: true,
          });
      }
      pass(
        `Native WebView responsive overflow/accessibility and form keyboard input light/dark ${width}x${height} at emulated DPR ${dpr}`,
      );
    }
    await session.send("Emulation.clearDeviceMetricsOverride");
    await navigate("Weekly Settlements / Loads");
    await fill("History from", "2027-01-01");
    await expect(
      page.getByRole("button", { name: /Synthetic Driver · Truck FAKE-001/ }),
    ).toHaveCount(0);
    await fill("History from", "2026-10-01");
    await fill("History through", "2026-10-19");
    await select("Filter by driver", "Synthetic Driver");
    await select("Filter by truck", "FAKE-001");
    await expect(
      page.getByRole("button", { name: /Synthetic Driver · Truck FAKE-001/ }),
    ).toHaveCount(1);
    pass(
      "Native date/driver/truck history filtering finds the persisted custom period",
    );
  }
} catch (error) {
  report.failure = String(error);
  await page
    .screenshot({ path: `${output}/native-failure.png`, fullPage: true })
    .catch(() => {});
  throw error;
} finally {
  await writeFile(
    `${output}/native-${process.argv.includes("--restart") ? "restart" : "initial"}.json`,
    JSON.stringify(report, null, 2),
  );
  await browser.close();
}
