import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { ExternalActions } from "./ExternalActions";
import { IntegrationSettings } from "../settings/IntegrationSettings";
import { integrations } from "../../contracts/integrations";
import { reportRequest } from "../../contracts/reports";
vi.mock("../../contracts/integrations", async (original) => ({
  ...(await original<typeof import("../../contracts/integrations")>()),
  integrations: {
    status: vi.fn(),
    connect: vi.fn(),
    cancelConnection: vi.fn(),
    disconnect: vi.fn(),
    prepare: vi.fn(),
    confirm: vi.fn(),
    cancel: vi.fn(),
  },
}));
const request = reportRequest("settlement_detail");
const prepared = {
  token: "synthetic-token",
  service: "sheets" as const,
  title: "Synthetic settlement",
  period: "10/01/2026 – 10/06/2026",
  draft: true,
  account: "test@example.invalid",
  recipient: null,
  subject: null,
  fileName: null,
  format: null,
};
beforeEach(() => {
  vi.resetAllMocks();
  vi.mocked(integrations.status).mockResolvedValue([
    {
      service: "sheets",
      connected: false,
      account: null,
      message: "Disconnected",
    },
    {
      service: "email",
      connected: false,
      account: null,
      message: "Disconnected",
    },
  ]);
  vi.mocked(integrations.prepare).mockResolvedValue(prepared);
  vi.mocked(integrations.confirm).mockResolvedValue({
    message: "Accepted snapshot",
    destination: null,
  });
  vi.mocked(integrations.cancel).mockResolvedValue();
});
afterEach(cleanup);
it("prepares a saved Sheets snapshot and requires deliberate confirmation", async () => {
  render(<ExternalActions request={request} enabled />);
  fireEvent.click(
    screen.getByRole("button", { name: "Send to Google Sheets" }),
  );
  expect(await screen.findByText(/DRAFT \/ NOT FINALIZED/u)).toBeTruthy();
  expect(integrations.prepare).toHaveBeenCalledWith(request, "sheets", null);
  expect(integrations.confirm).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole("button", { name: "Confirm Create Sheet" }));
  expect(await screen.findByText("Accepted snapshot")).toBeTruthy();
  expect(integrations.confirm).toHaveBeenCalledTimes(1);
});
it("cancels without sending and consumes the prepared token", async () => {
  render(<ExternalActions request={request} enabled />);
  fireEvent.click(
    screen.getByRole("button", { name: "Send to Google Sheets" }),
  );
  fireEvent.click(await screen.findByRole("button", { name: "Cancel Send" }));
  expect(await screen.findByText("Canceled. Nothing was sent.")).toBeTruthy();
  expect(integrations.cancel).toHaveBeenCalledWith("synthetic-token");
  expect(integrations.confirm).not.toHaveBeenCalled();
});
it("shows disconnected/offline errors without sending", async () => {
  vi.mocked(integrations.prepare).mockRejectedValue(
    "Disconnected; network required",
  );
  render(<ExternalActions request={request} enabled />);
  fireEvent.click(
    screen.getByRole("button", { name: "Send to Google Sheets" }),
  );
  expect((await screen.findByRole("alert")).textContent).toContain(
    "network required",
  );
  expect(integrations.confirm).not.toHaveBeenCalled();
});
it("defaults email to PDF and sends the entered recipient, subject and message only to preparation", async () => {
  vi.mocked(integrations.prepare).mockResolvedValue({
    ...prepared,
    service: "email",
    recipient: "recipient@example.invalid",
    subject: "Synthetic email",
    fileName: "synthetic.pdf",
    format: "pdf",
  });
  render(<ExternalActions request={request} enabled />);
  fireEvent.click(screen.getByRole("button", { name: "Email Report" }));
  expect(
    (screen.getByLabelText("Attachment format") as HTMLSelectElement).value,
  ).toBe("pdf");
  fireEvent.change(screen.getByLabelText("Recipient"), {
    target: { value: "recipient@example.invalid" },
  });
  fireEvent.change(screen.getByLabelText("Subject"), {
    target: { value: "Synthetic email" },
  });
  fireEvent.change(screen.getByLabelText("Optional message"), {
    target: { value: "Synthetic message" },
  });
  fireEvent.submit(
    screen.getByRole("form", { name: "Email report composition" }),
  );
  await screen.findByRole("button", { name: "Confirm Send Email" });
  expect(integrations.prepare).toHaveBeenCalledWith(request, "email", {
    recipient: "recipient@example.invalid",
    subject: "Synthetic email",
    body: "Synthetic message",
    format: "pdf",
  });
  expect(integrations.confirm).not.toHaveBeenCalled();
});
it("shows uncertain delivery once and never retries automatically", async () => {
  vi.mocked(integrations.confirm).mockRejectedValue(
    "Delivery uncertain; check Sent Mail",
  );
  render(<ExternalActions request={request} enabled />);
  fireEvent.click(
    screen.getByRole("button", { name: "Send to Google Sheets" }),
  );
  fireEvent.click(
    await screen.findByRole("button", { name: "Confirm Create Sheet" }),
  );
  expect((await screen.findByRole("alert")).textContent).toContain("uncertain");
  expect(integrations.confirm).toHaveBeenCalledTimes(1);
});
it("disabled external actions do not invoke adapters", () => {
  render(<ExternalActions request={request} enabled={false} />);
  expect(
    (screen.getByRole("button", { name: "Email Report" }) as HTMLButtonElement)
      .disabled,
  ).toBe(true);
  expect(integrations.prepare).not.toHaveBeenCalled();
});
it("loads disconnected settings and handles native selection cancellation", async () => {
  vi.mocked(integrations.connect).mockResolvedValue(null);
  render(<IntegrationSettings enabled />);
  await waitFor(() => expect(integrations.status).toHaveBeenCalled());
  fireEvent.click(screen.getAllByRole("button", { name: "Connect" })[0]!);
  expect(
    await screen.findByText("Client-file selection canceled."),
  ).toBeTruthy();
  expect(integrations.connect).toHaveBeenCalledWith("sheets");
});
it("updates connected account after successful connection abstraction", async () => {
  vi.mocked(integrations.connect).mockResolvedValue([
    {
      service: "sheets",
      connected: true,
      account: "test@example.invalid",
      message: "Configured",
    },
  ]);
  render(<IntegrationSettings enabled />);
  fireEvent.click(
    (await screen.findAllByRole("button", { name: "Connect" }))[0]!,
  );
  expect(await screen.findByText("test@example.invalid")).toBeTruthy();
  expect(
    screen.getByRole("button", { name: "Change account / Reconnect" }),
  ).toBeTruthy();
});
it("reports revocation limitations and clears account status", async () => {
  vi.mocked(integrations.disconnect).mockResolvedValue({
    message: "Local credential removed; remote revocation pending",
    destination: null,
  });
  render(<IntegrationSettings enabled />);
  fireEvent.click(
    (
      await screen.findAllByRole("button", {
        name: "Disconnect / Clear credential",
      })
    )[1]!,
  );
  expect(await screen.findByText(/remote revocation pending/u)).toBeTruthy();
  expect(integrations.disconnect).toHaveBeenCalledWith("email");
});
