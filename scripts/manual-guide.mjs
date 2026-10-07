import { readFileSync, writeFileSync } from "node:fs";
import assert from "node:assert/strict";
const sections = JSON.parse(
  readFileSync("src/features/manual/sections.json", "utf8"),
);
const text =
  "# TWX Payroll — User Guide\n\nBundled offline guide for release candidate 0.2.0. The in-app User Manual uses the same content.\n\n" +
  sections
    .map((s) => `## ${s.title}\n\n${s.paragraphs.join("\n\n")}\n`)
    .join("\n");
if (process.argv.includes("--check")) {
  assert.equal(readFileSync("USER_GUIDE.md", "utf8"), text);
  console.log("PASS offline manual and USER_GUIDE alignment");
} else writeFileSync("USER_GUIDE.md", text);
