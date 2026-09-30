import { test, expect, type Page } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";

async function login(page: Page) {
  await page.goto("/login");
  await page.getByLabel("Email address").fill("bookkeeper@example.test");
  await page.getByLabel("Password", { exact: true }).fill("test-password-only");
  await page.getByRole("button", { name: "Sign in to your workspace" }).click();
  await expect(page).toHaveURL(/\/payroll$/);
}

test("unauthenticated direct access to every workspace route fails closed", async ({ page }) => {
  for (const route of ["/", "/payroll", "/settlements", "/directory"]) {
    await page.goto(route);
    await expect(page).toHaveURL(/\/login$/);
    await expect(page.getByRole("heading", { name: "Sign in", exact: true })).toBeVisible();
    await expect(page.getByRole("navigation", { name: "Main navigation" })).toHaveCount(0);
  }
});

test("login, page navigation, current-page indicators, and logout work through SSR cookies", async ({ page }) => {
  await login(page);
  await expect(page.getByRole("heading", { name: "Driver payroll", exact: true })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Owner payroll", exact: true })).toBeVisible();
  await expect(page.getByRole("link", { name: "TWX Payroll", exact: true })).toHaveAttribute("aria-current", "page");
  await page.reload();
  await expect(page).toHaveURL(/\/payroll$/);
  await page.getByRole("link", { name: "Weekly settlements", exact: true }).click();
  await expect(page.getByRole("heading", { name: "Weekly Load / Settlement Detail" })).toBeVisible();
  await page.getByRole("link", { name: "Directory", exact: true }).click();
  await expect(page.getByRole("heading", { name: "Your directory starts here" })).toBeVisible();
  await page.getByRole("button", { name: "Sign out", exact: true }).click();
  await expect(page).toHaveURL(/\/login$/);
  await page.goto("/payroll");
  await expect(page).toHaveURL(/\/login$/);
});

test("invalid credentials and provider failures display an error and preserve entered email", async ({ page }) => {
  await page.goto("/login");
  await page.getByLabel("Email address").fill("unknown@example.test");
  await page.getByLabel("Password", { exact: true }).fill("wrong-password");
  await page.getByRole("button", { name: "Sign in to your workspace" }).click();
  await expect(page.locator("#login-error")).toHaveAttribute("role", "alert");
  await expect(page.locator("#login-error")).toContainText("Unable to sign in");
  await expect(page.getByLabel("Email address")).toHaveValue("unknown@example.test");
  await page.getByLabel("Email address").fill("unavailable@example.test");
  await page.getByLabel("Password", { exact: true }).fill("test-password-only");
  await page.getByRole("button", { name: "Sign in to your workspace" }).click();
  await expect(page.locator("#login-error")).toContainText("Unable to sign in");
  await expect(page.getByLabel("Email address")).toHaveValue("unavailable@example.test");
});

test("revoked sessions cannot use a cached cookie to re-enter the workspace", async ({ page, request }) => {
  await login(page);
  await request.post("http://127.0.0.1:54321/test/revoke-all");
  await page.goto("/directory");
  await expect(page).toHaveURL(/\/login$/);
});

test("a forged session cookie never grants access", async ({ page, context }) => {
  await context.addCookies([{ name: "sb-127-auth-token", value: "base64-" + Buffer.from(JSON.stringify({ access_token: "forged.jwt.token", refresh_token: "forged", user: { email: "administrator@example.test" } })).toString("base64url"), domain: "127.0.0.1", path: "/" }]);
  await page.goto("/payroll");
  await expect(page).toHaveURL(/\/login$/);
});

test("external post-login destinations are rejected", async ({ page }) => {
  await page.goto("/login?next=https://evil.example.test");
  await page.getByLabel("Email address").fill("bookkeeper@example.test");
  await page.getByLabel("Password", { exact: true }).fill("test-password-only");
  await page.getByRole("button", { name: "Sign in to your workspace" }).click();
  await expect(page).toHaveURL(/\/payroll$/);
});

test("missing project configuration shows setup guidance and no sign-in bypass", async ({ page }) => {
  await page.goto("http://127.0.0.1:3101/payroll");
  await expect(page).toHaveURL("http://127.0.0.1:3101/login");
  await expect(page.getByRole("heading", { name: "Workspace setup needed" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Sign in to your workspace" })).toHaveCount(0);
});

test("system theme works before hydration and manual theme survives reload and OS changes", async ({ page }) => {
  await page.emulateMedia({ colorScheme: "dark" });
  await page.goto("/login");
  await expect(page.locator("html")).toHaveAttribute("data-theme", "dark");
  await page.getByRole("button", { name: "Switch to light mode" }).click();
  await expect(page.locator("html")).toHaveAttribute("data-theme", "light");
  await page.reload();
  await expect(page.locator("html")).toHaveAttribute("data-theme", "light");
  await page.emulateMedia({ colorScheme: "light" });
  await page.emulateMedia({ colorScheme: "dark" });
  await expect(page.locator("html")).toHaveAttribute("data-theme", "light");
  // The inline script resolves the stored preference even when hydration is disabled.
  await page.route("**/_next/static/**/*.js", (route) => route.abort());
  await page.reload({ waitUntil: "domcontentloaded" });
  await expect(page.locator("html")).toHaveAttribute("data-theme", "light");
});

test("theme controls remain usable when browser storage is unavailable", async ({ page }) => {
  await page.addInitScript(() => {
    Storage.prototype.getItem = () => { throw new Error("storage disabled"); };
    Storage.prototype.setItem = () => { throw new Error("storage disabled"); };
  });
  await page.goto("/login");
  await page.getByRole("button", { name: "Switch to dark mode" }).click();
  await expect(page.locator("html")).toHaveAttribute("data-theme", "dark");
});

for (const width of [375, 768, 1280]) {
  test(`responsive shell, keyboard access, and light/dark accessibility at ${width}px`, async ({ page }) => {
    const errors: string[] = [];
    page.on("pageerror", (error) => errors.push(error.message));
    await page.setViewportSize({ width, height: 900 });
    await page.goto("/login");
    await page.keyboard.press("Tab");
    await expect(page.getByRole("link", { name: "Skip to main content" })).toBeFocused();
    await page.keyboard.press("Enter");
    await login(page);
    for (const mode of ["light", "dark"] as const) {
      if ((await page.locator("html").getAttribute("data-theme")) !== mode) await page.getByRole("button", { name: `Switch to ${mode} mode` }).click();
      expect((await new AxeBuilder({ page }).withTags(["wcag2a", "wcag2aa", "wcag21aa"]).analyze()).violations).toEqual([]);
      expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth)).toBe(true);
      const bounds = await page.getByRole("button", { name: "Sign out", exact: true }).boundingBox();
      expect(bounds?.height).toBeGreaterThanOrEqual(44);
    }
    expect(errors).toEqual([]);
    if (width === 1280) await page.screenshot({ path: "outputs/phase-1-desktop.png", fullPage: true });
    if (width === 375) await page.screenshot({ path: "outputs/phase-1-mobile.png", fullPage: true });
  });
}

test("login form has accessible controls and errors in both themes", async ({ page }) => {
  await page.goto("/login");
  await page.getByLabel("Email address").fill("wrong@example.test");
  await page.getByLabel("Password", { exact: true }).fill("wrong");
  await page.getByRole("button", { name: "Sign in to your workspace" }).click();
  await expect(page.locator("#login-error")).toBeVisible();
  for (const mode of ["light", "dark"] as const) {
    if ((await page.locator("html").getAttribute("data-theme")) !== mode) await page.getByRole("button", { name: `Switch to ${mode} mode` }).click();
    expect((await new AxeBuilder({ page }).withTags(["wcag2a", "wcag2aa", "wcag21aa"]).analyze()).violations).toEqual([]);
  }
});
