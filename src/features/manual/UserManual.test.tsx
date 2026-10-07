import { cleanup, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, expect, it, vi } from "vitest";
import { UserManual } from "./UserManual";
import sections from "./sections.json";
afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});
it("renders every section from bundled content with no network requests", () => {
  const fetch = vi.fn();
  vi.stubGlobal("fetch", fetch);
  render(<UserManual />);
  expect(sections).toHaveLength(14);
  for (const section of sections) {
    expect(screen.getByRole("heading", { name: section.title })).toBeTruthy();
    expect(
      screen.getByRole("link", { name: section.title }).getAttribute("href"),
    ).toBe(`#manual-${section.id}`);
  }
  expect(fetch).not.toHaveBeenCalled();
  expect(document.querySelector('a[href^="http"]')).toBeNull();
});
it("section links and return links support keyboard navigation", async () => {
  render(<UserManual />);
  const user = userEvent.setup();
  await user.tab();
  expect(document.activeElement).toBe(
    screen.getByRole("link", { name: "Getting Started" }),
  );
  await user.tab();
  expect(document.activeElement).toBe(
    screen.getByRole("link", { name: "Initial Setup" }),
  );
  const link = screen.getAllByRole("link", {
    name: "Back to section list",
  })[0]!;
  link.focus();
  await user.keyboard("{Enter}");
  expect(document.activeElement).toBe(
    screen.getByRole("link", { name: "Getting Started" }),
  );
});
