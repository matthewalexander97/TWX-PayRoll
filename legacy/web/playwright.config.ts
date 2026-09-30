import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./tests/e2e",
  fullyParallel: false,
  workers: 1,
  timeout: 30_000,
  reporter: [["list"], ["html", { open: "never" }]],
  use: { baseURL: "http://127.0.0.1:3100", channel: process.env.PLAYWRIGHT_CHANNEL ?? "msedge", trace: "retain-on-failure" },
  globalSetup: "./tests/support/global-setup.ts",
});
