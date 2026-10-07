import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

const read = (path) => readFileSync(path, "utf8");
const config = JSON.parse(read("src-tauri/tauri.conf.json"));
assert.equal(config.identifier, "com.twx.payroll");
assert.equal(config.bundle.windows.nsis.installMode, "currentUser");
assert.equal(config.bundle.windows.nsis.template, "installer/installer.nsi");
assert.equal(config.bundle.windows.nsis.installerHooks, undefined);
assert.equal(config.bundle.windows.allowDowngrades, false);
assert.equal(config.bundle.windows.webviewInstallMode.type, "offlineInstaller");
const files = ["src-tauri/installer/installer.nsi"];
if (process.argv.includes("--generated"))
  files.push("src-tauri/target/release/nsis/x64/installer.nsi");
for (const file of files) {
  const text = read(file);
  assert.doesNotMatch(text, /DeleteAppDataCheckbox|\$\(deleteAppData\)/i);
  // A future upstream change must not reintroduce recursive workspace deletion.
  assert.doesNotMatch(text, /^\s*RmDir\s+\/r(?:\s|$)/im);
  const start = text.search(/^Section Uninstall\s*$/m);
  assert.ok(start >= 0, "Uninstall section must exist");
  const uninstall = text.slice(start);
  assert.ok(uninstall.length > 0 && uninstall.length < text.length);
  assert.doesNotMatch(
    uninstall,
    /^\s*(?:Delete|RmDir)\b[^\n]*\$(?:LOCALAPPDATA|APPDATA)/im,
  );
  assert.match(
    text,
    /Your payroll database, settings and recovery files will remain/,
  );
  assert.match(uninstall, /Delete "\$INSTDIR\\\$\{MAINBINARYNAME\}\.exe"/);
  assert.match(uninstall, /Delete "\$INSTDIR\\uninstall\.exe"/);
  assert.match(uninstall, /DeleteRegKey HKCU "\$\{UNINSTKEY\}"/);
  console.log(`PASS data-preserving uninstall policy: ${file}`);
}
