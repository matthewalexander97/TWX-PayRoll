# Desktop Phase 4 — business UI review

October 5, 2026. Desktop Phases 1–3 are accepted. Phase 4 is implemented and submitted for review; Phase 5 is not started. USD ($) display was explicitly confirmed by the user. Synthetic fixtures only; no reports/exports, integrations, signing, installer, distribution or deployment were implemented.

## Business screens and components

TWX Payroll now has separate Driver Payroll and Owner Payroll areas, custom-period history and driver/truck filters. Driver rows display saved driver/truck labels, dates, Amount, ACH and Total. Driver detail exposes code, ACH and notes, with a read-only settlement Amount. Owner detail accepts a manually entered Amount, owner/truck/code, dates and notes, with no ACH field or owner settlement workflow.

Weekly Settlements / Loads contains searchable history, a settlement header, multiple editable load cards, configurable adjustment cards, Rust calculation summary, save/finalize controls and simple record history. Header fields include driver, truck, saved driver type, explicit payout percentage, dates, notes and status. Load cards include business Load ID, BOL, PO, plant/job, actual driver/truck, Net Weight / Tons, informational mileage, Completion Rate and backend-calculated Total. Adjustment cards provide type, TWX code, explanation, ADD/DEDUCT direction and nonnegative amount.

Drivers, Owners and Trucks have create/edit/archive reference forms. Drivers also manages Driver Types and payout percentages. Settings retains appearance and Backup & recovery and adds TWX Codes, Plants, Jobs and Adjustment Types. Known adjustment-name buttons prefill user-created forms; they do not seed records or infer codes/rates. Archived records remain available for existing selections, but are excluded from new choices. Reference edits preserve existing financial labels/percentages. Reports remains a placeholder.

Shared labelled fields, inline errors, history filters, audit disclosure and responsive forms/cards are in `src/components/BusinessForms.tsx`. Theme tokens, visible keyboard focus, touch-size controls, compact navigation and table-to-card behavior remain reusable. A subtle unconfigured/disabled-backup notice links the operational concern to Settings without blocking entry.

## Exact writes, linkage and validation

React sends decimal strings and string versions. Digit-string formatting displays USD, including negative results and all i64 digits, without converting financial values to JavaScript Number. Rust previews use the existing exact domain functions; no financial formula was duplicated in React. Existing INTEGER scales and centralized provisional per-load/percentage half-up policy are unchanged.

Saving a draft validates inputs and uses the existing application service. Finalize first saves the current form, then finalizes the returned persisted version. These are distinct operations: if finalization fails, the successfully saved draft remains available. Finalization itself calculates, updates snapshots, creates/updates the unique linked DRIVER payroll and appends audits atomically. A finalized save uses the same existing transaction boundary, preserving ACH/code/notes and updating payroll Amount and Total. Before/after history is retained; there is no separate revision-management workflow.

Driver-payroll IPC accepts only ID, expected version, ACH, code and notes; unexpected Amount fields are rejected. Owner-payroll IPC accepts manual Amount and owner fields, never ACH. Rust still enforces driver XOR owner, owner no-ACH and linkage constraints. A negative final payout is preserved, requires explicit UI acknowledgment before finalization/finalized edits and is independently rejected by Rust without confirmation. No payment interpretation/execution is implied.

Rust rejects malformed/nonfinite/exponent input, overprecision, overflow, invalid/reversed dates and stale versions. Structured errors identify fields where possible. React keeps entered form values on failure and offers deliberate reload for stale records. Missing references/constraint failures are reported without partial financial writes. Empty optional mileage is NULL; explicit zero is retained. Header/load identities may differ and remain explicit; a truck default only prefills a new header when no driver is selected.

## Migration, storage and permissions

New immutable migration `004_mileage.sql` adds nullable `loads.mileage` as exact nonnegative decimal TEXT. It has no fixed scale/unit conversion and never affects money. Its 128-character bound limits input size rather than fractional precision. Rust and a SQLite CHECK reject malformed values. Existing historical snapshots are untouched by migration; later saves capture mileage with the aggregate. Migrations 001–003 were not edited. Schema version is 4; table count remains 16. rusqlite_migration still owns PRAGMA user_version with no extra ledger.

Backup validation recognizes exact versions 1–4, with application identity required for versions 3/4. A new test restores a real version-3 business snapshot into version 4, preserves the original file byte-for-byte and retains its owner. Existing WAL, rollback, interruption, retention and older-schema tests still pass.

Eleven new commands are registered through build.rs and the local main-window capability:

- get_reference_data, save_reference_record
- list_settlement_records, get_settlement_record, preview_settlement
- save_settlement_record, finalize_settlement_record
- list_payroll_records, save_driver_payroll, save_owner_payroll
- get_record_history

Together with the ten existing appearance/recovery commands there are 21 permissions. No arbitrary SQL, frontend file paths, filesystem/shell plugins, remote-origin IPC or broad core permissions were added. All writes share the Rust serialized connection; Rust generates UUIDs. Fixed enums choose reference tables/entities; query values are parameterized. Ordinary production config has no debugging browser arguments. Legacy web sources remain excluded from the desktop dependency/build graph.

No dependency versions/features changed. Generated package-lock.json and Cargo.lock remain present and repository-ready. Major versions: Tauri/API/CLI 2.12.0; React/React DOM 19.3.0; TypeScript 6.0.3; Tailwind 4.3.3; Vite 8.3.1; Vitest 5.0.2; Rust/Cargo 1.98.1; rusqlite 0.40.2; rusqlite_migration 2.6.0. Full exact direct versions remain in DEPENDENCIES.md. Native runtime SQLite is **3.53.2**.

## Tests added

Thirteen frontend tests cover separate driver/owner forms, driver-only ACH, manual owner Amount, string-only sends/large integer display, custom periods/filter payloads, multiple loads, default driver/type selection, backend totals, ADD/DEDUCT/finalize, negative acknowledgment, input preservation/inline errors, stale reload, finalized audit/save, owner multiple trucks and reference percentage/default edits. The total frontend suite is 23 tests.

Six Rust workspace tests exercise the full typed-boundary draft/finalize/ACH/correction/reopen workflow; negative-final confirmation; string-only IPC, overflow/precision/date validation and Amount injection; manual owner fields and parameterized filters; historical default/type independence; and version-3→4 preservation/mileage. One additional backup test verifies staged version-3 restore without rewriting the source. The total Rust suite is 42 tests, including prior exact arithmetic, constraint, transaction rollback, recovery and history coverage.

The native script drives the actual Windows WebView2 application and real Tauri IPC, not a browser mock. It refuses writes unless the database path contains the isolated Phase 4 test identifier. Native axe checks cover payroll driver/owner forms, settlement editor, reference editor and Settings in light/dark at 1440×900, 1024×768, 800×600 and 375×700, with no page overflow. Keyboard input and Tab navigation are exercised in the owner form. Device scale factors 1, 1.25, 1.5 and 2 are CDP-emulated in the actual WebView; Windows display settings were unchanged. This does not replace the later physical-monitor/Windows release matrix.

## Real native synthetic workflow

The final clean run used temporary identifier **com.twx.payroll.phase4-smoke-final**. An earlier diagnostic run used com.twx.payroll.phase4-smoke. Both are deliberately isolated; no synthetic business fixture was written to the ordinary com.twx.payroll database. The ordinary release was also launched and its empty business tables verified read-only after migration 004.

| Required step | Observed result |
|---|---|
| 1. Create owner | PASS Synthetic Owner through native UI |
| 2. Create driver type | PASS Synthetic Type, explicitly entered 85% |
| 3. Create driver | PASS Synthetic Driver linked to that type |
| 4. Create truck/owner/default driver | PASS FAKE-001 linked to both |
| 5. Create settlement/two loads | PASS custom 10/01/2026–10/19/2026, fake load/BOL/PO and plant/job |
| 6. Tons × Completion Rate | PASS 10×5=$50; 20×4=$80; mileage persisted separately |
| 7. Deduction/addition | PASS Insurance DEDUCT $10; Reimbursement ADD $2; synthetic code/explanations |
| 8. Finalize | PASS FINALIZED and linked driver entry |
| 9. Final payout | PASS $130×85%=$110.50; −$10+$2=$102.50 |
| 10. Linked payroll Amount | PASS read-only $102.50 |
| 11. Add ACH | PASS $1.25 plus code and notes |
| 12. Payroll total | PASS $103.75 |
| 13. Close/reopen | PASS actual native window/process closed and relaunched |
| 14. Persistence | PASS settlement/payroll/mileage/ACH/code/notes and SQLite dark theme |
| 15. Finalized edit | PASS first Completion Rate changed from 5 to 6 |
| 16. Payroll updates | PASS gross $140; driver gross $119; final/Amount $111; retained ACH; Total $112.25 |
| 17. Audit/history | PASS CORRECTION has before 50000/after 60000; primary payroll/history reflects updates |
| 18. Backup dirty-state | PASS real SQLite change_generation exceeds saved_generation after business writes |

Additional native checks: manual owner $123.45 has no ACH control; date/driver/truck history filters retrieve the saved period; injected Number ACH and manual driver Amount are rejected; ungranted filesystem, shell, SQL and window commands are denied. Ordinary production launch showed schema 4/SQLite 3.53.2 and Settings/backup/reference UI. Relaunch produced one ordinary process/window, retaining single-instance behavior. The native diagnostic app was closed after verification; the ordinary review app was left available.

Reported logical test database path:
`C:\Users\[redacted]\AppData\Local\com.twx.payroll.phase4-smoke-final\data\twx-payroll.sqlite3`.
Windows in this Codex-hosted environment resolves the physical file beneath:
`C:\Users\[redacted]\AppData\Local\Packages\OpenAI.Codex_2p2nqsd0c76g0\LocalCache\Local\com.twx.payroll.phase4-smoke-final\data\twx-payroll.sqlite3`.
This redirection is environmental; the application uses Tauri's path resolver. Read-only checks verified schema, integer values, integrity and foreign keys. The ordinary path uses com.twx.payroll instead of the test identifier.

## Commands and results

Environment setup `. ./work/phase2-env.ps1` selects the previously cached pinned Rust toolchain. `CARGO_NET_OFFLINE=true` is used. The existing environment-only Cargo relay supplies the cached registry namespace, with TLS validation preserved; it does not change production code or lockfiles. No dependency installation/download was needed.

| Command/check | Result |
|---|---|
| npm.cmd run typecheck | PASS strict TS, also run by full verification/build |
| npm.cmd run lint | PASS ESLint |
| npm.cmd test | PASS 23 Vitest tests plus foundation config/SQLite check |
| npm.cmd run build | PASS Vite production frontend and strict TS |
| npm.cmd run test:ui | PASS Edge/Playwright shell/navigation/theme/keyboard/axe at four widths |
| npm.cmd run format:check | PASS Prettier |
| npm.cmd run verify:frontend | PASS complete chain, final log work/phase4-frontend-final.log |
| npx.cmd prettier --write src scripts/*.mjs | PASS formatting changes |
| cargo fmt --manifest-path src-tauri/Cargo.toml | PASS formatting changes |
| node work/cargo-relay.mjs test --manifest-path src-tauri/Cargo.toml --locked | PASS typed workspace tests, first 41 then expanded final suite below |
| node work/cargo-relay.mjs --native | PASS invokes scripts/verify-native.ps1; work/phase4-native.log |
| cargo fmt --manifest-path src-tauri/Cargo.toml -- --check | PASS via native script |
| cargo check --manifest-path src-tauri/Cargo.toml --locked | PASS via native script |
| cargo test --manifest-path src-tauri/Cargo.toml --locked | PASS 42 tests via native script |
| cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings | PASS via native script; no warning suppressions |
| npm.cmd run tauri -- build --no-bundle -- --locked | PASS ordinary Windows optimized executable, via native script |
| node work/cargo-relay.mjs --tauri build --debug --no-bundle --config work/phase4-native-config.json -- --locked | PASS isolated diagnostic native builds, including final clean fixture run |
| node scripts/verify-business-native.mjs | PASS final native initial workflow; work/phase4-native-initial-final.log |
| node scripts/verify-business-native.mjs --restart | PASS final restart/edit/IPC/accessibility/keyboard/filter workflow; work/phase4-native-restart-final.log |
| node scripts/verify-business-native.mjs --resume / --restart --post-edit | PASS earlier diagnostic recovery runs after harness corrections; not used for final clean acceptance |
| node work/phase4-runtime.cjs | PASS read-only dirty state, audit, integer totals, integrity/FK/schema |
| node work/phase4-production-runtime.cjs | PASS ordinary schema 4, integrity and zero business fixture counts |
| Computer-use launch/observe/close/reopen and ordinary relaunch | PASS final Windows builds and single instance |
| Get-FileHash release executable/migrations; lockfile existence; git status --short | PASS inspection; checkout remains uncommitted |

Initial failures were fixed: a frontend test expected an amount as a standalone label; strict TS rejected Testing Library's unsupported exact option; lint rejected synchronous state clearing in effect guards (moved to explicit handlers); one Rust negative-test fixture unintentionally reset its deduction; native harness selectors/category names were corrected to the actual labels and CORRECTION event. The final clean runs above pass. Node's experimental SQLite warning remains limited to read-only/config test tooling; the product uses bundled rusqlite. No native compiler/test/build warnings or remaining failures were observed. Test diagnostic settings are absent from the ordinary release; nothing was published.

Production artifact: `src-tauri/target/release/twx-payroll.exe`, SHA-256 `3CC71A5A6720CABE60F9DB7F597C31EE743F76769E2786B38B77A1D4C627CFA1`. Local artifacts/logs/screenshots are under ignored outputs/desktop-phase-4 and work. Both lockfiles and source files are present but not committed or pushed; Git index access remains restricted in this environment.

## Material files and review decisions

New: src/contracts/business.ts; src/components/BusinessForms.tsx; src/features/references/ReferenceManager.tsx; src/features/settlements/Settlements.tsx; src/features/payroll/Payroll.tsx; src/features/BusinessWorkflow.test.tsx; src-tauri/src/workspace.rs and workspace/tests.rs; migration 004; generated named command permission manifests; scripts/verify-business-native.mjs; this review.

Changed: App.tsx/App.test.tsx, styles.css, application.rs and its test fixture, domain.rs, storage.rs, backup.rs/backup tests, commands.rs, lib.rs, build.rs, capabilities/main.json, scripts/verify-foundation.mjs, AGENTS.md, ARCHITECTURE.md, PRODUCT_SPEC.md, IMPLEMENTATION_PLAN.md, README.md, DEPENDENCIES.md and RECOVERY_GUIDE.md. No obsolete web code, dependency version or migration 001–003 changed.

Before reports/exports: review provisional rounding against representative historical statements; approve required report layouts/columns and historical snapshot rendering; confirm duplicate/reference/import conventions and any additional finalization completeness rules; decide mileage units if they become meaningful; decide any eventual handling of negative balances/payment execution. Existing historical records are never silently reinterpreted. Windows installer/signing/WebView2 provisioning, physical hardware DPI and offline clean-machine distribution/recovery remain later release gates. Review the Phase 4 screens/workflows and authorize the next phase separately.

**Stopped for review. No Phase 5 work is authorized or started.**
