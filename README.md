# TWX Payroll desktop foundation

Desktop Phase 1 implementation only: Tauri 2, React, strict TypeScript, Tailwind, responsive placeholders and remembered local appearance. Native compiler/build checks and desktop runtime smoke tests pass; see docs/DESKTOP_NATIVE_ACCEPTANCE.md for evidence and limitations. No business tables, calculations, exports, recovery features, integrations, installer or deployment are implemented.

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

Frontend verification runs strict TS, ESLint, five Vitest tests, foundation configuration/SQL checks, production Vite build, Edge/Playwright resizing/navigation/theme/keyboard and axe accessibility checks, and formatting checks. Edge must be installed for test:ui; screenshots go to outputs/desktop-phase-1. The SQL check uses Node's experimental SQLite module against an in-memory database; it does not substitute for Rust migration tests. Native verification runs cargo fmt/check/test/clippy and Tauri build --no-bundle. Also smoke-test the real app manually: restart theme persistence, stable resolved data location, single-instance focus, window resize/DPI, offline launch and restricted IPC. Native runtime checks have now run; see docs/DESKTOP_NATIVE_ACCEPTANCE.md for exact results and DPI coverage limits.

## Structure

- src/app: shell orchestration and tests
- src/components: responsive navigation
- src/features: shared placeholder and appearance/settings views
- src/contracts: typed desktop IPC adapter, no generic SQL or files
- src-tauri/src: startup, narrow commands, SQLite connection/settings
- src-tauri/migrations: immutable embedded SQL migrations
- src-tauri/capabilities: local main-window permissions only
- scripts: frontend SQL/config/UI and native verification
- docs: architecture, requirements, phase gates and verification report
- legacy/web: isolated historical Next.js/Supabase scaffold; excluded from Vite/TypeScript/Tailwind and desktop dependency graph

## Local storage

Stable application ID: com.twx.payroll. Rust resolves app.path().app_local_data_dir() and appends data/twx-payroll.sqlite3 (Windows LocalAppData); no hardcoded username or Documents/install path. Only app_settings exists: theme key, checked system/light/dark value, UTC updated_at timestamp, STRICT constraints. The frontend supplies no filesystem path or SQL. There is no localStorage business or theme persistence.

One connection is serialized behind a Rust mutex; single-instance startup prevents a second ordinary app writer. Connections enable foreign keys, WAL, synchronous FULL, trusted_schema OFF and a five-second busy timeout. Extension loading is disabled by default and no API enables it. rusqlite bundles SQLite. rusqlite_migration embeds ordered SQL, manages transactional migrations and PRAGMA user_version; there is no hand-maintained metadata table. Unknown future schema versions fail before migrations; existing data is never silently reset. Applied SQL files must remain immutable; this runner has no applied-file checksum detection. Add new migrations instead.

Three named commands exist: get_app_info, get_preferences, set_theme_preference. Rust validates themes as an enum and commits settings before reporting success. Capabilities authorize only those commands for local window main. No remote IPC, broad core permissions, filesystem, SQL plugin or shell permissions are granted. Production CSP permits bundled resources and Tauri IPC only; development adds the local Vite/HMR endpoint. Bundling/distribution is disabled.

Appearance follows Windows by default, responds to OS changes in system mode, and remembers manual choices in SQLite. The shell waits for preference loading; failures show an error and disable settings writes. Failed theme saves preserve the previous preference. CSS uses local system fonts and no remote assets.

Money/calculations must never use JavaScript floating-point Number. Business precision, rate terminology, rounding points and all financial rules remain unresolved. Backup/restore belongs to the next reviewed phase and must use SQLite-safe snapshots, never blind copies of an active WAL database. No later phase starts automatically.
