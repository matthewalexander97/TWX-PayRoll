# Independent source maintenance

The installed app does not need the repository or development tools. To build source on Windows x64, install Node 24.13.0/npm 11.6.2 (or a supported compatible Node 24 release), Rust 1.98.1 with rustfmt/clippy and the x86_64-pc-windows-msvc target, Microsoft C++ Build Tools with Desktop development with C++, and a supported Windows SDK. The checked rust-toolchain.toml pins Rust. WebView2 Runtime is needed for local runtime testing. Dependency versions are recorded in DEPENDENCIES.md; npm and Cargo lockfiles freeze the graph.

From a normal developer PowerShell at the repository root:

```powershell
npm ci
npm run verify:frontend
pwsh -NoProfile -File scripts/verify-native.ps1
npm run tauri -- dev
npm run build:installer
npm run handoff
```

The first installer build downloads Tauri's NSIS tools and Microsoft's offline Evergreen WebView2 installer. Keep HTTPS certificate validation enabled. The ignored work/cargo-relay helper is specific to this development host's cached registry/TLS environment, is not an application dependency, and is not required on a normal developer installation. Do not copy that cache into the handoff.

Build outputs: dist (bundled frontend), src-tauri/target/release/twx-payroll.exe and src-tauri/target/release/bundle/nsis/*-setup.exe. The handoff script copies a strict allowlist into outputs/release/0.2.0 with SHA256SUMS.txt. It excludes source credentials, databases, private reports and logs. outputs is ignored by Git; deliver it separately after review.

Ordered immutable SQL migrations live in src-tauri/migrations; storage.rs embeds them through rusqlite_migration/PRAGMA user_version. Schema 4 remains current. Never edit applied migrations or reset the database to upgrade. Financial rules live in domain.rs and application.rs. UI does not calculate authoritative money. Production identifier is com.twx.payroll; live data uses Tauri's app_local_data_dir()/data/twx-payroll.sqlite3. Export preference is an independent machine-local export-directory.json. Backups use the SQLite Online Backup API, never a raw copy of an active file.

legacy/web is superseded and excluded from desktop builds. Do not run its scripts as verification. Narrow named commands are declared in build.rs, commands.rs, lib.rs and capabilities/main.json. No generic filesystem, shell, SQL or remote-origin grants belong in this app.

Before committing manually, inspect staged files for credentials and generated data. This environment cannot write .git/index.lock; do not change permissions to bypass that restriction. Recommended commands are provided in the Phase 7 review. No commit/push is implied by build success.

## Final release validation (0.2.0)

The data-preserving NSIS template is src-tauri/installer/installer.nsi, derived from the pinned CLI's upstream template with provenance in the adjacent README. It removes both the app-data checkbox and deletion code. npm test checks the source policy; npm run build:installer checks the generated script after compilation; npm run handoff refuses a generated installer script that violates it. Retain this customization when updating Tauri. Runtime uninstall/reinstall acceptance is a separate required check.

Current verification and artifact hashes are in RELEASE_VALIDATION.md. DESKTOP_PHASE_7_REVIEW.md retains historical candidate hashes; use the current SHA256SUMS.txt when validating the delivered files.
