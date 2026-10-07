# TWX Payroll desktop

Desktop Phases 1–5 are accepted. Phase 6 adds optional customer-owned Google Sheets/Gmail report outputs for review. Local bookkeeping, recovery and CSV/XLSX/PDF/DOCX/print remain usable offline. [Integration setup/lifecycle](docs/INTEGRATIONS_GUIDE.md), [Phase 6 review](docs/DESKTOP_PHASE_6_REVIEW.md). Synthetic data only pending business acceptance. Routing, installer/signing, distribution and deployment require separate approval.

## Development prerequisites

Use Node 24 LTS (verified 24.13.0), npm (verified 11.6.2), pinned Rust 1.98.1, Microsoft C++ Build Tools with Desktop development with C++ and Windows SDK, and WebView2. Rust is pinned in rust-toolchain.toml. A signing certificate is not needed for Phase 1. Customer-owned signing/release continuity and offline WebView2 provisioning remain distribution decisions.

```powershell
npm.cmd ci
npm.cmd run tauri -- dev
```

Browser-only UI preview: npm.cmd run dev, then http://127.0.0.1:1420. This preview cannot access SQLite and explicitly states that theme changes last only for the session. It is not desktop runtime verification.

## Verification

```powershell
npm.cmd run verify:frontend
pwsh -File scripts/verify-native.ps1
```

Frontend verification runs strict TS, ESLint, Vitest tests, foundation configuration/SQL checks, production Vite build, Edge/Playwright resizing/navigation/theme/keyboard and axe accessibility checks, and formatting checks. Edge must be installed for test:ui; screenshots go to outputs/desktop-phase-1. The SQL check uses Node's experimental SQLite module against an in-memory database; it does not substitute for Rust migration tests. Native verification runs cargo fmt/check/test/clippy and Tauri build --no-bundle. Also smoke-test real native selection, backup history, deliberate restore, refreshed state and restart persistence. Historical Phase 1 runtime/DPI coverage is recorded in docs/DESKTOP_NATIVE_ACCEPTANCE.md.

## Structure

- src/app: shell orchestration and tests
- src/components: responsive navigation
- src/features: references, payroll, settlements, saved reports/local exports, optional external report actions and appearance/backup/integration settings
- src/contracts: typed desktop IPC adapter, no generic SQL or files
- src-tauri/src: startup, narrow commands, SQLite connection/settings, exact domain, catalog/history repositories and financial application services
- src-tauri/migrations: immutable embedded SQL migrations
- src-tauri/capabilities: local main-window permissions only
- scripts: frontend SQL/config/UI and native verification
- docs: architecture, requirements, phase gates and verification report
- legacy/web: isolated historical Next.js/Supabase scaffold; excluded from Vite/TypeScript/Tailwind and desktop dependency graph

## Local storage

Stable application ID: com.twx.payroll. Rust resolves app.path().app_local_data_dir() and appends data/twx-payroll.sqlite3 (Windows LocalAppData); no hardcoded username or Documents/install path. Schema version 4 has 16 tables, including backup_settings and backup_runs, and adds optional informational loads.mileage. Migrations 001–003 are unchanged. The frontend supplies no filesystem path or SQL. There is no localStorage business or theme persistence.

One connection is serialized behind a Rust mutex; single-instance startup prevents a second ordinary app writer. Connections enable foreign keys, WAL, synchronous FULL, trusted_schema OFF and a five-second busy timeout. Extension loading is disabled by default and no API enables it. rusqlite bundles SQLite. rusqlite_migration embeds ordered SQL, manages transactional migrations and PRAGMA user_version; there is no hand-maintained metadata table. Unknown future schema versions fail before migrations; existing data is never silently reset. Applied SQL files must remain immutable; this runner has no applied-file checksum detection. Add new migrations instead.

Thirty-one named commands exist: ten appearance/recovery, eleven business/history, three saved local reports and seven optional integration lifecycle/confirmation commands. Rust validates all privileged writes, exact financial inputs, expected versions and negative-final confirmation. Native Windows pickers own file/client selection; restore and external report sends use opaque one-use tokens and confirmation. Capabilities authorize only these commands for local window main. No remote IPC, broad core permissions, filesystem, SQL plugin, arbitrary network or shell permissions are granted. Production CSP permits bundled resources and Tauri IPC only; development adds the local Vite/HMR endpoint. Optional provider HTTPS is Rust-owned. Bundling/distribution is disabled.

Appearance follows Windows by default, responds to OS changes in system mode, and remembers manual choices in SQLite. The shell waits for preference loading; failures show an error and disable settings writes. Failed theme saves preserve the previous preference. CSS uses local system fonts and no remote assets.

Financial rules use exact Rust i128 intermediates and INTEGER cents, tons_milli, completion_rate_scaled and payout_rate_ppm. PRODUCT_SPEC.md defines the provisional per-line and percentage half-up rounding policy. Financial IPC values are strings and the frontend only formats digit strings; Rust returns every calculated total. USD ($) is the approved display currency. Finalized edits and linked DRIVER payroll updates preserve before/after audits atomically; OWNER payroll is manual with no ACH field. Negative final payout requires explicit confirmation; there is no payment execution.

The reproducible native business smoke script is scripts/verify-business-native.mjs. It refuses to write unless the running app uses the separate com.twx.payroll.phase4-smoke database. Its temporary Tauri/WebView2 diagnostic config and debugging port are used only during verification, under ignored work, and are absent from the ordinary build. See the Phase 4 review for initial/restart/resume commands and all 18 workflow results.

Backups use SQLite Online Backup API, validated schema/integrity/foreign keys and SHA-256 completion manifests. Select a backup folder in Settings. Automatic backups evaluate at startup and every minute while open: changed business data is saved once the prior successful backup is 24 hours old, with hourly failure retries. Keep 10 automatic generations; manual and pre-restore copies are never automatically removed. Same-drive copies cannot protect against disk failure. Restore stages and validates the candidate, preserves a safety snapshot, serializes access and rolls back on validation failure. See docs/RECOVERY_GUIDE.md. No next phase starts automatically.

## Phase 7 release candidate

Version 0.2.0 adds export-folder settings, offline User Manual and per-user NSIS packaging with offline WebView2. See USER_GUIDE.md, DATA_RECOVERY.md, RELEASE_NOTES.md, RELEASE_CHECKLIST.md and docs/SOURCE_HANDOFF.md. Full commands: `npm run verify:frontend`, `pwsh -File scripts/verify-native.ps1`, `npm run build:installer`, `npm run handoff`. Schema/financial rules remain unchanged. Independent installed-PC acceptance is pending because no separate Windows test environment is available. No public release or Git commit/push is implied.
