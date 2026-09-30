import { chromium } from "@playwright/test";
import assert from "node:assert/strict";
import { writeFileSync } from "node:fs";
const browser = await chromium.connectOverCDP("http://127.0.0.1:9223");
const page = browser
  .contexts()
  .flatMap((c) => c.pages())
  .find((p) => p.url().includes("1420"));
const session = await page.context().newCDPSession(page);
const results = [];
try {
  for (const [width, height, dpr] of [
    [1280, 850, 1.25],
    [1024, 768, 1.5],
    [800, 600, 2],
    [375, 700, 2],
  ]) {
    await session.send("Emulation.setDeviceMetricsOverride", {
      width,
      height,
      deviceScaleFactor: dpr,
      mobile: false,
    });
    const metrics = await page.evaluate(() => ({
      width: innerWidth,
      height: innerHeight,
      dpr: devicePixelRatio,
      overflow: document.documentElement.scrollWidth > innerWidth,
      heading: document.querySelector("h1")?.textContent,
      theme: document.documentElement.dataset.theme,
      errors: [...document.querySelectorAll("[role=alert]")].map(
        (e) => e.textContent,
      ),
    }));
    assert.equal(metrics.overflow, false);
    assert.deepEqual(metrics.errors, []);
    assert.equal(metrics.theme, process.env.TWX_EXPECTED_THEME ?? "dark");
    results.push(metrics);
  }
} finally {
  await session.send("Emulation.clearDeviceMetricsOverride");
  await browser.close();
}
writeFileSync(
  "outputs/desktop-phase-1/native-scale-results.json",
  JSON.stringify(
    {
      method:
        "Actual Windows Tauri WebView with CDP-emulated CSS viewport/DPI; OS display settings unchanged",
      results,
    },
    null,
    2,
  ),
);
console.log(JSON.stringify(results, null, 2));
