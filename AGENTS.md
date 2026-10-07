# TWX Payroll — contributor instructions

## Current authorization

Desktop Phases 1–6 are accepted; live Google provider validation is pending. Phase 7 local Windows release candidate, export settings, offline User Manual and release validation are authorized. Read ARCHITECTURE.md, docs/PRODUCT_SPEC.md, docs/IMPLEMENTATION_PLAN.md, Phase 1–6 reviews, RECOVERY_GUIDE and INTEGRATIONS_GUIDE before changes. No routing/maps, unrelated business features, changed financial rules or public publication. Synthetic data only; native fixtures use an isolated test database. No independent Windows test environment is available; report that limitation. Prepare manual Git checkpoint commands without weakening .git security. Stop for review before another phase.

The previously created web scaffold is superseded work, not the desktop foundation. It is isolated under legacy/web and excluded from all desktop builds. Do not run its scripts or count its tests as desktop verification. Historical web setup/review documents are labeled superseded.

## Engineering conventions

- Use Tauri 2, React, strict TypeScript, Tailwind CSS, SQLite, and narrow Rust/Tauri commands. Select supported stable versions when implementation is authorized; record versions and include JavaScript and Rust lockfiles in the developer's manual checkpoint.
- Keep one local app and one authoritative business database. No multi-tenant infrastructure, mandatory online accounts, or hosted services.
- Separate UI, validation, use cases, exact financial/domain rules, SQLite repositories, OS/filesystem services, local reports, and optional external adapters.
- Rust validates privileged commands and owns authoritative calculations/transactions. Never expose arbitrary SQL, arbitrary filesystem operations, or shell execution to the frontend.
- Use versioned SQLite migrations managed by rusqlite_migration using PRAGMA user_version, without a separate manually maintained ledger. Enable foreign keys on every connection. Business records belong in SQLite, not localStorage, JSON files, spreadsheets, or reports. Theme may use local settings.

## Financial integrity

- Implement the confirmed Completion Rate, payout percentage, ADD/DEDUCT, driver-only ACH, linkage and audited finalized-edit rules in PRODUCT_SPEC.md. Do not invent additional business rules.
- Use INTEGER cents, tons_milli (1,000/ton), completion_rate_scaled (10,000/money unit per ton), and payout_rate_ppm (1,000,000=100%). Checked Rust i128 intermediates and i64 persistence. No REAL or JavaScript Number for authoritative financial values. Apply the centralized provisional half-up policy in PRODUCT_SPEC.md.
- Exchange financial values as strings across IPC. Rust checks exact arithmetic and overflow before persisting integers; intermediate division requires an explicit approved rounding policy.
- Do not assume rounding behavior. PRODUCT_SPEC.md must define decimal precision and the point at which rounding occurs for each approved financial calculation before implementation.
- NULL means unknown; zero means explicit zero. Reject malformed/nonfinite/overprecision/overflow input without silent rounding.
- Save/finalize/correct and audit atomically. Use expected versions, a serialized writer, and audited application services for finalized edits.
- Preserve finalized statement facts, labels/TWX codes, currency/scale, rule versions, and totals. Enforce exactly one driver or owner with a database CHECK, without a generic payee hierarchy.

## Local security, recovery, and portability

- OS-account access is the initial security boundary. No cloud login or user/role/tenant tables in the single-user MVP. A PIN alone does not encrypt files.
- Tauri capabilities allow only the named local window and specific commands. No broad filesystem, remote-page IPC, generic frontend SQL, or unrestricted shell permissions.
- Resolve a stable per-user application-local-data path. The live database is not stored in Documents, installation directories, or synchronized/network folders.
- Backups are first-class release requirements: SQLite-safe snapshots, validation, retained generations, restore rollback, and honest last-success/failure status. Never raw-copy an active WAL database.
- Restore quiesces/closes connections, validates compatibility, preserves rollback, and recovers after interruption. Never attach old WAL sidecars to a restored main database.
- Optional integration credentials belong to the customer, remain in an appropriate OS secure store, and never enter UI, logs, snapshots, or ordinary backups.
- Reports read saved inputs and finalized historical snapshots, never unsaved frontend state, current labels or recalculated old totals. One canonical Rust dataset drives every local format. Generated files are derived copies, not authoritative records or backups. Frontend report commands accept identifiers/filters and formats, never paths, SQL, file bytes or shell arguments.
- Offline bookkeeping must keep working. A backup in a user-selected synced folder is not live database synchronization.

## Verification

Authorized implementation phases run strict type checking, lint, relevant TS/Rust tests, cargo fmt --check, cargo clippy with warnings denied, frontend build, and a Windows Tauri production build/runtime smoke check. Verify migration/path/capability behavior and later backup/restore/financial rules with synthetic fixtures. A browser build alone does not verify a desktop release.

Report failed/unrun checks accurately. Report environment limitations without declaring native or independent-machine verification passed. Preserve customer data through upgrades/uninstall and ship offline recovery guidance. Do not begin the next phase without instructions.
