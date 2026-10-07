import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  business,
  emptyReferences,
  money,
  decimal,
  type Aggregate,
  type References,
  type Row,
} from "../contracts/business";
import { Payroll } from "./payroll/Payroll";
import { Settlements } from "./settlements/Settlements";
import { ReferenceManager } from "./references/ReferenceManager";
vi.mock("../contracts/business", async (importOriginal) => {
  const a = await importOriginal<typeof import("../contracts/business")>();
  return {
    ...a,
    business: Object.fromEntries(
      Object.keys(a.business).map((k) => [k, vi.fn()]),
    ),
  };
});
const base = { row_version: "1", is_active: "1" };
const refs: References = {
  ...emptyReferences(),
  driver_types: [
    { ...base, id: "type", name: "Synthetic Type", payout_rate_ppm: "850000" },
  ],
  drivers: [
    { ...base, id: "driver", name: "Synthetic Driver", driver_type_id: "type" },
  ],
  owners: [{ ...base, id: "owner", name: "Synthetic Owner" }],
  trucks: [
    {
      ...base,
      id: "truck",
      truck_number: "FAKE-001",
      owner_id: "owner",
      default_driver_id: "driver",
    },
    {
      ...base,
      id: "truck2",
      truck_number: "FAKE-002",
      owner_id: "owner",
      default_driver_id: null,
    },
  ],
  twx_codes: [
    { ...base, id: "code", code: "FAKE-CODE", label: "Synthetic Code" },
  ],
  adjustment_types: [
    {
      ...base,
      id: "deduct",
      name: "Insurance",
      default_direction: "DEDUCT",
      twx_code_id: "code",
    },
    {
      ...base,
      id: "add",
      name: "Reimbursement",
      default_direction: "ADD",
      twx_code_id: null,
    },
  ],
};
const header: Row = {
  id: "settlement",
  row_version: "1",
  driver_id: "driver",
  truck_id: "truck",
  period_start: "2026-10-01",
  period_end: "2026-10-19",
  driver_name_snapshot: "Synthetic Driver",
  truck_number_snapshot: "FAKE-001",
  driver_type_name_snapshot: "Synthetic Type",
  payout_rate_ppm: "850000",
  status: "DRAFT",
  notes: "",
  final_payout_cents: null,
};
const load: Row = {
  id: "load1",
  row_version: "1",
  driver_id: "driver",
  truck_id: "truck",
  plant_id: null,
  job_id: null,
  load_number: "FAKE-LOAD",
  bol_number: "FAKE-BOL",
  po_number: "FAKE-PO",
  tons_milli: "10000",
  completion_rate_scaled: "50000",
  mileage: "12.3456789",
  total_cents: null,
};
const aggregate: Aggregate = {
  header,
  loads: [
    load,
    {
      ...load,
      id: "load2",
      tons_milli: "20000",
      completion_rate_scaled: "40000",
    },
  ],
  adjustments: [],
};
const driver: Row = {
  id: "payroll-driver",
  row_version: "1",
  entry_kind: "DRIVER",
  driver_id: "driver",
  owner_id: null,
  truck_id: "truck",
  payee_name_snapshot: "Synthetic Driver",
  truck_number_snapshot: "FAKE-001",
  settlement_id: "settlement",
  period_start: "2026-10-01",
  period_end: "2026-10-19",
  amount_cents: "10250",
  ach_charge_cents: "0",
  total_cents: "10250",
  twx_code_id: null,
  twx_code_snapshot: null,
  notes: "",
};
const owner: Row = {
  ...driver,
  id: "payroll-owner",
  entry_kind: "OWNER",
  driver_id: null,
  owner_id: "owner",
  payee_name_snapshot: "Synthetic Owner",
  settlement_id: null,
  amount_cents: "12345",
  ach_charge_cents: null,
  total_cents: "12345",
};
const totals = {
  loadTotalsCents: ["5000", "8000"],
  payoutRatePpm: "850000",
  grossRouteCents: "13000",
  driverGrossCents: "11050",
  finalPayoutCents: "10250",
  negative: false,
};
beforeEach(() => {
  vi.resetAllMocks();
  vi.mocked(business.payrolls).mockResolvedValue([driver, owner]);
  vi.mocked(business.settlements).mockResolvedValue([header]);
  vi.mocked(business.settlement).mockResolvedValue(aggregate);
  vi.mocked(business.preview).mockResolvedValue(totals);
  vi.mocked(business.history).mockResolvedValue([]);
  vi.mocked(business.saveSettlement).mockResolvedValue(aggregate);
  vi.mocked(business.finalize).mockResolvedValue({
    ...aggregate,
    header: {
      ...header,
      status: "FINALIZED",
      row_version: "2",
      final_payout_cents: "10250",
    },
  });
  vi.mocked(business.saveDriver).mockResolvedValue({
    ...driver,
    ach_charge_cents: "125",
    total_cents: "10375",
    row_version: "2",
  });
  vi.mocked(business.saveOwner).mockResolvedValue(owner);
  vi.mocked(business.saveReference).mockResolvedValue(refs.owners[0]!);
});
afterEach(cleanup);
async function openSettlement() {
  render(<Settlements refs={refs} enabled onChanged={vi.fn()} />);
  fireEvent.click(
    await screen.findByRole("button", { name: /Synthetic Driver · Truck/ }),
  );
  await screen.findByRole("heading", { name: "Settlement detail" });
}
describe("payroll and exact presentation", () => {
  it("renders separate driver/owner areas, derived amount and driver-only ACH", async () => {
    render(<Payroll refs={refs} enabled onChanged={vi.fn()} />);
    await screen.findByRole("button", { name: /Synthetic Owner · Truck/ });
    expect(screen.getAllByText("$102.50").length).toBeGreaterThan(0);
    fireEvent.click(screen.getByRole("button", { name: "Synthetic Driver" }));
    expect(
      (screen.getByLabelText("Settlement Amount") as HTMLInputElement).readOnly,
    ).toBe(true);
    expect(screen.getByLabelText("ACH Charge")).toBeTruthy();
    fireEvent.click(
      screen.getByRole("button", { name: /Synthetic Owner · Truck/ }),
    );
    const own = screen.getByRole("group", { name: "Owner payroll fields" });
    expect(within(own).queryByLabelText(/ACH/)).toBeNull();
    expect(
      (within(own).getByLabelText("Owner Amount") as HTMLInputElement).value,
    ).toBe("123.45");
  });
  it("sends decimal ACH/code/notes only and displays the backend driver total", async () => {
    render(<Payroll refs={refs} enabled onChanged={vi.fn()} />);
    fireEvent.click(
      await screen.findByRole("button", {
        name: "Synthetic Driver",
      }),
    );
    fireEvent.change(screen.getByLabelText("ACH Charge"), {
      target: { value: "1.25" },
    });
    fireEvent.change(screen.getByLabelText("TWX Code"), {
      target: { value: "code" },
    });
    fireEvent.change(screen.getByLabelText("Driver Payroll Notes"), {
      target: { value: "Synthetic note" },
    });
    fireEvent.click(
      screen.getByRole("button", { name: "Save Driver Payroll" }),
    );
    await screen.findByText("$103.75");
    expect(business.saveDriver).toHaveBeenCalledWith({
      id: driver.id,
      expectedVersion: "1",
      achCharge: "1.25",
      twxCodeId: "code",
      notes: "Synthetic note",
    });
  });
  it("keeps owner input after a backend error and supports custom dates/manual amount", async () => {
    vi.mocked(business.saveOwner).mockRejectedValue({
      code: "VALIDATION",
      message: "Too many decimal places",
      field: "amount",
    });
    render(<Payroll refs={refs} enabled onChanged={vi.fn()} />);
    fireEvent.click(screen.getByRole("button", { name: "New Owner Payroll" }));
    fireEvent.change(screen.getByLabelText("Owner Amount"), {
      target: { value: "12.345" },
    });
    fireEvent.change(screen.getByLabelText("Owner Pay-period start"), {
      target: { value: "2026-10-01" },
    });
    fireEvent.change(screen.getByLabelText("Owner Pay-period end"), {
      target: { value: "2026-10-19" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Save Owner Payroll" }));
    await screen.findByRole("alert");
    expect(
      (screen.getByLabelText("Owner Amount") as HTMLInputElement).value,
    ).toBe("12.345");
    expect(
      screen.getByLabelText("Owner Amount").getAttribute("aria-invalid"),
    ).toBe("true");
  });
  it("filters history using dates/truck while owner remains separate", async () => {
    render(<Payroll refs={refs} enabled onChanged={vi.fn()} />);
    fireEvent.change(screen.getByLabelText("History from"), {
      target: { value: "2026-10-18" },
    });
    fireEvent.change(screen.getByLabelText("Filter by truck"), {
      target: { value: "truck2" },
    });
    await waitFor(() =>
      expect(business.payrolls).toHaveBeenLastCalledWith({
        periodStart: "2026-10-18",
        periodEnd: null,
        driverId: null,
        truckId: "truck2",
      }),
    );
  });
  it("formats all i64 bits without converting money to Number", () => {
    expect(money("9223372036854775807")).toBe("$92233720368547758.07");
    expect(money("-100")).toBe("-$1.00");
    expect(decimal("12345", 3)).toBe("12.345");
  });
});
describe("settlement workflow", () => {
  it("creates custom draft, truck default driver and multiple editable loads", async () => {
    render(<Settlements refs={refs} enabled onChanged={vi.fn()} />);
    fireEvent.click(screen.getByRole("button", { name: "New Settlement" }));
    fireEvent.change(screen.getByLabelText("Pay-period start"), {
      target: { value: "2026-10-01" },
    });
    fireEvent.change(screen.getByLabelText("Pay-period end"), {
      target: { value: "2026-10-19" },
    });
    fireEvent.change(screen.getByLabelText("Settlement Truck #"), {
      target: { value: "truck" },
    });
    expect(
      (screen.getByLabelText("Settlement Driver") as HTMLSelectElement).value,
    ).toBe("driver");
    expect(
      (screen.getByLabelText("Payout Percentage") as HTMLInputElement).value,
    ).toBe("85.0000");
    for (let i = 1; i <= 2; i++) {
      fireEvent.click(screen.getByRole("button", { name: "Add Load" }));
      const g = within(screen.getByRole("group", { name: `Load ${i}` }));
      fireEvent.change(g.getByLabelText("Net Weight / Tons"), {
        target: { value: i === 1 ? "10" : "20" },
      });
      fireEvent.change(g.getByLabelText("Completion Rate"), {
        target: { value: i === 1 ? "5" : "4" },
      });
    }
    await screen.findByText("$50.00");
    expect(screen.getByText("$80.00")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Save Draft" }));
    await screen.findByText("Draft saved.");
    expect(business.saveSettlement).toHaveBeenCalledWith(
      expect.objectContaining({
        periodStart: "2026-10-01",
        periodEnd: "2026-10-19",
        loads: expect.arrayContaining([
          expect.objectContaining({ tons: "10", completionRate: "5" }),
        ]),
      }),
      false,
    );
  });
  it("uses backend totals and explicit ADD/DEDUCT types, then finalizes linkage", async () => {
    await openSettlement();
    for (const [i, type] of ["deduct", "add"].entries()) {
      fireEvent.click(screen.getByRole("button", { name: "Add Adjustment" }));
      const g = within(
        screen.getByRole("group", { name: `Adjustment ${i + 1}` }),
      );
      fireEvent.change(g.getByLabelText("Adjustment Type"), {
        target: { value: type },
      });
      fireEvent.change(g.getByLabelText("Adjustment Amount"), {
        target: { value: i === 0 ? "10" : "2" },
      });
      expect((g.getByLabelText("Direction") as HTMLSelectElement).value).toBe(
        i === 0 ? "DEDUCT" : "ADD",
      );
    }
    await waitFor(() =>
      expect(screen.getByTestId("final-payout").textContent).toBe("$102.50"),
    );
    fireEvent.click(
      screen.getByRole("button", { name: "Finalize Settlement" }),
    );
    await screen.findByText(/Settlement finalized/);
    expect(business.finalize).toHaveBeenCalledWith("settlement", "1", false);
  });
  it("requires deliberate negative confirmation and preserves the signed result", async () => {
    vi.mocked(business.preview).mockResolvedValue({
      ...totals,
      negative: true,
      finalPayoutCents: "-100",
    });
    await openSettlement();
    await screen.findByText(/Final payout is negative/);
    expect(
      (
        screen.getByRole("button", {
          name: "Finalize Settlement",
        }) as HTMLButtonElement
      ).disabled,
    ).toBe(true);
    fireEvent.click(
      screen.getByLabelText("I confirm this negative final payout."),
    );
    expect(
      (
        screen.getByRole("button", {
          name: "Finalize Settlement",
        }) as HTMLButtonElement
      ).disabled,
    ).toBe(false);
    fireEvent.click(
      screen.getByRole("button", { name: "Finalize Settlement" }),
    );
    await waitFor(() =>
      expect(business.finalize).toHaveBeenCalledWith("settlement", "1", true),
    );
  });
  it("retains finalized edits on stale version conflict and offers reload", async () => {
    vi.mocked(business.settlement).mockResolvedValue({
      ...aggregate,
      header: { ...header, status: "FINALIZED", row_version: "2" },
    });
    vi.mocked(business.saveSettlement).mockRejectedValue({
      code: "CONFLICT",
      message: "Record changed; reload before saving",
      field: null,
    });
    await openSettlement();
    fireEvent.change(screen.getByLabelText("Settlement Notes"), {
      target: { value: "Unsaved synthetic correction" },
    });
    fireEvent.click(
      screen.getByRole("button", { name: "Save Finalized Edit" }),
    );
    await screen.findByRole("alert");
    expect(
      (screen.getByLabelText("Settlement Notes") as HTMLTextAreaElement).value,
    ).toBe("Unsaved synthetic correction");
    expect(
      screen.getByRole("button", { name: "Reload latest record" }),
    ).toBeTruthy();
    expect(business.saveSettlement).toHaveBeenCalledWith(
      expect.objectContaining({
        expectedVersion: "2",
        notes: "Unsaved synthetic correction",
      }),
      false,
    );
  });
  it("shows inline exact-value errors without losing form values", async () => {
    await openSettlement();
    vi.mocked(business.preview).mockRejectedValue({
      code: "VALIDATION",
      message: "Tons allow 3 decimal places",
      field: "loads.0.tons",
    });
    const input = within(
      screen.getByRole("group", { name: "Load 1" }),
    ).getByLabelText("Net Weight / Tons");
    fireEvent.change(input, { target: { value: "0.0001" } });
    await screen.findByRole("alert");
    expect(input.getAttribute("aria-invalid")).toBe("true");
    expect((input as HTMLInputElement).value).toBe("0.0001");
  });
  it("shows saved audit events and finalized edit success", async () => {
    vi.mocked(business.settlement).mockResolvedValue({
      ...aggregate,
      header: { ...header, status: "FINALIZED" },
    });
    vi.mocked(business.history).mockResolvedValue([
      {
        id: "1",
        row_version: "1",
        category: "CORRECTION",
        occurred_at: "2026-10-05T00:00:00Z",
        reason: "Synthetic audit reason",
        before_json: "{}",
        after_json: "{}",
      },
    ]);
    await openSettlement();
    expect(await screen.findByText("Synthetic audit reason")).toBeTruthy();
    fireEvent.click(
      screen.getByRole("button", { name: "Save Finalized Edit" }),
    );
    await waitFor(() => expect(business.saveSettlement).toHaveBeenCalled());
  });
});
describe("reference forms", () => {
  it("lists one owner with multiple trucks and edits default driver explicitly", async () => {
    const changed = vi.fn().mockResolvedValue(undefined);
    render(
      <ReferenceManager refs={refs} kinds={["trucks"]} onChanged={changed} />,
    );
    expect(screen.getAllByText(/Synthetic Owner/).length).toBe(2);
    fireEvent.click(screen.getByRole("button", { name: /FAKE-001/ }));
    fireEvent.change(screen.getByLabelText("Default Driver"), {
      target: { value: "" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Save Reference" }));
    await waitFor(() => expect(changed).toHaveBeenCalled());
    expect(business.saveReference).toHaveBeenCalledWith(
      expect.objectContaining({
        kind: "trucks",
        id: "truck",
        expectedVersion: "1",
        defaultDriverId: null,
      }),
    );
  });
  it("edits driver type percentage and supports known adjustment defaults without seeds", async () => {
    const changed = vi.fn().mockResolvedValue(undefined);
    const v = render(
      <ReferenceManager
        refs={refs}
        kinds={["driver_types"]}
        onChanged={changed}
      />,
    );
    fireEvent.click(screen.getByRole("button", { name: /Synthetic Type/ }));
    fireEvent.change(screen.getByLabelText("Payout Percentage"), {
      target: { value: "85.1234" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Save Reference" }));
    await waitFor(() =>
      expect(business.saveReference).toHaveBeenCalledWith(
        expect.objectContaining({ payoutPercentage: "85.1234" }),
      ),
    );
    v.unmount();
    render(
      <ReferenceManager
        refs={refs}
        kinds={["adjustment_types"]}
        onChanged={changed}
      />,
    );
    fireEvent.click(
      screen.getByRole("button", { name: "Add Adjustment Type" }),
    );
    fireEvent.click(screen.getByRole("button", { name: "Use Reimbursement" }));
    expect((screen.getByLabelText("Name") as HTMLInputElement).value).toBe(
      "Reimbursement",
    );
    expect(
      (screen.getByLabelText("Default Direction") as HTMLSelectElement).value,
    ).toBe("ADD");
  });
});
