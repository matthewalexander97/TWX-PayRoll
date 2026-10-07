# TWX Payroll — desktop architecture

Status: Desktop Phases 1–7 implementation accepted. Final release validation and release-specific corrections only; no independent Windows environment is available. Routing/maps and public publication remain gated.

Current Phase 6 boundary: integrations are optional Rust output adapters. Historical phase-specific exclusions below describe those phases, not a ban on the expressly approved Phase 6 work. See INTEGRATIONS_GUIDE.md and DESKTOP_PHASE_6_REVIEW.md for current setup/verification.

## Boundaries

Tauri 2 hosts locally bundled React, strict TypeScript and Tailwind. Rust owns connection/migrations, validation, exact financial domain functions, repositories and application transactions. UI owns presentation only. Health/theme IPC remains narrow; Phase 3 adds only named backup/restore operations for the local main window. Phase 2 services are Rust APIs exercised by synthetic tests; no SQL/filesystem/shell/remote-origin access is added to React. legacy/web is excluded from every desktop build.

Code boundaries: src/app/components/features/contracts for presentation; src-tauri/src/domain.rs for exact arithmetic; repositories.rs for catalogs and historical reads; application.rs for financial mutations/finalization/audits; storage.rs for SQLite configuration/migrations/settings. No network service, multi-tenancy or authentication schema.

## SQLite and migration ownership

Stable identifier com.twx.payroll resolves app_local_data_dir()/data/twx-payroll.sqlite3 under per-user Windows LocalAppData. One instance and one serialized Rust writer; connections enable foreign_keys, WAL, synchronous FULL, trusted_schema OFF and five-second busy timeout. No extension loading. rusqlite_migration owns PRAGMA user_version and transactional ordered embedded SQL. Migration 001 remains byte-for-byte unchanged; 002 adds business structures. Unknown future schemas and migration failures preserve existing data and produce errors, never reset/delete databases.

All tables are STRICT with INTEGER/TEXT storage. Catalog IDs and record IDs are opaque nonblank TEXT supplied by trusted Rust callers (test fixtures use deterministic synthetic IDs); no business meaning is encoded in IDs. Future IPC adapters must generate collision-resistant IDs. Catalogs have created_at/updated_at UTC timestamps, row_version and is_active. Foreign keys restrict deletion; catalogs should be archived. Business rows have status DRAFT/FINALIZED, UTC timestamps, expected row versions and finalized snapshots. Child edits advance the parent version. Dates are validated both with SQL real-calendar checks and Rust validation. No fixed week.

## Table dictionary

All *_cents, tons_milli, completion_rate_scaled and payout_rate_ppm are exact INTEGER with scales/ranges from PRODUCT_SPEC.md. No REAL or decimal TEXT operands. JSON snapshot/audit integers are strings.

| Table | Important fields and relationships |
|---|---|
| app_settings | Existing Phase 1 theme table, unchanged |
| driver_types | id PK, name, payout_rate_ppm 0..1,000,000, active/version/timestamps |
| drivers | id PK, name, driver_type_id FK; active/version/timestamps |
| owners | id PK, name; active/version/timestamps |
| trucks | id PK, truck_number UNIQUE, owner_id NOT NULL FK, default_driver_id nullable FK; active/version/timestamps |
| twx_codes | id PK, code UNIQUE, label; active/version/timestamps |
| plants | id PK, name; active/version/timestamps |
| jobs | id PK, name, plant_id nullable FK; active/version/timestamps |
| adjustment_types | id PK, name, default_direction ADD/DEDUCT, twx_code_id nullable FK; active/version/timestamps |
| weekly_settlements | id PK, driver/truck/owner FKs and name/number snapshots, driver_type_name_snapshot, period_start/end, payout_rate_ppm snapshot, notes, status/version/timestamps, gross_route_cents, driver_gross_cents, final_payout_cents, rule_version, snapshot_json, finalized_at |
| loads | id PK, settlement_id FK, position unique per settlement, actual driver/truck FKs and snapshots, plant/job nullable FKs and snapshots, load_number/BOL/PO, tons_milli, completion_rate_scaled, total_cents |
| settlement_adjustments | id PK, settlement_id FK, position unique per settlement, adjustment_type_id FK plus name snapshot, twx_code_id nullable FK plus code/label snapshots, explanation, direction ADD/DEDUCT, nonnegative amount_cents |
| payroll_entries | id PK, entry_kind DRIVER/OWNER, driver XOR owner FK, truck FK, payee/truck snapshots, TWX FK and code/label snapshots, period_start/end, notes, unique settlement_id nullable FK, amount_cents, ach_charge_cents (DRIVER only), total_cents, status/version/timestamps, rule_version, snapshot_json, finalized_at |
| audit_events | INTEGER PK, occurred_at UTC, entity_type/id, category/reason, before_json nullable, after_json; immutable via update/delete triggers |

Phase 3 adds backup_settings and backup_runs; no actors/roles or generic payee hierarchy. No owner-truck join table. No duplicate authoritative Net Weight/Tons or percentage-adjustment fields. Parent-FK indexes support children and catalog lookups; period/status indexes support historical retrieval and entity audit indexes support chronological history. Settlement-linked DRIVER payroll requires a settlement and a unique link; OWNER payroll cannot link a driver settlement and cannot have ACH. No duplicate-payee/period uniqueness is invented.

## Atomic services and historical integrity

BEGIN IMMEDIATE serializes a full aggregate mutation. Save draft validates exact operands/IDs/periods, checks expected version and replaces aggregate children atomically. Finalize calculates with checked i128, sets final totals and snapshot, generates or updates one payroll row identified by unique settlement_id, preserves existing ACH/code/notes, appends both audit records and commits. Editing finalized inputs uses the same transaction and recalculates automatically. Stale versions fail without changes. Repeated unchanged finalization is idempotent. Payroll-level ACH changes check payroll version and cannot modify settlement Amount.

Snapshot labels are captured on initial reference selection; unchanged references retain stored labels, even during later financial edits. Selecting a different reference captures its current label. The settlement payout percentage is captured on creation and subsequently changes only through an explicit financial edit. Final snapshots capture complete header/child inputs, computed cents and rule version. Audit before/after captures full old/new snapshots (including linked payroll events) within the same transaction. UI needs no revision object. Reference edits cannot rewrite financial history.

Database CHECKs enforce dates, ranges, payee/type combinations, statuses and required final values. Append-only audit triggers prevent accidental mutation. Application services own aggregate calculations, synchronization and mandatory audit; a local administrator with direct database access can bypass application policy. No misleading claim of database-level tamper resistance. Finalized deletion is rejected; draft deletion is not exposed in this phase. Financial values are never calculated with SQL multiplication or JavaScript Number.

## Rounding and future policy changes

Use twx-fixed-v1-provisional, with integer scale/half-up algorithms specified in PRODUCT_SPEC.md. Round each load once to cents, sum lines, round payout percentage result once, then add/subtract cents. Signed half ties round away from zero. Validate i64 persistence bounds after checked i128 operations. No arithmetic dependency is required. Historical records retain rule_version and saved results. An unknown policy must refuse recalculation; future policy changes require explicit migration/review, not silent rewriting.

## Later recovery, reports and distribution

Before real business data: implement SQLite Online Backup API snapshots, validate integrity/foreign keys/schema/domain invariants, stage and atomically publish completed generations, retain last good copies, and restore only with quiesced connections and rollback. Never blindly copy an active database or attach old WAL files. Destination may be removable/already synced, but live DB stays local. Phase 3 scheduling and retention defaults are specified below; encryption remains outside this phase. Phase 2 uses synthetic disposable migration tests rather than touching customer records.

Reports later consume canonical historical snapshots for PDF/print, CSV, Excel and DOCX. Sheets/email/routing use optional customer-owned adapters and secure OS credential storage; external calls occur outside financial transactions and failures leave manual offline operation usable. No integration is implemented now.

Windows distribution later requires supported Windows/CPU/DPI testing, WebView2 provisioning, signing/release ownership, upgrade/recovery and uninstall data preservation. No publishing or installer bundling in Phase 2.


## Phase 3 clarification and approved implementation policy

Weekly settlement final payout automatically creates or updates the corresponding DRIVER payroll Amount. OWNER payroll remains manually entered. No owner-specific settlement-generation workflow exists unless future requirements explicitly authorize one.

Manual and automatic backups use rusqlite SQLite Online Backup API, including committed WAL data. The live database stays in LocalAppData. Native Windows file/folder pickers supply Rust-owned selections; React never sends filesystem paths. Validate full known schema, TWX application identity, integrity and foreign keys; reject newer schemas. Older known Phase 1/2 databases are recognized by their exact schema and migrated only in a private staged copy. Original migrations stay immutable.

A backup is staged in its chosen destination, validated, closed, SHA-256 hashed and published without overwriting existing files. A small adjacent completion manifest records identity/type/schema/size/hash, not business data. Record SUCCESS only after snapshot validation and publication. Failures retain truthful history; incomplete files are not completed generations. Timestamp plus random ID prevents name collisions.

Automatic backup is enabled by default but requires a selected destination. Without one, Settings clearly states that backup protection has not been configured; no same-drive fallback is presented as external protection. Evaluate at startup and every 60 seconds while running. Back up when tracked business data changed and the latest successful backup is at least 24 hours old, or no successful baseline exists. Retry failures no more often than hourly. Theme/UI actions and backup-history writes do not cause excessive generations. Default retention is 10 successful automatic generations in the current destination; never auto-delete manual or pre-restore safety backups. Delete only matching TWX completion manifests/files whose identity, recorded path and checksum are verified.

Restore is a two-step prepare/confirm operation: select and validate a backup, create/migrate an app-owned staged copy, display the selected name/date and replacement consequence, then require explicit confirmation with a one-use opaque token. Before altering live data, create and validate a retained pre-restore safety snapshot. A durable app-owned recovery marker is synced before restore. Serialize all database access and use SQLite Online Backup API to replace database contents transactionally, without raw-copying live files or combining WAL sidecars. Reopen and validate the restored database. Refresh React state after success.

On post-restore failure, retain the candidate and roll back from the safety snapshot; if rollback cannot complete, block writes and preserve the marker/copies for restart recovery. Startup with a pending marker conservatively restores the verified safety snapshot before exposing storage. Never delete the only known-good copy. Restore preserves local destination/automatic settings and merges backup-operation history, while business records and appearance return to the selected snapshot. A safety copy on the same disk supports undo but does not protect against disk failure.

Schema 3 adds backup_settings (one local policy/destination/change-generation row) and backup_runs (operation ID/time/type/path/status/schema/size/hash/validation/error metadata). Metadata JSON files are recovery/completion records, not alternate business storage. No provider API, arbitrary filesystem/SQL/shell permission, full payroll UI, installer or distribution is included. Synthetic data only until Phase 3 review accepts recovery.


## Phase 4 approved UI boundary

Reference lists/add/edit/archive, settlement creation/draft/finalization/finalized edits, separate DRIVER and manual OWNER payroll, period/driver/truck history filters and simple audit browsing are authorized. Typed named IPC uses decimal strings and expected versions; Rust supplies authoritative preview totals and invokes existing atomic application services. Driver Amount cannot be overwritten by the payroll command. Negative final payouts require explicit confirmation in both React and Rust. Currency display is USD ($), confirmed by the user on October 5, 2026; no conversion or financial policy change is introduced.

Migration 004 adds nullable loads.mileage as exact nonnegative decimal TEXT. It is informational, excluded from all formulas and preserved in aggregate snapshots/audits. No fixed mileage scale, unit conversion or rounding is invented; the 128-character limit is an input-size bound, not decimal precision. Historical finalized snapshots/audit JSON remain unchanged by migration; later edits capture mileage with the rest of the aggregate. Migrations 001–003 stay immutable. Backup compatibility recognizes schema versions 1–4 and stages older known versions before restore.

Native verification uses a temporary Tauri configuration with an isolated com.twx.payroll.phase4-smoke identifier and its own LocalAppData database. No synthetic business fixture is entered into the normal com.twx.payroll workspace. Diagnostic browser arguments/configuration stay under ignored work and are absent from the ordinary production build. Browser-only preview offers no fake business persistence. Phase 4 is accepted; its dated review remains historical evidence. Phase 5 is limited to local reports and print. Distribution remains gated.

## Phase 5 local reporting architecture

src-tauri/src/reports builds a typed section/cell model under the serialized SQLite access guard. Finalized settlement and linked payroll snapshots retain historical names, code labels, integer values and rule versions; unknown/corrupt policy snapshots fail explicitly. Saved drafts use persisted inputs and do not invent finalized totals. No report query writes business, audit or backup data and no migration is added. Rendering and native selection occur after releasing the database guard, so slow filesystem/viewer work does not hold a financial transaction.

CSV is one rectangular union of columns with section and row identifiers; numeric cells use exact decimal strings and dangerous spreadsheet text receives a leading apostrophe. XLSX uses separate driver/owner summary sheets, presentation-only numeric cells after a 15-significant-digit/fixed-scale round-trip check, and exact formatted text otherwise. These floating-point export cells never drive authoritative financial calculations. PDF embeds the bundled OFL Noto Sans font, wraps tables, repeats section/column headings and adds period/page footers. Unsupported font characters fail explicitly rather than replacing historical labels. DOCX is a real local OOXML package with landscape tables and multiline notes.

Windows COM IFileSaveDialog is owned by the main Tauri HWND. Rust chooses the only accepted extension and publishes a completed/synced tempfile in the selected directory with persist_noclobber; collisions/races do not overwrite. Application-data destinations are rejected. Print writes a unique private report-previews PDF then a private Windows adapter opens only that generated PDF through the OS viewer association. Frontend has no generic shell or filesystem permission and cannot supply a path, verb or arguments. Preview files are derived local copies, excluded from SQLite backups; physical printing is controlled by the viewer.

The capability is local main only, with three explicit report permissions added (24 total named permissions). Remote-origin IPC and broad SQL/filesystem/shell remain unavailable. External integrations and distribution require future authorization.

## Phase 6 optional output adapters

Phase 6 was explicitly approved after acceptance of Phases 1–5. integrations/{oauth,transport,vault,output} separates fixed-endpoint HTTPS, Google installed OAuth, current-user Windows DPAPI and canonical report output adapters. Separate Sheets and Gmail connections use customer-owned Desktop client files selected privately by Rust. Tokens never cross IPC or enter SQLite/report/audit/backup data. Sheets creates a new spreadsheet with literal exact historical values; Gmail sends generated MIME attachments only after a one-use report confirmation. No frontend paths, file bytes, URLs, credentials, SQL or shell parameters are accepted.

Preparation captures a canonical saved revision under the existing SQLite guard, then releases it. Integration workers serialize their own network/state operations independently, so an unavailable Google connection does not block local bookkeeping. A short-lived 127.0.0.1 callback exists only during OAuth. External HTTPS uses normal Windows certificate verification, fixed URLs, bounded bodies/deadlines, no redirects and no automatic retries. Startup performs no network call.

No migration is needed: schema remains 4, and migrations 001–004 remain immutable. Non-secret integration-operations.json records operation/destination IDs, timestamps, report kind, request digest and status. It deliberately remains outside SQLite so restoring financial data cannot roll back the knowledge of external effects. A synced SENDING marker precedes each non-idempotent operation; interrupted/ambiguous calls block an identical repeat. Corrupt operational metadata disables external actions only. Encrypted integration-credentials/*.dpapi is excluded from ordinary SQLite backups; new-machine recovery reconnects customer accounts. No business record is stored in either operational file.

Seven explicit local-main permissions add status, connection/cancel/disconnect and prepare/confirm/cancel report actions (31 total). Existing local CSP still denies frontend Internet connections. No integration plugin grants broad HTTP, filesystem, shell or remote-page IPC. INTEGRATIONS_GUIDE.md defines lifecycle, setup, size bounds, journal retention/reconciliation limits and pending live test acceptance. Routing/maps, hosted services, synchronization, multi-user features and Windows distribution remain gated.


## Phase 7 release candidate boundary

The user explicitly authorizes per-user Windows packaging and local release validation, export-folder settings and an offline User Manual. Historical exclusions above remain evidence of earlier scopes. No routing/maps, new financial rules, cloud synchronization, real data or public publishing. Production identity remains com.twx.payroll; version 0.2.0; schema stays 4 with immutable migrations 001–004. An installed release still needs independent-machine upgrade/recovery/uninstall verification; this developer host cannot supply that evidence.

Export Settings is Rust-owned machine-local configuration (export-directory.json), outside SQLite so restoring business data does not resurrect an obsolete machine path. Absence means the current OS/Tauri-resolved Documents directory. Only native selection can set a custom path. React accepts/display status and invokes named get/select/reset/open commands; it supplies no path. Ordinary export stages/syncs completed bytes and atomically publishes without clobbering, trying -2, -3 suffixes on collisions. Save As remains a distinct named command. Missing/unwritable destinations fail explicitly with no fallback; backup settings and financial/audit rows are untouched.

Print cache cleanup is seven-day, app-owned UUID PDF files only, with Windows exclusive DELETE access and DELETE_ON_CLOSE to prevent a probe/delete race. Open/locked files remain. A serialized cache operation enforces a 100-file/100-MiB quota and refuses new previews when full; no potentially open file is forcibly removed. Ordinary exports remain available.

Operation journal policy retains all entries, capped at 8 MB; SUCCESS/UNCERTAIN/SENDING digests are never automatically pruned or reset. At capacity new external actions fail before a provider send. Users verify Sheets/Sent Mail and use local exports; no uncertain operation automatically repeats and no reconciliation override is shipped. A lost journal/new PC requires remote review before new outputs.

The User Manual is bundled JSON rendered through simple React headings, local section links and paragraphs. USER_GUIDE.md is generated from that same content and checked in automated verification. No network/documentation service is involved. USER_GUIDE, DATA_RECOVERY, RELEASE_NOTES, RELEASE_CHECKLIST and SOURCE_HANDOFF describe the candidate and actual remaining validation gates. NSIS currentUser with offline WebView2 packaging was chosen for this known-user application; signing remains optional and the candidate is unsigned. No automatic updater/deployment is added.

Acceptance: strict frontend/native suites, export settings persistence/collision/failure/independent-backup tests, manual keyboard/offline/theme/axe checks, production and installer builds, isolated native report/recovery checks. Separately record independent installed app identity/path, runtime-only prerequisites, upgrade and uninstall preservation when a separate environment exists. Do not claim those blocked checks passed. Stop for review after Phase 7.

## Final release-validation correction

Phase 7 implementation is accepted. Only release-specific corrections are authorized. The pinned custom NSIS template removes the app-data deletion control and its code, preserving the complete workspace during interactive/passive/silent uninstall. Tests inspect both the source and generated script. This is implementation evidence, not runtime uninstall acceptance. No independent Windows environment is available; docs/RELEASE_VALIDATION.md tracks pending gates and current candidate hashes.
