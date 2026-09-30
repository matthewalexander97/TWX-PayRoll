import { chromium } from "@playwright/test";
import assert from "node:assert/strict";
import { writeFileSync } from "node:fs";
import http from "node:http";
const browser = await chromium.connectOverCDP("http://127.0.0.1:9223");
const page = browser
  .contexts()
  .flatMap((c) => c.pages())
  .find((p) => p.url().startsWith("http://127.0.0.1:1420"));
assert(page, "Expected actual Tauri dev WebView");
const result = await page.evaluate(async () => {
  const invoke = window.__TAURI_INTERNALS__.invoke;
  const info = await invoke("get_app_info");
  const preferences = await invoke("get_preferences");
  const denied = [];
  for (const [command, args] of [
    ["plugin:window|set_title", { label: "main", title: "Unauthorized title" }],
    ["plugin:window|close", { label: "main" }],
    ["plugin:path|resolve", { paths: ["C:/Windows"] }],
    ["plugin:fs|read_text_file", { path: "C:/Windows/win.ini" }],
    ["plugin:shell|execute", { program: "cmd.exe", args: [] }],
    [
      "plugin:sql|select",
      { db: "sqlite:test.db", query: "SELECT 1", values: [] },
    ],
    ["set_theme_preference", { theme: "invalid" }],
  ]) {
    try {
      await invoke(command, args);
      denied.push({ command, rejected: false });
    } catch (error) {
      denied.push({ command, rejected: true, error: String(error) });
    }
  }
  return {
    info,
    preferences,
    denied,
    metrics: {
      width: innerWidth,
      height: innerHeight,
      dpr: devicePixelRatio,
      theme: document.documentElement.dataset.theme,
      overflow: document.documentElement.scrollWidth > innerWidth,
    },
  };
});
assert.equal(result.info.sqliteVersion, "3.53.2");
assert.equal(result.info.schemaVersion, 1);
const expectedTheme = process.env.TWX_EXPECTED_THEME ?? "dark";
assert.equal(result.preferences.theme, expectedTheme);
assert.equal(result.metrics.theme, expectedTheme);
for (const check of result.denied)
  assert.equal(check.rejected, true, check.command);
assert.match(result.denied[0].error, /not allowed|denied/i);
assert.match(result.denied[1].error, /not allowed|denied/i);
assert.match(result.denied[2].error, /not allowed|denied/i);
result.info.databasePath = result.info.databasePath.replace(
  /Users\\[^\\]+\\/,
  "Users\\[redacted]\\",
);
const server = http.createServer((req, res) => {
  res.setHeader("Content-Type", "text/html");
  res.end(
    "<!doctype html><title>Local IPC origin probe</title><p>Native acceptance origin probe</p>",
  );
});
await new Promise((resolve) => server.listen(4174, "127.0.0.1", resolve));
try {
  await page.goto("http://127.0.0.1:4174");
  result.remote = await page.evaluate(async () => {
    try {
      return {
        rejected: false,
        value: await window.__TAURI_INTERNALS__.invoke("get_preferences"),
      };
    } catch (error) {
      return { rejected: true, error: String(error) };
    }
  });
  assert.equal(result.remote.rejected, true);
  assert.match(result.remote.error, /not allowed|denied/i);
} finally {
  await page.goto("http://127.0.0.1:1420");
  await page.waitForFunction(() =>
    Boolean(document.documentElement.dataset.theme),
  );
  server.closeAllConnections();
  await new Promise((resolve) => server.close(resolve));
  await browser.close();
}
writeFileSync(
  "outputs/desktop-phase-1/native-runtime-results.json",
  JSON.stringify(result, null, 2),
);
console.log(JSON.stringify(result, null, 2));
