import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { BackupPanel } from "./BackupPanel";
import { recovery, type BackupStatus } from "../../contracts/desktop";
vi.mock("../../contracts/desktop", () => ({
  recovery: {
    status: vi.fn(),
    selectFolder: vi.fn(),
    backup: vi.fn(),
    automatic: vi.fn(),
    selectRestore: vi.fn(),
    cancelRestore: vi.fn(),
    restore: vi.fn(),
  },
}));
const baseline: BackupStatus = {
  destination: "synthetic/backups",
  automaticEnabled: true,
  retentionCount: 10,
  notice: "Configured",
  lastSuccessfulBackup: null,
  runs: [],
};
beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(recovery.status).mockResolvedValue(baseline);
  vi.mocked(recovery.selectRestore).mockResolvedValue({
    token: "opaque-test-token",
    filename: "synthetic.sqlite3",
    schemaVersion: 3,
    createdAt: null,
  });
  vi.mocked(recovery.restore).mockResolvedValue(baseline);
  vi.mocked(recovery.cancelRestore).mockResolvedValue(undefined);
});
afterEach(cleanup);
it("requires deliberate confirmation and refreshes after successful restore", async () => {
  const refreshed = vi.fn().mockResolvedValue(undefined);
  render(<BackupPanel enabled onRestored={refreshed} />);
  await screen.findByText("synthetic/backups");
  fireEvent.click(screen.getByRole("button", { name: "Restore Backup" }));
  const confirm = await screen.findByRole("button", {
    name: "Confirm Restore",
  });
  expect(confirm.hasAttribute("disabled")).toBe(true);
  expect(recovery.restore).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole("checkbox", { name: /I understand/ }));
  fireEvent.click(confirm);
  await screen.findByText(/Backup restored/);
  expect(recovery.restore).toHaveBeenCalledWith("opaque-test-token");
  expect(refreshed).toHaveBeenCalledTimes(1);
});
it("cancels without invoking restore", async () => {
  render(<BackupPanel enabled onRestored={vi.fn()} />);
  await screen.findByText("synthetic/backups");
  fireEvent.click(screen.getByRole("button", { name: "Restore Backup" }));
  fireEvent.click(
    await screen.findByRole("button", { name: "Cancel Restore" }),
  );
  await waitFor(() => expect(recovery.cancelRestore).toHaveBeenCalled());
  expect(recovery.restore).not.toHaveBeenCalled();
});
it("keeps failure visible without displaying false success", async () => {
  vi.mocked(recovery.backup).mockRejectedValue(
    "Synthetic destination unavailable",
  );
  render(<BackupPanel enabled onRestored={vi.fn()} />);
  await screen.findByText("synthetic/backups");
  fireEvent.click(screen.getByRole("button", { name: "Backup Now" }));
  expect((await screen.findByRole("alert")).textContent).toContain(
    "destination unavailable",
  );
  expect(screen.queryByText("Backup created and validated.")).toBeNull();
});
it("does not expose path entry or backup operations in browser preview", () => {
  render(<BackupPanel enabled={false} onRestored={vi.fn()} />);
  expect(screen.getByText(/available in the desktop application/)).toBeTruthy();
  expect(recovery.status).not.toHaveBeenCalled();
  expect(screen.queryByRole("textbox")).toBeNull();
});
it("handles native picker cancellation without enabling confirmation", async () => {
  vi.mocked(recovery.selectRestore).mockResolvedValue(null);
  render(<BackupPanel enabled onRestored={vi.fn()} />);
  await screen.findByText("synthetic/backups");
  fireEvent.click(screen.getByRole("button", { name: "Restore Backup" }));
  await waitFor(() => expect(recovery.selectRestore).toHaveBeenCalled());
  expect(screen.queryByRole("button", { name: "Confirm Restore" })).toBeNull();
});
