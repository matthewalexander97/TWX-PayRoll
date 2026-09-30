# TWX Payroll — desktop implementation plan

Status: Desktop Phase 1 authorized and implementation prepared for review. Frontend checks pass; native compiler/build checks and runtime smoke tests have passed; see docs/DESKTOP_NATIVE_ACCEPTANCE.md. Phase 2 is not authorized. Superseded web scaffold is isolated under legacy/web.

## Phase 0 — revised review (approved)

Revise four planning documents, local ownership/security, SQLite schema/exact values, data path, backup/recovery, offline distribution, and domain questions. Mark historical web documents without deleting code. Acceptance: consistent offline single-user architecture and clear desktop phase gates. No scaffolding/installations/migrations/deployment.

## Phase 1 — desktop foundation (currently authorized)

Select supported stable Tauri 2/React/TypeScript/Tailwind/bundler/Rust/SQLite versions; record versions, package/Cargo lockfiles, and Windows developer prerequisites. Build local bundled shell/navigation, empty payroll/settlement/directory views, loading/error states, resize/DPI/keyboard behavior, light/dark/manual remembered theme, About/read-only data location. No login system.

Establish stable app ID, app-local data path, single-instance/serialized writer, private SQLite connections, typed health/settings commands, least-privilege capabilities/CSP, and transactional migrations with rusqlite_migration owning PRAGMA user_version. Only nonfinancial app_settings may be created; no migration ledger table. Test migrations use disposable databases. No domain/reference/audit/backup_runs tables, financial logic, backup/export/API commands, or production distribution.

Acceptance: desktop window launches offline on Windows; resolved path/user/identifier survives restart; active storage is not synced/Documents/install folder; connection pragmas and migration/settings infrastructure work; failed migrations roll back/unknown future schemas refuse writes; themes/layout/keyboard work; unauthorized IPC/path commands fail without broad SQL/filesystem/shell permission. Run typecheck, lint, relevant TS/Rust tests, cargo fmt --check, cargo clippy with warnings denied, frontend build, Windows Tauri production build/runtime smoke check. Report signing/WebView2 prerequisites honestly. Stop before Phase 2.

## Phase 2 — backup/recovery before real business data

Approve Q16. Implement manual backup/full standalone database export, destination picker, in-app automatic schedule/catch-up, retention, backup_runs, last-success/failure state, and guided restore. Use SQLite Backup API, validation/manifest/checksum/completion protocol, restore lock/journal/rollback, correct WAL handling, and offline recovery documentation. Closed-app scheduling remains separate.

Acceptance: backup active WAL commits; restore on clean offline installation; reject corruption/newer schema; handle interruption, disk-full, inaccessible/removable/synced destinations, file locks, stale sidecars, migration incompatibility, and retention without deleting last good generation. Use disposable synthetic nonbusiness schema. Reconcile state/settings after restart/restore. Run all phase checks. Exit: reliable recovery before real data entry.

## Phase 3 — approved business schema and directory

Requires explicit authorization and schema decisions Q5–Q7/Q9/Q12/Q13. Implement the 12 domain/audit tables, SQLite migrations/FKs/CHECKs/indexes/finalized guards, audited Rust repositories, and catalog CRUD/archive. Seed synthetic examples/adjustment labels only; no rate/deduction assumptions. Take validated pre-migration backups.

Acceptance: clean/upgraded schema parity; both/neither payroll payees fail; archive/FK/history behavior holds; malformed/overflow/overprecision values fail in Rust; finalized guards/transaction audit work; failed migrations preserve data. Run phase checks. Exit: reviewed relational persistence/catalogs.

## Phase 4 — payroll drafts

Implement separate driver/owner forms/history/filters, approved exact minor-unit values, independent unknown ACH, atomic/versioned saves, and recoverable errors. No inferred payable amount.

Acceptance: create/edit/reopen/prior weeks offline; exact values/kind separation; conflicts/locks/disk errors cannot partly commit; backups restore identical records. Run phase checks.

## Phase 5 — weekly settlement/load drafts

Implement explicit start/end header, required load fields with approved rate terminology/units, flexible adjustments, provenance, reordering, aggregate atomic saves, history/filters, and pending-total labels. No unapproved formulas.

Acceptance: full record/restart round-trip, new adjustment types, NULL/zero/decimal distinctions, child/parent version conflicts, rollback/reordering, backup/restore fidelity. Run phase checks.

## Phase 6 — approved calculations/finalization/corrections

Require relevant Q1–Q11/Q13 answers and completed rule-approval records/worked examples. Implement exact Rust arithmetic, explicit rounding, atomic snapshot/status/audit finalization, immutable statement history, and approved simple correction flow. New rules never rewrite old revisions.

Acceptance: approved examples plus tie/sign/precision/overflow/nonterminating-division/rounding boundaries; sum-of-rounded-lines versus rounded-sum examples; stale editors/failed transactions cannot partly finalize; catalog/rule changes preserve statements; correction lineage/report/payment rules agree. Run phase checks.

## Phase 7 — local reports/exports/print

Approve report samples/selection. Implement canonical dataset, local PDF/print first, CSV/XLSX/DOCX adapters, approved destinations, draft/revision labels, safe spreadsheet text, offline fonts/pagination.

Acceptance: formats match database/snapshots exactly; Unicode/notes/page breaks work; offline printing/open or recoverable error; interrupted/disk-full writes do not publish partial artifacts. Run phase checks and rendered-report review.

## Phase 8 — Windows release and recovery readiness

Finalize Windows/CPU/DPI targets, per-user installer, offline WebView2 provisioning, signing/release ownership, stable path, updates/migration/rollback and uninstall preservation. Ship installer/source/schema/recovery materials independent of developer accounts. Excel import is separately reviewed/authorized if Q18 requires it.

Acceptance: clean-PC offline install/use/backup/restore/reinstall/upgrade without development tools/cloud accounts; recovery drill and data preservation; business acceptance on approved rules. Run full checks and installer smoke tests. Publishing/shipping is not authorized by this planning task.

## Phase 9 — optional customer-owned integrations

Only requested routing/Sheets/email providers; local secure credentials, callback/native mail limits, external work outside financial transactions, idempotent retries, manual override preservation/draft review. No mandatory hosted TWX backend. Sync/multi-device is a separate architecture.

Acceptance: offline/no credentials/provider failures leave bookkeeping usable; customer revocation/reconfiguration works; secrets absent from UI/logs/backups; no duplicate imports/sends or finalized edits. Run adapter-specific checks.

## Gates

No phase starts automatically. Later Phase 1 can proceed without financial answers because it excludes business tables/calculations. Recovery deliberately precedes real-data entry. Schema and financial implementation await their relevant business approvals. Split phases into smaller reviewable milestones if needed. Phase 1 does not select business quantity precision. Do not run superseded web builds. Installer signing is deferred to distribution and does not block Phase 1. See docs/DESKTOP_PHASE_1_REVIEW.md for verification evidence and remaining native acceptance.
