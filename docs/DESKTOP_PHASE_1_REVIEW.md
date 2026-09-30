# Desktop Phase 1 implementation review

> Historical foundation report. Native prerequisites are now installed and verification is complete as described in DESKTOP_NATIVE_ACCEPTANCE.md. The original blocked results below are retained as historical evidence.

Date: September 29, 2026. Scope: desktop foundation only. Implementation is prepared for review; **Phase 1 native acceptance is incomplete** because Microsoft C++ Build Tools/Windows SDK are missing. No Phase 2 work, business schema, calculation, export, backup/restore, external integration, installer or publishing has started.

## Implemented

Tauri 2 startup/configuration, React/Vite frontend, strict TypeScript (including unchecked index and exact optional-property checks), Tailwind 4 with source scanning restricted to src, local fonts/assets, desktop operations shell, responsive navigation and seven placeholder areas. Reusable Navigation, Placeholder and Settings components separate shell orchestration from presentation. Light/dark/system themes follow Windows in system mode and persist through a Rust-owned settings command. Failed saves preserve the previous theme; startup storage failures show an error and disable settings writes instead of resetting data.

The previous Next.js/Supabase source, configurations, dependency manifests and tests are preserved under legacy/web and excluded from the desktop build. Root scripts, TypeScript includes, Vite entry, Tailwind scanning and dependencies use only desktop code. Historical web documentation is archived there; root README/dependency/review documents describe desktop status.

## Architecture and storage

- Fixed identifier com.twx.payroll; Tauri app_local_data_dir resolves per-user Windows LocalAppData. Append data/twx-payroll.sqlite3; no username, Documents, install directory or frontend-provided path.
- rusqlite 0.40.2 bundles SQLite 3.53.2 source through libsqlite3-sys 0.38.2. Actual desktop runtime SQLite version is not yet verified.
- Only app_settings exists: key TEXT PK NOT NULL CHECK key='theme'; value TEXT NOT NULL CHECK system/light/dark; updated_at TEXT NOT NULL UTC timestamp default. STRICT table; initial preference system. No business precision decisions or financial fields.
- One private connection behind a Rust mutex, five-second busy timeout, foreign_keys ON, WAL, synchronous FULL, trusted_schema OFF. SQLite extension loading is disabled by default; no enabling API is exposed. Single-instance Rust plugin focuses the existing main window.
- rusqlite_migration 2.6.0 owns transactional ordered migrations and PRAGMA user_version. SQL is embedded with include_str!. No redundant schema_migrations/metadata/checksum table. Unknown future schemas are rejected before journal/migration changes. Existing data is not automatically replaced after failure.
- Applied migrations are immutable source files. New changes require new ordered migrations. This framework does not validate historical applied-file checksums at runtime; source review and lockfiles supply reproducibility. No manual metadata mechanism is added.
- Rust storage tests cover fresh/idempotent migration, restart persistence, constraints, WAL/FK configuration, stable suffix, rejected future schema and rollback after invalid migration. These tests exist but could not execute due to the missing linker.
- Backend async commands dispatch off the UI thread and serialize settings operations. Their only inputs are validated theme enum values. The frontend uses a typed host adapter; browser preview has explicit session-only appearance and no fake desktop persistence.

## Capabilities and theme

Only three permissions are granted to local window main: allow-get-app-info, allow-get-preferences, allow-set-theme-preference. No remote origins, broad core defaults, arbitrary SQL/filesystem/path commands, shell plugin or frontend SQL plugin. Production CSP permits bundled resources and Tauri IPC; development adds only the local Vite/HMR endpoint. Installer bundling is disabled. Runtime rejection of unauthorized IPC is still pending a real desktop launch; configuration assertions passed.

Theme choices are system/light/dark. Settings save in a transaction before UI success/application of a new preference. System mode uses matchMedia and responds to Windows changes; manual choices override those changes. Initial shell content waits for preferences; loading failure uses system fallback with error and disabled writes. No localStorage is used. Keyboard focus, skip navigation, accessible theme button labels and current-navigation markers are included. Both light and dark contrast were checked with axe, and light muted text was darkened to fix a real contrast failure.

## Structure and materially changed files

```
src/app/App.tsx                 shell, loading/errors/theme orchestration
src/app/App.test.tsx            five host-boundary/theme/error tests
src/components/Navigation.tsx  reusable navigation and area definitions
src/features/Placeholder.tsx   shared planned-area screens
src/features/settings/Settings.tsx  appearance and read-only app/storage info
src/contracts/desktop.ts       three typed IPC calls
src/main.tsx, src/styles.css    local React entry and responsive theme styles
src-tauri/src/lib.rs            startup, data resolver, single instance
src-tauri/src/commands.rs       narrow commands and serialized state
src-tauri/src/storage.rs        connection/migration/settings plus Rust tests
src-tauri/migrations/001_settings.sql  only Phase 1 table
src-tauri/capabilities/main.json       exact local permissions
src-tauri/{Cargo.toml,Cargo.lock,build.rs,tauri.conf.json}
src-tauri/src/main.rs, src-tauri/icons/*
scripts/verify-{foundation,ui}.mjs, scripts/verify-native.ps1
package.json, package-lock.json, rust-toolchain.toml
tsconfig.json, vite.config.ts, eslint.config.mjs, index.html, .gitignore
AGENTS.md, ARCHITECTURE.md, README.md
docs/{PRODUCT_SPEC,IMPLEMENTATION_PLAN,DEPENDENCIES,DESKTOP_PHASE_1_REVIEW}.md
legacy/web/*                   isolated superseded scaffold/documents
```

The source archive contains current desktop source and lockfiles, excluding dependencies, toolchains, native targets, generated output and obsolete web source. Full superseded code remains in the workspace.

## Exact major versions

Tauri Rust/API/CLI 2.12.0; tauri-build 2.7.0; single-instance 2.5.1; React/React DOM 19.3.0; TypeScript 6.0.3; Vite 8.3.1; Vite React plugin 6.1.1; Tailwind/Vite plugin 4.3.3; Rust/Cargo 1.98.1; rusqlite 0.40.2; rusqlite_migration 2.6.0; serde 1.0.229; tempfile 3.27.0. ESLint 10.11.0; typescript-eslint 8.71.0; Vitest 5.0.2; jsdom 29.1.1; Testing Library React 16.3.3; Playwright 1.63.0; axe Playwright 4.13.0; Prettier 3.9.9. Complete direct versions are in docs/DEPENDENCIES.md and both lockfiles freeze transitive dependencies.

## Verification results

| Command/check | Result |
|---|---|
| npm install (workspace npm cache) / npm install --package-lock-only | Passed; lockfile generated. Initial default cache write was denied; workspace cache resolved it. Latest jsdom 30.1.1 warned about unsupported Node patch; replaced with compatible 29.1.1. Final installs have no engine warnings. |
| npm run typecheck | Passed strict TypeScript. |
| npm run lint | Passed ESLint for frontend and Vite config. |
| npm test | Passed five Vitest tests and foundation checks. First run had three theme button accessible-name failures; explicit labels fixed them. |
| npm run test:foundation / node scripts/verify-foundation.mjs | Passed configuration/capability/local-build assertions and actual settings SQL/constraints on disposable in-memory Node SQLite. Not a Rust runner test. |
| npm run build | Passed strict TS and production Vite build; approximately 227 kB JS / 71 kB gzip. |
| npm run test:ui / node scripts/verify-ui.mjs | Passed Edge browser tests at 1440x900, 1024x768, 800x600, 375x700: every navigation area, no page-wide overflow, manual/system themes, OS theme changes, keyboard theme activation and light/dark axe WCAG checks. Screenshots saved. First axe invocation required explicit browser context; fixed. Light contrast initially failed and was corrected; no suppression. |
| npm run format:check | Passed Prettier on frontend/config/scripts. |
| npm run verify:frontend | Entire combined suite passed after fixes. |
| npm audit --omit=dev; npm install full audit | Zero known vulnerabilities reported. |
| rustup toolchain install 1.98.1 --profile minimal --component rustfmt --component clippy | Passed workspace-local toolchain installation; no global PATH change. |
| cargo fmt --manifest-path src-tauri/Cargo.toml; cargo fmt --manifest-path src-tauri/Cargo.toml -- --check | Formatted and passed final check. |
| cargo generate-lockfile --manifest-path src-tauri/Cargo.toml | Initial Windows Schannel TLS failure. Retried using a temporary local Node HTTPS relay with TLS validation enabled and Cargo checksum verification; succeeded, 464 locked packages. No relay/config is part of application source or runtime. |
| cargo metadata --manifest-path src-tauri/Cargo.toml --locked --no-deps --format-version 1 | Passed manifest/dependency metadata through temporary relay. |
| cargo check --manifest-path src-tauri/Cargo.toml --locked (also --offline after caching) | Failed: linker link.exe not found. Application Rust type correctness remains unverified. |
| cargo test --manifest-path src-tauri/Cargo.toml --locked (also --offline after caching) | Failed before test execution: missing link.exe. Initial offline attempt needed tempfile; cached and retried. No Rust test is claimed passed. |
| cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings (also --offline after caching) | Failed: missing link.exe. Initial registry override forwarding was fixed before retry. No warning suppression. |
| npm run tauri -- info | Ran successfully as diagnostics: Rust 1.98.1 and WebView2 154.0.4258.37 present; MSVC/Windows SDK absent. Environment is not fully ready. |
| Tauri build --no-bundle -- --locked | Frontend beforeBuildCommand passed; native production compilation failed: link.exe absent. Invoked via node work/cargo-relay.mjs --tauri build --no-bundle -- --locked to avoid environment TLS issue. No executable/installer built. |
| npm run tauri -- icon work/app-icon.svg --output src-tauri/icons | Generated local brand icons successfully; Windows font scanner warned about malformed unrelated mstmc.ttf. Icon contains paths, no font text. |
| git add package-lock.json src-tauri/Cargo.lock | Failed: permission denied creating .git/index.lock. Both lockfiles exist but are uncommitted. No commit is claimed. |
| Real Tauri dev launch / offline restart / single-instance / IPC denial / Windows DPI smoke tests | Not run; blocked by missing native build prerequisites. Browser checks do not establish these results. |

Node emits an ExperimentalWarning for its SQLite test module; production uses Rust rusqlite, not Node SQLite. No warnings/failures were hidden or converted into claimed passes.

## Review before Phase 2

Install/configure Microsoft C++ Build Tools and Windows SDK in a developer environment, then run scripts/verify-native.ps1 and real Tauri development/runtime smoke checks. Fix any Rust/Tauri errors found before marking Phase 1 accepted. Commit the generated lockfiles once repository index access is available. Signing is not a prerequisite for this development work; retain signing/release continuity requirements for the later distribution phase.

Confirm the stable application ID/data ownership and UI direction. Business quantity precision, rate terminology, ACH, percentage behavior, weights/tons, settlement formula, payroll linkage, relationships, date rules and rounding remain unresolved in PRODUCT_SPEC.md. No approved financial storage scale or formulas were added. Proposed next phase is reviewed SQLite-safe backup/recovery infrastructure, pending Q16 approval; business data and financial phases require separate authorization and rule resolution. Never design backup as a blind active-database file copy.

STOP: Phase 2 has not begun.
