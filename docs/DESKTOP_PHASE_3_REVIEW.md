# Desktop Phase 3 — backup and recovery review

October 4, 2026. Desktop Phases 1 and 2 were accepted. Phase 3 implements SQLite-safe backup/recovery only and is returned for review. Full business UI, reports/exports, integrations and distribution have not begun. Synthetic fixtures only; no real payroll data was entered.

## Architecture and snapshot mechanism

Rust owns backup policy, database access, Windows native file selection, validation, publication and recovery. The existing single-instance app and serialized Storage mutex coordinate all ordinary access. React receives typed operation/status objects and no arbitrary filesystem or SQL API. Long backup/restore operations run on blocking worker threads. Original financial services and formulas are unchanged.

`backup.rs` uses rusqlite's SQLite Online Backup API, in bounded page steps, to obtain a consistent snapshot including committed WAL contents. It never copies the active main file. A private destination-stage file is validated, switched to standalone DELETE journaling, closed and flushed, then SHA-256 hashed. No-clobber publication produces a timestamp/UUID `.sqlite3` file and adjacent completion manifest. SUCCESS is recorded only after validation/publication; errors remain FAILED/INTERRUPTED. A retention failure is separately recorded as attention needed without hiding a completed snapshot.

Validation opens the database, checks SQLite integrity and foreign keys, verifies the exact embedded schema for its declared version, checks TWX application identity and validates the saved theme. Manifest checks include application, filename, schema, byte length and SHA-256. Versions 1/2 are recognized through their exact legacy schema; version 3 also requires application_id. Newer, corrupt, unrelated or mismatched databases are refused. This protects against corruption and incompatible input; checksums are not signatures or tamper-proof accounting.

## Restore algorithm

Native file selection is validated before changing live records. Rust stages and migrates a separate candidate without modifying the selected source, hashes it, then returns its name/date and an opaque one-use token. The UI requires an acknowledgment and a separate Confirm Restore action; Rust also requires the confirmation boolean and matching token.

After confirmation, Rust validates the staged candidate again, creates a complete PRE_RESTORE safety snapshot and publishes a flushed recovery marker. Under coordinated database access it transfers the candidate into the live database using SQLite Online Backup API, closes/reopens the connection, restores connection pragmas and verifies schema/integrity/foreign keys. It preserves the machine's current backup destination/automatic policy and merges operation history. Business records and appearance come from the snapshot. React reloads saved preferences/application information, avoiding stale theme state.

If post-apply verification fails, the failed candidate remains and the safety snapshot is restored and validated. If rollback or marker cleanup fails, writes are blocked and the marker/copies remain. Startup with a pending marker conservatively restores the verified safety copy before exposing storage and records ROLLED_BACK. Old WAL sidecars are never attached to a replaced main file. Safety copies and failed candidates are not automatically removed. `RECOVERY_GUIDE.md` explains normal undo, restart recovery, and preserving a whole failed workspace before using a fresh app's guided restore, without developer services or database editing.

## Destination, automatic backups and retention

The Windows COM folder picker supplies a Rust-owned canonical folder. The directory must exist, be writable and be outside the application's data directory. The live database remains in application LocalAppData. Backups may be on local/external/already-synchronized storage; no provider APIs are required. A local write does not prove cloud synchronization, and same-drive copies do not protect against drive failure.

Automatic backup defaults to enabled but requires a chosen destination. Until configured, the UI explicitly states that protection is unconfigured. The running app checks at startup and every 60 seconds. It backs up tracked business changes when the last successful backup is at least 24 hours old, or no baseline exists. Failure retries are limited to once per hour. Theme/UI/history changes do not create excessive generations. Restore resets the baseline so a new copy is due. Disabling automatic backups is reflected in the status notice.

Keep 10 successful automatic generations in the current destination. The documented default is not yet a UI setting. Retention deletes only recognized automatic files whose history, naming, manifest and checksum agree. Manual and pre-restore generations, unrelated files and files in former destinations are not automatically deleted. History displays 100 recent operations while preserving older metadata in SQLite. Manual/safety/candidate files can consume disk over time; cleanup remains deliberate.

## Schema and command changes

Migration `003_recovery.sql` adds two STRICT operational tables, bringing the total to 16. `backup_settings` is a singleton holding the selected destination, automatic flag, retention count, dirty/saved generations and epoch success/attempt times. `backup_runs` holds a UUID key, epoch/UTC timestamps, MANUAL/AUTOMATIC/PRE_RESTORE/RESTORE kind, path, status, schema version, integer byte size, checksum, validation and error summary. CHECK constraints bound settings/status/type values; a time index supports history. Business-table change triggers mark committed changes in the same transaction. No manual migration ledger was added; rusqlite_migration owns user_version=3. Migrations 001/002 were not edited in this phase.

Seven named commands were added: get_backup_status, select_backup_folder, backup_now, set_automatic_backup, select_restore_backup, cancel_restore and restore_backup. Alongside the three existing health/theme commands, the local main window has exactly ten allow permissions. No remote-origin, arbitrary filesystem, shell, generic SQL, broad core or dialog-plugin capability was added. Native selection never accepts a frontend path. Invalid/expired tokens and missing confirmation are rejected by Rust. Foundation assertions check the precise capability allowlist and desktop build boundary. Native exploit/remote-origin injection tests were not repeated in Phase 3; historical Phase 1 results are separate evidence.

The Settings panel adds destination selection, Backup Now, last success, automatic toggle, readable operation history, restore preview/confirmation/cancellation and honest errors/busy states. Business navigation remains placeholder-only. Planning documents explicitly say settlement final payout generates/updates DRIVER payroll Amount; OWNER payroll remains manual, with no invented owner settlement workflow.

## Dependencies and native results

Existing major stack versions remain Tauri/API/CLI 2.12.0, React 19.3.0, TypeScript 6.0.3, Tailwind 4.3.3, Vite 8.3.1, Rust/Cargo 1.98.1, rusqlite 0.40.2 and rusqlite_migration 2.6.0. Enable rusqlite backup; use tempfile 3.27.0 at runtime and add direct sha2 0.10.9, uuid 1.26.1 and Windows-only windows 0.62.2, reusing already-locked versions. Cargo.lock was updated; package-lock.json remains unchanged and present. Full exact versions are in DEPENDENCIES.md. No dependency downloads/upgrades were needed.

Actual native SQLite runtime: **3.53.2**. A production Windows executable was produced at `src-tauri/target/release/twx-payroll.exe`; final SHA-256 is `0F3983CBF5B36E69409B993336D8E8B68EE9E698F36415075478261EAF7C7DFC`. No installer, signing, deployment or distribution was performed.

The native Windows smoke test selected `work/phase3-native-backups` using the real folder picker, made a manual snapshot at 21:03:10 UTC, observed SUCCESS in history, changed System to Light through the UI, selected the saved snapshot using the real file picker, and confirmed restore. At 21:06:13 UTC restore succeeded and the UI returned to System. A PRE_RESTORE snapshot retained Light. At 21:06:37 UTC the running automatic scheduler created a new System baseline. Closing/relaunching retained System, the configured destination and all four history records. All database/copy integrity checks returned ok, foreign-key errors were empty, hashes matched, and the pending marker was absent. The native data mutation was the synthetic appearance preference; synthetic business-row restoration and rollback were verified in Rust tests, not through a full business UI. The live business/audit tables remained empty.

Displayed logical database path, username redacted:

    C:\Users\[redacted]\AppData\Local\com.twx.payroll\data\twx-payroll.sqlite3

Read-only canonical-path inspection revealed this development host's Windows package redirection:

    C:\Users\[redacted]\AppData\Local\Packages\OpenAI.Codex_2p2nqsd0c76g0\LocalCache\Local\com.twx.payroll\data\twx-payroll.sqlite3

Both are per-user local application data, outside Documents and sync storage. The application uses Tauri's stable path resolver, not a Codex-specific hardcoded path. Independent launch outside the packaged development host and clean-machine upgrade/recovery must verify the production physical location before distribution. Do not confuse this observed host redirection with a portable release verification.

## Tests and verification commands

Final suites: **35 Rust tests, 10 frontend tests; zero failed or ignored Rust tests**. Thirteen new Rust recovery tests cover initialized snapshots, business rows, committed/uncommitted WAL consistency, unique names/generations, cadence/noise/disabled policy, unconfigured destination, retention/manual preservation, unavailable/non-directory destinations and retry status, corrupt/unrelated/future/FK rejection, confirmed restore/settings/history/safety, forced post-apply rollback, legacy staging migration, pending-marker startup/interrupted history, candidate tampering and Windows exclusive-file-lock rejection. Five new frontend tests cover explicit confirmation/token/refresh, cancellation, honest failure, desktop-only access/no path entry and picker cancellation. Prior migration/financial tests remain included.

| Command or check | Result |
|---|---|
| `npm.cmd run verify:frontend` | PASS: entire frontend chain below |
| `npm run typecheck` / `tsc --noEmit` | PASS |
| `npm run lint` | PASS |
| `npm test` / `vitest run` | PASS: 10 tests, plus foundation checks |
| `node scripts/verify-foundation.mjs` | PASS: exact capabilities, build boundary, CSP, stable identifier and original settings SQL |
| `npm run build` | PASS: strict TypeScript and Vite production build, repeated by native build |
| `npm run test:ui` / `node scripts/verify-ui.mjs` | PASS: browser navigation/theme/layout/accessibility at 1440×900, 1024×768, 800×600 and 375×700 |
| `npm run format:check` | PASS |
| `cargo fmt --manifest-path src-tauri/Cargo.toml` | PASS: formatted implementation and final connection hardening |
| `node work/cargo-relay.mjs test --manifest-path src-tauri/Cargo.toml --locked` | PASS final 35 tests after connection hardening; initial implementation failures corrected as below |
| `node work/cargo-relay.mjs --native` | PASS final full native suite, invokes PowerShell script below |
| `pwsh -NoProfile -File scripts/verify-native.ps1` | PASS via cached workspace environment |
| `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` | PASS |
| `cargo check --manifest-path src-tauri/Cargo.toml --locked` | PASS |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked` | PASS: 35 tests; binary/doc targets also pass |
| `cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings` | PASS, no warning suppression |
| `npm run tauri -- build --no-bundle -- --locked` | PASS optimized Windows executable |
| `node work/phase2-runtime.cjs` | PASS read-only before/after theme, schema, integrity, FK and empty business-table inspection |
| `node work/phase3-runtime.cjs` | PASS read-only logical/canonical paths, snapshot themes, history, integrity/FK, SHA and absent marker |
| `Get-FileHash src-tauri/target/release/twx-payroll.exe` | PASS: final fingerprint above |
| Cargo.lock source inspection | Standard crates.io registry sources; no local relay dependency recorded |
| Native production selection/backup/restore/restart | PASS as detailed above; final connection hardening also passed all native tests/build |

Commands ran with workspace Rust/Cargo 1.98.1 via `work/phase2-env.ps1`, CARGO_NET_OFFLINE=true and the existing ignored cached-registry relay wrapper. These are environment scaffolding, not application runtime dependencies. Standard developers use scripts/verify-native.ps1 directly.

Initial compiler checks found two E0597 temporary query-iterator lifetimes, corrected with explicit local bindings. Initial Windows backup tests exposed AccessDenied from flushing a read-only file handle; snapshot/staged flushes now open read/write. Tests passed after those fixes. Final review explicitly configured foreign keys/trusted_schema/busy timeout on all backup connections and corrected the disabled-policy notice. No failed checks were suppressed. Node's ExperimentalWarning concerns only Node SQLite verification tooling. One exploratory read found no `.prettierignore`; the configured format command passed. UI automation needed fresh captures and a coordinate click for the native Open split button; the actual restore completed successfully.

Physical disk failure/full-disk conditions, sudden power loss, provider upload completion and removable-media failure were not physically injected. Interrupted recovery and rollback are deterministic database tests. This phase did not repeat native DPI/single-instance/remote IPC probes already documented for Phase 1, or perform an installer/clean-machine drill. The implementation plan's clean-machine and physical-failure acceptance items remain unverified; they must not be treated as passed or as permission to enter real payroll data. No Tauri development launch is claimed for Phase 3; the production application was run directly. The final hardened production binary also launched successfully and read the restored System preference and schema 3.

## Materially added or changed files

- Planning/documentation: AGENTS.md, ARCHITECTURE.md, README.md, docs/PRODUCT_SPEC.md, docs/IMPLEMENTATION_PLAN.md, docs/DEPENDENCIES.md, docs/RECOVERY_GUIDE.md and this review.
- Native implementation: src-tauri/src/backup.rs, src-tauri/src/backup/tests.rs, src-tauri/src/dialogs.rs; updated storage.rs, commands.rs and lib.rs.
- Migration/configuration: src-tauri/migrations/003_recovery.sql, Cargo.toml, Cargo.lock, build.rs and capabilities/main.json. Generated named command permission manifests follow build.rs.
- Frontend: src/contracts/desktop.ts, src/features/settings/BackupPanel.tsx, src/features/settings/BackupPanel.test.tsx, src/features/settings/Settings.tsx, src/app/App.tsx, src/app/App.test.tsx and src/styles.css.
- Verification: scripts/verify-foundation.mjs. Ignored work scripts/logs contain local native inspection evidence; outputs/desktop-phase-3 contains review/recovery documents and redacted runtime evidence, not a distributed binary.

## Review before the next phase

Approve the documented 24-hour dirty-data cadence, 10-auto-copy retention and retained manual/safety/candidate behavior. Decide whether a later release needs user-configurable retention or encryption; neither is implied here. Review the host path-redirection finding before independent distribution and preserve the recovery guide with offline release materials. Confirm provisional financial precision/rounding against TWX historical examples before real payroll use. Full business UI, exports/integrations, installation/signing and clean-machine recovery remain later approval gates.

Source and both lockfiles are present but remain uncommitted in the existing repository. The previously encountered Git index write restriction was not bypassed; no commit or GitHub push is claimed. No next phase begins with this report.
