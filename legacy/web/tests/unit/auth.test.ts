import test from "node:test";
import assert from "node:assert/strict";
import { parseLoginInput, safeDestination } from "../../src/lib/auth/input.ts";
import { getSupabaseConfig } from "../../src/lib/supabase/config.ts";

test("login accepts trimmed email but preserves exact password characters", () => {
  const form = new FormData();
  form.set("email", "  bookkeeper@example.test  ");
  form.set("password", "  exact password  ");
  form.set("next", "/settlements");
  assert.deepEqual(parseLoginInput(form), { email: "bookkeeper@example.test", password: "  exact password  ", destination: "/settlements" });
});

test("login rejects missing, malformed, oversized, and non-string credentials", () => {
  for (const [email, password] of [["bad", "x"], ["a@b.test", ""], ["a@b.test", "x".repeat(1025)], ["x".repeat(255) + "@b.test", "x"]]) {
    const form = new FormData(); form.set("email", email); form.set("password", password);
    assert.equal(parseLoginInput(form), null);
  }
  assert.equal(parseLoginInput(new FormData()), null);
  const file = new FormData(); file.set("email", new Blob(["not an email"])); file.set("password", "x");
  assert.equal(parseLoginInput(file), null);
});

test("post-login destinations exclude redirects to arbitrary or external URLs", () => {
  for (const value of ["https://evil.test", "//evil.test", "/\\evil.test", "/login", "/payroll?next=//evil.test", null]) assert.equal(safeDestination(value), "/payroll");
  assert.equal(safeDestination("/directory"), "/directory");
});

test("public Supabase configuration fails closed for placeholders, malformed URLs, and secret keys", () => {
  const originalUrl = process.env.NEXT_PUBLIC_SUPABASE_URL;
  const originalKey = process.env.NEXT_PUBLIC_SUPABASE_PUBLISHABLE_KEY;
  try {
    for (const [url, key] of [["", ""], ["https://your-project.supabase.co", "your-publishable-key"], ["bad", "public"], ["http://external.test", "public"], ["https://project.supabase.co", "sb_secret_test"], ["https://project.supabase.co", `a.${Buffer.from(JSON.stringify({ role: "service_role" })).toString("base64url")}.b`]]) {
      process.env.NEXT_PUBLIC_SUPABASE_URL = url; process.env.NEXT_PUBLIC_SUPABASE_PUBLISHABLE_KEY = key;
      assert.equal(getSupabaseConfig(), null);
    }
    process.env.NEXT_PUBLIC_SUPABASE_URL = "https://project.supabase.co";
    process.env.NEXT_PUBLIC_SUPABASE_PUBLISHABLE_KEY = "sb_publishable_example";
    assert.deepEqual(getSupabaseConfig(), { url: "https://project.supabase.co", key: "sb_publishable_example" });
  } finally {
    if (originalUrl === undefined) delete process.env.NEXT_PUBLIC_SUPABASE_URL; else process.env.NEXT_PUBLIC_SUPABASE_URL = originalUrl;
    if (originalKey === undefined) delete process.env.NEXT_PUBLIC_SUPABASE_PUBLISHABLE_KEY; else process.env.NEXT_PUBLIC_SUPABASE_PUBLISHABLE_KEY = originalKey;
  }
});
