import { preview } from "vite";
import { chromium } from "@playwright/test";
import assert from "node:assert/strict";
import AxeBuilder from "@axe-core/playwright";
import { mkdir } from "node:fs/promises";
const server = await preview({
  preview: { host: "127.0.0.1", port: 4173, strictPort: true },
});
let browser;
try {
  browser = await chromium.launch({ channel: "msedge" });
  const context = await browser.newContext();
  const page = await context.newPage();
  for (const [width, height] of [
    [1440, 900],
    [1024, 768],
    [800, 600],
    [375, 700],
  ]) {
    await page.setViewportSize({ width, height });
    await page.goto("http://127.0.0.1:4173");
    await page
      .getByRole("heading", { name: "TWX Payroll", exact: true })
      .waitFor();
    for (const area of [
      "Weekly Settlements / Loads",
      "Drivers",
      "Owners",
      "Trucks",
      "Reports",
      "Reference Records",
      "User Manual",
      "TWX Payroll",
    ]) {
      if (width <= 720)
        await page.getByRole("button", { name: "Toggle navigation" }).click();
      await page.getByRole("button", { name: area, exact: true }).click();
      await page.getByRole("heading", { name: area, exact: true }).waitFor();
    }
    if (width <= 720)
      await page.getByRole("button", { name: "Toggle navigation" }).click();
    await page.getByRole("button", { name: "Settings", exact: true }).click();
    await page.getByRole("button", { name: "dark" }).click();
    assert.equal(await page.locator("html").getAttribute("data-theme"), "dark");
    assert.equal(
      await page.evaluate(
        () => document.documentElement.scrollWidth <= window.innerWidth,
      ),
      true,
      `Overflow at ${width}`,
    );
    const accessibility = await new AxeBuilder({ page })
      .withTags(["wcag2a", "wcag2aa", "wcag21aa"])
      .analyze();
    assert.deepEqual(
      accessibility.violations.map((issue) => ({
        id: issue.id,
        nodes: issue.nodes.length,
      })),
      [],
      `Accessibility at ${width}`,
    );
    if (width === 1440) {
      await mkdir("outputs/desktop-phase-1", { recursive: true });
      await page.screenshot({
        path: "outputs/desktop-phase-1/settings-dark.png",
      });
    }
    await page.getByRole("button", { name: "light" }).click();
    assert.equal(
      await page.locator("html").getAttribute("data-theme"),
      "light",
    );
    const lightAccessibility = await new AxeBuilder({ page })
      .withTags(["wcag2a", "wcag2aa", "wcag21aa"])
      .analyze();
    if (lightAccessibility.violations.length)
      console.log(
        JSON.stringify(
          lightAccessibility.violations.map((issue) =>
            issue.nodes.map((node) => ({
              html: node.html,
              summary: node.failureSummary,
            })),
          ),
        ),
      );
    assert.deepEqual(
      lightAccessibility.violations.map((issue) => issue.id),
      [],
      `Light accessibility at ${width}`,
    );
    await page.getByRole("button", { name: "system", exact: true }).focus();
    await page.keyboard.press("Enter");
    await page.emulateMedia({ colorScheme: "dark" });
    await page.waitForFunction(
      () => document.documentElement.dataset.theme === "dark",
    );
    await page.emulateMedia({ colorScheme: "light" });
    await page.waitForFunction(
      () => document.documentElement.dataset.theme === "light",
    );
    await page.getByRole("button", { name: "dark", exact: true }).click();
    await page.emulateMedia({ colorScheme: "light" });
    assert.equal(await page.locator("html").getAttribute("data-theme"), "dark");
    await page.getByRole("button", { name: "light", exact: true }).click();
    if (width === 375)
      await page.screenshot({
        path: "outputs/desktop-phase-1/settings-compact.png",
      });
    // Manual content is bundled: block all subsequent network access after startup.
    await page.route("**/*", (route) => route.abort());
    for (const theme of ["light", "dark"]) {
      if (width <= 720)
        await page.getByRole("button", { name: "Toggle navigation" }).click();
      await page.getByRole("button", { name: "Settings", exact: true }).click();
      await page.getByRole("button", { name: theme, exact: true }).click();
      if (width <= 720)
        await page.getByRole("button", { name: "Toggle navigation" }).click();
      await page
        .getByRole("button", { name: "User Manual", exact: true })
        .click();
      await page
        .getByRole("link", { name: "Reports and Exports", exact: true })
        .focus();
      await page.keyboard.press("Enter");
      await page
        .getByRole("heading", { name: "Reports and Exports", exact: true })
        .waitFor();
      assert.equal(
        await page.evaluate(
          () => document.documentElement.scrollWidth <= innerWidth,
        ),
        true,
        `Manual overflow ${width}`,
      );
      assert.deepEqual(
        (
          await new AxeBuilder({ page })
            .withTags(["wcag2a", "wcag2aa", "wcag21aa"])
            .analyze()
        ).violations.map((v) => v.id),
        [],
        `Manual ${theme} ${width}`,
      );
    }
    await page.unroute("**/*");
    console.log(`PASS browser layout/navigation/theme ${width}x${height}`);
  }
} finally {
  if (browser) await browser.close();
  await new Promise((resolve, reject) =>
    server.httpServer.close((error) => (error ? reject(error) : resolve())),
  );
}
