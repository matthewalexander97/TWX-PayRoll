# Desktop Phase 2 — core implementation review

Date: October 3, 2026. Phase 1 was accepted by the user. Phase 2 is implemented and returned for review. Full business UI, SQLite-safe backup/recovery, exports, integrations and distribution have not been implemented. No next phase has begun. Synthetic test fixtures only.

## Documentation changes

AGENTS.md, ARCHITECTURE.md, PRODUCT_SPEC.md and IMPLEMENTATION_PLAN.md now reflect the confirmed Completion Rate terminology, single Net Weight/Tons quantity, driver-type payout percentage, explicit ADD/DEDUCT, driver-only ACH addition, one-owner/multiple-trucks relationship, optional default driver, explicit periods, settlement-generated payroll and simple audited finalized edits. Confirmed answers replace the former unresolved planning statements. The revised phase order puts the authorized business core in Phase 2 and keeps SQLite-safe backup/recovery as the next separately approved prerequisite to real-data entry. Historical Phase 1 reports remain dated evidence.

README describes the core and unchanged placeholder UI. DEPENDENCIES records exact-pinned serde_json 1.0.151, previously present transitively; no arithmetic library or dependency upgrade was needed. Cargo.lock adds that direct dependency edge; package-lock.json is unchanged.

## Final tables and relationships

Schema version 2 contains 14 tables: app_settings, driver_types, drivers, owners, trucks, twx_codes, plants, jobs, adjustment_types, weekly_settlements, loads, settlement_adjustments, payroll_entries, audit_events.

- driver_types stores configurable payout_rate_ppm; drivers has a required driver_type_id. Settlement creation snapshots the selected percentage; later type changes affect new records only.
- owners → trucks is one-to-many through required owner_id. Nullable trucks.default_driver_id is convenience only. Actual settlement/load driver_id and truck_id persist independently.
- jobs may reference plants; adjustment_types may reference a default TWX code and direction. Explicit adjustment rows store direction and nonnegative amount_cents with optional code references. No assumed production seed values.
- weekly_settlements has explicit period_start/end, driver/truck/owner references, snapshot names, payout percentage, computed totals, status, row version, UTC timestamps and final snapshot/rule version. Loads and adjustments reference the settlement and have unique display positions within it.
- loads stores one tons_milli quantity, completion_rate_scaled, rounded total_cents, actual driver/truck and optional plant/job IDs with label snapshots. Child timestamps/versioning belong to the aggregate header and its audit events.
- payroll_entries enforces DRIVER with driver_id and no owner_id, or OWNER with owner_id and no driver_id. DRIVER requires unique settlement_id and nonnegative non-NULL ACH; OWNER requires no settlement link and NULL ACH. Stored amount_cents, ach_charge_cents and total_cents remain separate. Owner amounts are manual; no unconfirmed owner percentage formula.
- audit_events stores affected entity type/id, timestamp, category/reason and before/after JSON. UPDATE, DELETE and replacement-by-ID are rejected by triggers. Entity-scoped reads distinguish overlapping IDs in different tables.

All tables are STRICT. Nonblank TEXT primary keys identify domain rows; audit IDs are INTEGER. Catalogs have active/version/timestamps. Foreign keys restrict deletion, and FK/period/history indexes support retrieval. Dates are real-calendar validated in Rust and SQLite, with end >= start. No weekday, period-length or duplicate-payee/week restriction. No generic payee hierarchy, migration ledger, cloud accounts or business JSON files.

## Exact storage and arithmetic

| Value | Exact encoding |
|---|---|
| Money | INTEGER cents, scale 100; signed i64 results |
| Tons / Net Weight | tons_milli, scale 1,000; nonnegative i64 |
| Completion Rate | completion_rate_scaled, scale 10,000 per money unit per ton; nonnegative i64 |
| Payout percentage | payout_rate_ppm, 1,000,000 = 100%; inclusive 0..1,000,000; four decimal places in percentage notation |
| Intermediate calculations | Checked Rust i128, followed by checked i64 output bounds |
| Historical JSON / future financial IPC | Exact integer or decimal strings, never floating-point JSON operands |

Strict decimal parsing rejects whitespace, signs on nonnegative inputs, malformed text, exponent/nonfinite values, excess decimal places and overflow. No REAL, authoritative JavaScript Number or SQL money multiplication/summing is used.

The centralized rule is twx-fixed-v1-provisional. half_up divides exact integers, examines the remainder and adjusts on ties away from zero. It avoids adding half the denominator to the numerator, which could overflow near a wide-integer limit.

1. Load cents = half_up(tons_milli × completion_rate_scaled / 100,000).
2. Sum the individually rounded load cents exactly.
3. Driver gross cents = half_up(gross route cents × payout_rate_ppm / 1,000,000).
4. Final settlement cents = driver gross + ADD amounts − DEDUCT amounts.
5. Driver payroll total cents = settlement-derived Amount + ACH cents.

Example synthetic fixture: 12.345 tons × 6.7891 = load 83.81; 85% yields driver gross 71.24; deduction 1.00 yields settlement 70.24; ACH 1.25 produces payroll 71.49. Two half-cent loads round individually to a two-cent gross, rather than rounding their exact combined value to one cent. Zero and signed negative final results remain exact, without clamping.

## Transactional behavior and history

BusinessService runs BEGIN IMMEDIATE and checks expected row versions. save_settlement creates/replaces an aggregate draft. Editing a finalized aggregate automatically calculates and synchronizes payroll in the same transaction. finalize_settlement rounds/calculates, writes finalized totals and snapshot, upserts the single settlement-linked payroll row, appends both audit events and commits. Repeating an unchanged finalized version returns existing results without duplicate payroll or audit; stale versions are rejected.

Payroll synchronization preserves its ACH, TWX code and notes. update_driver_payroll edits ACH/code/notes with its own version check and audit and cannot change settlement Amount. save_owner_payroll accepts an explicit manual amount and writes a finalized owner record/audit with no ACH field in its input type. No external payment execution, ledger reversal or hidden adjustment formula is implied.

Label snapshots are captured on reference selection; unchanged references retain their saved labels during subsequent edits. Selecting a different reference captures the new label. Percentage is captured from driver type on creation and changes thereafter only through an explicit financial edit. Final snapshots preserve all header/load/adjustment inputs, labels, results, dates and rule version. Before/after audits preserve older snapshots while the bookkeeper keeps the same record identity. Unknown policy versions refuse recalculation.

All aggregate/audit/payroll effects roll back on any failure, including a late audit insertion failure after payroll synchronization. Services are callable Rust APIs, not exposed as generic SQL or new Tauri commands. The existing mutex serializes application storage. Append-only guards prevent accidental audit edits; local administrators can still modify the database outside application controls, so this is not tamper-proof accounting infrastructure.

## Migration and runtime evidence

Added src-tauri/migrations/002_business.sql. rusqlite_migration still owns user_version. The original 001_settings.sql is byte-for-byte identical to the Phase 1 source snapshot (SHA256 0A142528B10520B3DF16B0DFE6122C5D59A6FBE682E723A0046D92914F6A9A2A). No applied Phase 1 migration was edited, no manual migration metadata was added, and migration failures never reset/delete an existing database.

Synthetic tests exercise clean creation, real migration-001 databases upgraded to version 2, repeat reopen, future-version refusal and forced migration failure with original settings/extra fixture data preserved.

The actual production desktop executable also launched and restarted successfully using the existing settings-only LocalAppData database. It upgraded from version 1 to 2, retained System preference and its original updated_at timestamp, and reported SQLite 3.53.2 in native Settings. Read-only inspection returned integrity_check=ok and no foreign-key errors. All 13 business/audit tables are empty in that live development database; no test or real business rows were entered there.

Observed path, username redacted:

    C:\Users\[redacted]\AppData\Local\com.twx.payroll\data\twx-payroll.sqlite3

Production executable was produced at src-tauri/target/release/twx-payroll.exe. SHA256: 1E48C2E62C84F9876D07F0D4AB1BF3D80ADE20A7574FD17C51FE0AC64003D680. It was run locally for validation, not bundled, signed, published or distributed. Computer Use inspected the native window and Settings; no business debug UI or diagnostic capability was added. The Phase 1 shell remains a foundation preview.

## Tests and every verification command

Final Rust suite: 22 passed, 0 failed, 0 ignored (18 new Phase 2 tests plus 4 retained/updated storage tests). Frontend: 5 passed. Native binary/doc-test targets also completed with no failures. No warnings were suppressed.

| Command | Result |
|---|---|
| cargo fmt --manifest-path src-tauri/Cargo.toml | PASS; formatted new code, repeated after final additions |
| node work/cargo-relay.mjs test --manifest-path src-tauri/Cargo.toml --offline | PASS initial 20 tests; updated Cargo.lock for the already-cached serde_json direct dependency |
| npm.cmd run verify:frontend | PASS: typecheck, lint, five tests, foundation assertions, production build, UI checks and format check |
| npm run typecheck (tsc --noEmit) | PASS in frontend suite and final native beforeBuildCommand |
| npm run lint | PASS in frontend suite and after placeholder wording update |
| npm test (vitest run and npm run test:foundation) | PASS in suite and final repeat after wording update |
| node scripts/verify-foundation.mjs | PASS existing capability/build-boundary and immutable Phase 1 settings migration assertions; business migrations verified in Rust |
| npm run build | PASS strict TypeScript and Vite production frontend build, including final native build |
| npm run test:ui | PASS browser navigation/themes/accessibility at 1440x900, 1024x768, 800x600 and 375x700 |
| npm run format:check | PASS in suite and final repeat |
| node work/cargo-relay.mjs --native | PASS twice, including final 22-test code and production build; invokes the PowerShell native script below |
| pwsh -NoProfile -File scripts/verify-native.ps1 | PASS entire script through cached workspace environment |
| cargo fmt --manifest-path src-tauri/Cargo.toml -- --check | PASS in native script |
| cargo check --manifest-path src-tauri/Cargo.toml --locked | PASS in native script |
| cargo test --manifest-path src-tauri/Cargo.toml --locked | PASS final 22 tests, binary/doc targets |
| cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings | PASS; no suppression |
| npm run tauri -- build --no-bundle -- --locked | PASS final optimized Windows production binary, with production frontend build |
| node work/read-live-settings.cjs | PASS read-only before/after inspection: schema 1→2, theme unchanged, integrity ok |
| node work/phase2-runtime.cjs | PASS read-only schema/settings/integrity/FK checks and empty business-table confirmation after native launch/restart |
| Get-FileHash on original migration and package-lock versus Phase 1 source snapshot | PASS both match their Phase 1 versions |
| Cargo.lock scan for localhost/temporary relay source | PASS no temporary transport source persisted |
| Native production launch/restart via Computer Use | PASS local bundled frontend, schema 2, SQLite 3.53.2, saved theme and expected placeholders |

Rust commands used pinned workspace Rust/Cargo 1.98.1 with CARGO_HOME/RUSTUP_HOME beneath ignored work/toolchains and CARGO_NET_OFFLINE=true for the native scripts. The existing ignored temporary relay wrapper supplies the same cached registry namespace used during Phase 1. No downloads, certificate bypass, permanent transport configuration or runtime dependency on that wrapper was introduced. On a normal developer environment, scripts/verify-native.ps1 is the standard verification entry point.

Node's SQLite ExperimentalWarning appeared in the existing foundation/read-only inspection tooling; the application uses bundled Rust SQLite. This is not a failed check. No build/test/lint/native failure remained. Existing Git repository contents remain uncommitted; this task makes no GitHub push claim, and the prior Git write-permission issue is separate from successful builds.

## Materially added or changed files

- AGENTS.md; ARCHITECTURE.md; README.md.
- docs/PRODUCT_SPEC.md; docs/IMPLEMENTATION_PLAN.md; docs/DEPENDENCIES.md; docs/DESKTOP_NATIVE_ACCEPTANCE.md (historical-status banner); this report.
- src-tauri/migrations/002_business.sql (new); 001_settings.sql unchanged.
- src-tauri/src/domain.rs (new exact math, validation and tests).
- src-tauri/src/repositories.rs (new configurable catalog writes and historical reads).
- src-tauri/src/application.rs and application/tests.rs (new atomic financial services and integration tests).
- src-tauri/src/storage.rs (register migration 2, expose typed Rust services, update storage test expectations).
- src-tauri/src/lib.rs (module/API registration).
- src-tauri/Cargo.toml and Cargo.lock (serde_json direct dependency).
- src/features/Placeholder.tsx (remove outdated claim that financial rules remain pending; no business workflow added).

Tauri command/capability/CSP files, desktop dependency versions and package-lock.json are unchanged. Legacy web code does not participate in builds.

## Review before any next phase

1. Confirm the provisional precision/rounding against TWX historical examples before real-data use. Rule versioning isolates future changes; no automatic historical recalculation.
2. Review currency display, signed negative settlement handling, manual nonnegative owner amounts and completeness expectations (the core can finalize an empty/zero settlement). No payment execution semantics are invented.
3. Review independent header/load actual driver/truck handling and the rule that a saved payout percentage changes only explicitly, including when changing a draft's selected driver. Decide UI warnings/defaults in the UI phase.
4. IDs are nonblank opaque TEXT in this Rust core; future command/UI adapters must generate collision-resistant IDs. No manual revision interface is needed.
5. Approve and complete SQLite-safe backup/recovery before any real payroll record. Then separately authorize full UI work. Reports, integrations, installer/signing/distribution remain later gates.
6. Commit the source and lockfiles when Git write access is available; no commit or push is claimed here.

STOP for review. No next phase is authorized by this completion report.

