# TWX Payroll — contributor instructions

## Current authorization

Desktop Phase 1 is authorized. Read ARCHITECTURE.md, docs/PRODUCT_SPEC.md, and docs/IMPLEMENTATION_PLAN.md before changes. Implement only the Tauri/React application foundation, operational settings and migration infrastructure. Do not implement business tables, calculations, exports, backup/restore, integrations, or distribution. Stop before Phase 2.

The previously created web scaffold is superseded work, not the desktop foundation. It is isolated under legacy/web and excluded from all desktop builds. Do not run its scripts or count its tests as desktop verification. Historical web setup/review documents are labeled superseded.

## Engineering conventions

- Use Tauri 2, React, strict TypeScript, Tailwind CSS, SQLite, and narrow Rust/Tauri commands. Select supported stable versions when implementation is authorized; record versions and commit JavaScript and Rust lockfiles.
- Keep one local app and one authoritative business database. No multi-tenant infrastructure, mandatory online accounts, or hosted services.
- Separate UI, validation, use cases, exact financial/domain rules, SQLite repositories, OS/filesystem services, local reports, and optional external adapters.
- Rust validates privileged commands and owns authoritative calculations/transactions. Never expose arbitrary SQL, arbitrary filesystem operations, or shell execution to the frontend.
- Use versioned SQLite migrations managed by rusqlite_migration using PRAGMA user_version, without a separate manually maintained ledger. Enable foreign keys on every connection. Business records belong in SQLite, not localStorage, JSON files, spreadsheets, or reports. Theme may use local settings.

## Financial integrity

- Do not invent rate terminology/formulas, ACH treatment, percentages, conversions, settlement totals, payroll linkage, assignments, or corrections. Pending questions are in PRODUCT_SPEC.md.
- Store money as INTEGER minor units with an approved currency/scale; fractional rates/quantities as canonical decimal TEXT. No REAL, SQLite decimal affinity, or JavaScript Number for authoritative financial values.
- Exchange financial values as strings across IPC. Rust checks exact arithmetic and overflow before persisting integers; intermediate division requires an explicit approved rounding policy.
- Do not assume rounding behavior. PRODUCT_SPEC.md must define decimal precision and the point at which rounding occurs for each approved financial calculation before implementation.
- NULL means unknown; zero means explicit zero. Reject malformed/nonfinite/overprecision/overflow input without silent rounding.
- Save/finalize/correct and audit atomically. Use expected versions, a serialized writer, and guards against changing finalized records.
- Preserve finalized statement facts, labels/TWX codes, currency/scale, rule versions, and totals. Enforce exactly one driver or owner with a database CHECK, without a generic payee hierarchy.

## Local security, recovery, and portability

- OS-account access is the initial security boundary. No cloud login or user/role/tenant tables in the single-user MVP. A PIN alone does not encrypt files.
- Tauri capabilities allow only the named local window and specific commands. No broad filesystem, remote-page IPC, generic frontend SQL, or unrestricted shell permissions.
- Resolve a stable per-user application-local-data path. The live database is not stored in Documents, installation directories, or synchronized/network folders.
- Backups are first-class release requirements: SQLite-safe snapshots, validation, retained generations, restore rollback, and honest last-success/failure status. Never raw-copy an active WAL database.
- Restore quiesces/closes connections, validates compatibility, preserves rollback, and recovers after interruption. Never attach old WAL sidecars to a restored main database.
- Optional integration credentials belong to the customer, remain in an appropriate OS secure store, and never enter UI, logs, snapshots, or ordinary backups.
- Offline bookkeeping must keep working. A backup in a user-selected synced folder is not live database synchronization.

## Verification

Authorized implementation phases run strict type checking, lint, relevant TS/Rust tests, cargo fmt --check, cargo clippy with warnings denied, frontend build, and a Windows Tauri production build/runtime smoke check. Verify migration/path/capability behavior and later backup/restore/financial rules with synthetic fixtures. A browser build alone does not verify a desktop release.

Report failed/unrun checks accurately. Phase 1 implementation requires the applicable checks; report environment limitations without declaring native verification passed. Preserve customer data through upgrades/uninstall and ship offline recovery guidance. Do not begin the next phase without instructions.
