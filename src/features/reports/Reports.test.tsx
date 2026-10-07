import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { Reports } from "./Reports";
import { ExportActions } from "./ExportActions";
import {
  reports,
  reportRequest,
  reportCell,
  type Report,
} from "../../contracts/reports";
import { emptyReferences } from "../../contracts/business";
vi.mock("../../contracts/reports", async (original) => ({
  ...(await original<typeof import("../../contracts/reports")>()),
  reports: {
    preview: vi.fn(),
    export: vi.fn(),
    saveAs: vi.fn(),
    print: vi.fn(),
  },
}));
const fixture: Report = {
  kind: "owner_payroll",
  title: "Owner Payroll",
  period: "10/01/2026 – 10/19/2026",
  draft: true,
  currency: "USD",
  sourceVersions: ["payroll:fake:1"],
  defaultName: "TWX-Owner",
  sections: [
    {
      name: "Owner Payroll",
      columns: ["Owner Name", "Amount"],
      rows: [
        [
          { kind: "text", value: "Synthetic Owner" },
          { kind: "money", value: "-5000" },
        ],
      ],
    },
  ],
};
beforeEach(() => vi.resetAllMocks());
afterEach(cleanup);
it("allows archived references in history filters without changing saved labels", async () => {
  const refs = emptyReferences();
  refs.drivers = [
    {
      id: "archived-driver",
      row_version: "2",
      is_active: "0",
      name: "Synthetic Old Driver",
    },
  ];
  refs.trucks = [
    {
      id: "archived-truck",
      row_version: "2",
      is_active: "0",
      truck_number: "OLD-FAKE",
    },
  ];
  refs.owners = [
    {
      id: "archived-owner",
      row_version: "2",
      is_active: "0",
      name: "Synthetic Old Owner",
    },
  ];
  vi.mocked(reports.preview).mockResolvedValue(fixture);
  render(<Reports refs={refs} enabled />);
  expect(
    screen.getByRole("option", { name: "Synthetic Old Driver (archived)" }),
  ).toBeTruthy();
  expect(
    screen.getByRole("option", { name: "OLD-FAKE (archived)" }),
  ).toBeTruthy();
  fireEvent.change(screen.getByLabelText("Filter by owner"), {
    target: { value: "archived-owner" },
  });
  fireEvent.click(screen.getByRole("button", { name: "Preview Saved Report" }));
  await waitFor(() =>
    expect(reports.preview).toHaveBeenCalledWith({
      ...reportRequest("payroll_summary"),
      ownerId: "archived-owner",
    }),
  );
});
it("formats exact signed USD and large integers without Number conversion", () => {
  expect(reportCell({ kind: "money", value: "-5000" })).toBe("-$50.00");
  expect(reportCell({ kind: "money", value: "9223372036854775807" })).toBe(
    "$92,233,720,368,547,758.07",
  );
  expect(reportCell({ kind: "percentage", value: "850000" })).toBe("85.0000%");
});
it("previews saved filters, marks drafts and clears stale previews", async () => {
  vi.mocked(reports.preview).mockResolvedValue(fixture);
  render(<Reports refs={emptyReferences()} enabled />);
  fireEvent.change(screen.getByLabelText("Report type"), {
    target: { value: "owner_payroll" },
  });
  expect(screen.queryByLabelText("Filter by driver")).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "Preview Saved Report" }));
  await screen.findByText("-$50.00");
  expect(screen.getByText("DRAFT / NOT FINALIZED")).toBeTruthy();
  expect(reports.preview).toHaveBeenCalledWith(reportRequest("owner_payroll"));
  fireEvent.change(screen.getByLabelText("Report type"), {
    target: { value: "driver_payroll" },
  });
  expect(screen.queryByText("-$50.00")).toBeNull();
});
it("shows native cancellation and errors and sends only identifiers/filters", async () => {
  vi.mocked(reports.export).mockResolvedValue(null);
  const request = reportRequest("settlement_detail");
  request.settlementId = "fake";
  render(<ExportActions request={request} />);
  fireEvent.change(screen.getByLabelText("Export format"), {
    target: { value: "csv" },
  });
  fireEvent.click(screen.getByRole("button", { name: "Export Saved Report" }));
  await screen.findByText("Save canceled. No file was published.");
  expect(reports.export).toHaveBeenCalledWith(request, "csv");
  vi.mocked(reports.export).mockRejectedValue("Choose a new filename");
  fireEvent.click(screen.getByRole("button", { name: "Export Saved Report" }));
  expect(await screen.findByRole("alert")).toHaveProperty(
    "textContent",
    "Choose a new filename",
  );
});
it("disables concurrent commands and provides local print preview feedback", async () => {
  let resolve!: (value: { fileName: string; message: string }) => void;
  vi.mocked(reports.print).mockImplementation(
    () =>
      new Promise((r) => {
        resolve = r;
      }),
  );
  render(<ExportActions request={reportRequest("driver_payroll")} />);
  fireEvent.click(screen.getByRole("button", { name: "Print / Preview PDF" }));
  expect(
    (screen.getByRole("button", { name: "Working…" }) as HTMLButtonElement)
      .disabled,
  ).toBe(true);
  resolve({ fileName: "preview.pdf", message: "Opened local preview" });
  await waitFor(() =>
    expect(screen.getByRole("status").textContent).toContain("preview.pdf"),
  );
});
it("retains a clear error for empty/corrupt saved reports", async () => {
  vi.mocked(reports.preview).mockRejectedValue("No saved records match");
  render(<Reports refs={emptyReferences()} enabled />);
  fireEvent.click(screen.getByRole("button", { name: "Preview Saved Report" }));
  expect(await screen.findByRole("alert")).toHaveProperty(
    "textContent",
    "No saved records match",
  );
  expect(
    screen.queryByRole("button", { name: "Export Saved Report" }),
  ).toBeNull();
});
