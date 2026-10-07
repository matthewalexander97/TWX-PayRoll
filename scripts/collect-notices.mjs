// Run cargo metadata --locked --format-version 1 --filter-platform x86_64-pc-windows-msvc first.
import fs from "node:fs";
import path from "node:path";
const metadata = JSON.parse(
  fs.readFileSync(process.argv[2], "utf8").replace(/^\uFEFF/u, ""),
);
const nodes = new Map(metadata.resolve.nodes.map((n) => [n.id, n]));
const selected = new Set();
function visit(id) {
  if (selected.has(id)) return;
  selected.add(id);
  for (const d of nodes.get(id)?.deps ?? [])
    if (d.dep_kinds.some((k) => k.kind === null)) visit(d.pkg);
}
visit(metadata.resolve.root);
const sections = [
  "TWX Payroll third-party notices\nGenerated from the locked Windows runtime dependency graph; original notices follow. The application itself has no new open-source license grant implied by these dependency notices.\n",
];
const missing = [];
function notices(root, depth = 0) {
  if (depth > 2) return [];
  return fs.readdirSync(root, { withFileTypes: true }).flatMap((e) => {
    const p = path.join(root, e.name);
    if (
      e.isDirectory() &&
      !["target", "node_modules", ".git", "tests"].includes(e.name)
    )
      return notices(p, depth + 1);
    return e.isFile() &&
      /^(licen[cs]e|copying|copyright|notice|unlicense)([.-]|$)/iu.test(
        e.name,
      ) &&
      fs.statSync(p).size < 1000000
      ? [p]
      : [];
  });
}
function add(name, version, license, root, explicit) {
  const files = notices(root);
  if (
    explicit &&
    fs.existsSync(path.join(root, explicit)) &&
    !files.includes(path.join(root, explicit))
  )
    files.push(path.join(root, explicit));
  sections.push(
    `\n${"=".repeat(72)}\n${name} ${version}\nDeclared license: ${license ?? "see source"}\n`,
  );
  const supplemental = fs.existsSync("third-party/notices")
    ? fs
        .readdirSync("third-party/notices")
        .filter((f) => f.startsWith(`${name}-`))
    : [];
  if (!files.length && !supplemental.length) missing.push(`${name}@${version}`);
  for (const file of supplemental)
    sections.push(
      fs.readFileSync(path.join("third-party/notices", file), "utf8"),
    );
  for (const file of files)
    sections.push(
      `\n--- ${path.relative(root, file)} ---\n${fs.readFileSync(file, "utf8")}\n`,
    );
}
for (const p of metadata.packages)
  if (selected.has(p.id) && p.source)
    add(
      p.name,
      p.version,
      p.license,
      path.dirname(p.manifest_path),
      p.license_file,
    );
const lock = JSON.parse(fs.readFileSync("package-lock.json", "utf8"));
for (const [p, entry] of Object.entries(lock.packages))
  if (p && !entry.dev) {
    const pkg = JSON.parse(
      fs.readFileSync(path.join(p, "package.json"), "utf8"),
    );
    add(pkg.name, pkg.version, pkg.license, p);
  }
sections.push(
  "\nNoto Sans font\n" +
    fs.readFileSync("src-tauri/assets/OFL-NotoSans.txt", "utf8"),
);
sections.push(
  "\nNSIS installer notices\n" +
    fs.readFileSync("src-tauri/target/.tauri/NSIS/COPYING", "utf8"),
);
for (const file of fs
  .readdirSync("third-party/notices")
  .filter((f) => f.startsWith("nsis-tauri-utils-")))
  sections.push(
    fs.readFileSync(path.join("third-party/notices", file), "utf8"),
  );
sections.push(
  "\nMicrosoft Edge WebView2 Evergreen Runtime is Microsoft software distributed through its unmodified offline installer. Microsoft terms and runtime updates apply. https://developer.microsoft.com/microsoft-edge/webview2/\n",
);
fs.writeFileSync("THIRD_PARTY_NOTICES.txt", sections.join("\n"));
console.log(
  JSON.stringify({ runtimeCrates: selected.size - 1, missingNotices: missing }),
);
if (missing.length) process.exitCode = 1;
