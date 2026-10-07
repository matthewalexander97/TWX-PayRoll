import {
  copyFile,
  mkdir,
  readFile,
  readdir,
  writeFile,
} from "node:fs/promises";
import { createHash } from "node:crypto";
import path from "node:path";
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
execFileSync(
  process.execPath,
  ["scripts/verify-installer.mjs", "--generated"],
  {
    stdio: "inherit",
  },
);
const config = JSON.parse(await readFile("src-tauri/tauri.conf.json", "utf8"));
assert.equal(config.identifier, "com.twx.payroll");
assert.equal(config.app.appDirectoriesOverride, undefined);
assert.ok(config.app.windows.every((w) => !w.additionalBrowserArgs));
const version = config.version;
const destination = path.resolve("outputs/release", version);
await mkdir(destination, { recursive: true });
const files = [
  [
    `src-tauri/target/release/bundle/nsis/TWX Payroll_${version}_x64-setup.exe`,
    `TWX Payroll_${version}_x64-setup.exe`,
  ],
  ["src-tauri/target/release/twx-payroll.exe", "twx-payroll.exe"],
  ...[
    "USER_GUIDE.md",
    "DATA_RECOVERY.md",
    "RELEASE_NOTES.md",
    "RELEASE_CHECKLIST.md",
    "THIRD_PARTY_NOTICES.txt",
  ].map((f) => [f, f]),
  ["docs/RECOVERY_GUIDE.md", "RECOVERY_GUIDE.md"],
  ["docs/INTEGRATIONS_GUIDE.md", "INTEGRATIONS_GUIDE.md"],
  ["docs/SOURCE_HANDOFF.md", "SOURCE_HANDOFF.md"],
  ["docs/DEPENDENCIES.md", "DEPENDENCIES.md"],
  ["docs/DESKTOP_PHASE_7_REVIEW.md", "DESKTOP_PHASE_7_REVIEW.md"],
  ["docs/RELEASE_VALIDATION.md", "RELEASE_VALIDATION.md"],
  ["src-tauri/assets/OFL-NotoSans.txt", "OFL-NotoSans.txt"],
];
for (const file of await readdir("third-party/source"))
  files.push([`third-party/source/${file}`, `third-party-source/${file}`]);
let hashes = "";
for (const [source, name] of files) {
  const target = path.join(destination, name);
  await mkdir(path.dirname(target), { recursive: true });
  await copyFile(source, target);
  hashes += `${createHash("sha256")
    .update(await readFile(target))
    .digest("hex")}  ${name}\n`;
}
await writeFile(path.join(destination, "SHA256SUMS.txt"), hashes);
// Refuse accidental leftovers: the handoff is an allowlist, not a copy of the workspace.
const allowed = new Set([...files.map(([, name]) => name), "SHA256SUMS.txt"]);
async function audit(folder, relative = "") {
  for (const e of await readdir(folder, { withFileTypes: true })) {
    const name = relative + e.name;
    if (e.isDirectory()) await audit(path.join(folder, e.name), name + "/");
    else assert.ok(allowed.has(name), `Unexpected handoff file: ${name}`);
  }
}
await audit(destination);
console.log(
  `Prepared ${files.length} files and SHA256SUMS.txt in ${destination}`,
);
