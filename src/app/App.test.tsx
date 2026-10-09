import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { App } from "./App";
import { desktop } from "../contracts/desktop";
vi.mock("../contracts/exports", () => ({
  exportSettings: {
    status: vi.fn().mockResolvedValue({
      directory: "Synthetic Documents",
      usesDocuments: true,
    }),
  },
}));
vi.mock("../contracts/integrations", () => ({
  integrations: { status: vi.fn().mockResolvedValue([]) },
  integrationError: () => "Integration unavailable",
}));
vi.mock("../contracts/business", async (importOriginal) => {
  const actual = await importOriginal<typeof import("../contracts/business")>();
  return {
    ...actual,
    business: {
      ...actual.business,
      references: vi.fn().mockResolvedValue(actual.emptyReferences()),
      payrolls: vi.fn().mockResolvedValue([]),
    },
  };
});
vi.mock("../contracts/desktop", () => ({
  recovery: {
    status: vi.fn().mockResolvedValue({
      destination: null,
      automaticEnabled: true,
      retentionCount: 10,
      notice: "Choose a backup folder",
      lastSuccessfulBackup: null,
      runs: [],
    }),
  },
  desktop: {
    available: vi.fn(),
    preferences: vi.fn(),
    info: vi.fn(),
    saveTheme: vi.fn(),
  },
}));
beforeEach(() => {
  vi.clearAllMocks();
  Object.defineProperty(window, "matchMedia", {
    writable: true,
    value: vi.fn(() => ({
      matches: false,
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
    })),
  });
  vi.mocked(desktop.available).mockReturnValue(true);
  vi.mocked(desktop.preferences).mockResolvedValue({ theme: "dark" });
  vi.mocked(desktop.info).mockResolvedValue({
    version: "0.1.0",
    databasePath: "synthetic/data/twx-payroll.sqlite3",
    schemaVersion: 1,
    sqliteVersion: "test",
  });
  vi.mocked(desktop.saveTheme).mockImplementation(async (theme) => ({ theme }));
});
afterEach(cleanup);
async function settings() {
  render(<App />);
  fireEvent.click(await screen.findByRole("button", { name: "Settings" }));
}
describe("desktop foundation", () => {
  it("places reference records below Reports and keeps integrations last in Settings", async () => {
    await settings();
    const nav = screen.getByRole("navigation");
    const labels = within(nav)
      .getAllByRole("button")
      .map((button) => button.textContent);
    expect(
      labels.findIndex((label) => label?.includes("Reference Records")),
    ).toBe(labels.findIndex((label) => label?.includes("Reports")) + 1);
    expect(
      screen.queryByRole("heading", { name: "Reference records" }),
    ).toBeNull();
    const headings = screen.getAllByRole("heading", { level: 2 });
    expect(headings.at(-1)?.textContent).toBe("Optional integrations");
    fireEvent.click(screen.getByRole("button", { name: "Reference Records" }));
    expect(
      await screen.findByRole("heading", {
        name: "Reference records",
      }),
    ).toBeTruthy();
    for (const name of ["TWX Codes", "Plants", "Jobs", "Adjustment Types"])
      expect(screen.getByRole("button", { name })).toBeTruthy();
    expect(
      screen.queryByRole("heading", { name: "Optional integrations" }),
    ).toBeNull();
  });
  it("restores a stored preference and exposes all placeholder areas", async () => {
    await settings();
    expect(document.documentElement.dataset.theme).toBe("dark");
    expect(screen.getByText("synthetic/data/twx-payroll.sqlite3")).toBeTruthy();
    for (const name of [
      "TWX Payroll",
      "Weekly Settlements / Loads",
      "Drivers",
      "Owners",
      "Trucks",
      "Reports",
    ])
      expect(screen.getByRole("button", { name })).toBeTruthy();
  });
  it("saves through the desktop boundary before applying the preference", async () => {
    await settings();
    fireEvent.click(screen.getByRole("button", { name: "light" }));
    await waitFor(() =>
      expect(document.documentElement.dataset.theme).toBe("light"),
    );
    expect(desktop.saveTheme).toHaveBeenCalledWith("light");
  });
  it("keeps the old theme and reports a failed persistence operation", async () => {
    vi.mocked(desktop.saveTheme).mockRejectedValue(
      new Error("synthetic failure"),
    );
    await settings();
    fireEvent.click(screen.getByRole("button", { name: "light" }));
    expect((await screen.findByRole("alert")).textContent).toContain(
      "not saved",
    );
    expect(document.documentElement.dataset.theme).toBe("dark");
  });
  it("makes browser preview limitations explicit without using fake persistence", async () => {
    vi.mocked(desktop.available).mockReturnValue(false);
    await settings();
    expect(screen.getByText(/Browser preview/)).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "light" }));
    expect(desktop.saveTheme).not.toHaveBeenCalled();
    expect(desktop.preferences).not.toHaveBeenCalled();
  });
  it("reports startup settings failure without silently overwriting storage", async () => {
    vi.mocked(desktop.preferences).mockRejectedValue(
      new Error("synthetic startup failure"),
    );
    render(<App />);
    expect((await screen.findByRole("alert")).textContent).toContain(
      "could not be loaded",
    );
    expect(desktop.saveTheme).not.toHaveBeenCalled();
  });
});
