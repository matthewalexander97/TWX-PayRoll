import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { ExportSettings } from "./ExportSettings";
import { exportSettings } from "../../contracts/exports";
vi.mock("../../contracts/exports", () => ({
  exportSettings: {
    status: vi.fn(),
    select: vi.fn(),
    reset: vi.fn(),
    open: vi.fn(),
  },
}));
beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(exportSettings.status).mockResolvedValue({
    directory: "Resolved Documents",
    usesDocuments: true,
  });
});
afterEach(cleanup);
it("shows Documents, saves native selection, opens only that setting and resets", async () => {
  render(<ExportSettings enabled />);
  await screen.findByText("Resolved Documents");
  vi.mocked(exportSettings.select).mockResolvedValue({
    directory: "Selected folder",
    usesDocuments: false,
  });
  fireEvent.click(screen.getByRole("button", { name: "Change Folder" }));
  await screen.findByText("Selected folder");
  fireEvent.click(screen.getByRole("button", { name: "Open Export Folder" }));
  await screen.findByText("Export folder opened.");
  expect(exportSettings.open).toHaveBeenCalledWith();
  vi.mocked(exportSettings.reset).mockResolvedValue({
    directory: "Current Documents",
    usesDocuments: true,
  });
  fireEvent.click(screen.getByRole("button", { name: "Reset to Documents" }));
  await screen.findByText("Current Documents");
  expect(screen.queryByRole("textbox")).toBeNull();
});
it("keeps previous selection on cancel and makes an unavailable folder recoverable", async () => {
  render(<ExportSettings enabled />);
  await screen.findByText("Resolved Documents");
  vi.mocked(exportSettings.select).mockResolvedValue(null);
  fireEvent.click(screen.getByRole("button", { name: "Change Folder" }));
  await screen.findByText(/selection canceled/);
  expect(screen.getByText("Resolved Documents")).toBeTruthy();
  vi.mocked(exportSettings.open).mockRejectedValue(
    "Export folder is unavailable. Change Folder.",
  );
  fireEvent.click(screen.getByRole("button", { name: "Open Export Folder" }));
  await screen.findByRole("alert");
  await waitFor(() =>
    expect(
      (
        screen.getByRole("button", {
          name: "Change Folder",
        }) as HTMLButtonElement
      ).disabled,
    ).toBe(false),
  );
});
it("resolver failure keeps native choose and reset controls available", async () => {
  vi.mocked(exportSettings.status).mockRejectedValue(
    "Documents cannot be resolved.",
  );
  render(<ExportSettings enabled />);
  await screen.findByRole("alert");
  expect(
    (screen.getByRole("button", { name: "Change Folder" }) as HTMLButtonElement)
      .disabled,
  ).toBe(false);
});
it("does not invoke desktop commands in browser preview", () => {
  render(<ExportSettings enabled={false} />);
  expect(exportSettings.status).not.toHaveBeenCalled();
  expect(
    (screen.getByRole("button", { name: "Change Folder" }) as HTMLButtonElement)
      .disabled,
  ).toBe(true);
});
