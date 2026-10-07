# TWX Payroll — implementation plan

Status: Desktop Phases 1–6 accepted; live Google provider validation remains pending. Phase 7 local Windows release candidate, export settings, offline User Manual and release validation are authorized. Routing/maps and public publication remain gated.

## Phase 1 — accepted desktop foundation

Tauri/React/TypeScript/Tailwind shell, responsive navigation, local theme persistence, SQLite connection/migrations, restricted commands, single instance, native validation. Evidence: DESKTOP_PHASE_1_REVIEW.md (historical), DESKTOP_NATIVE_ACCEPTANCE.md (native follow-up). Historical reports describe the rules/status at their dates.

## Phase 2 — business schema and exact core (accepted)

First update planning docs and contributor instructions to confirmed rules. Add immutable migration 002 retaining settings/001. Implement 13 business/catalog/audit tables, exact integer Rust domain functions, reference repositories, aggregate saves, atomic finalization and idempotent payroll synchronization, finalized edit history and historical retrieval. Snapshot labels and percentage actually used. No full business UI or expanded IPC is required.

Acceptance: synthetic Rust tests cover decimal parsing/scales/bounds, fractional tons/rates, half-cent below/at/above, zeros, 0%/100%, load sums, ADD/DEDUCT/mixed adjustments, driver ACH and owner no-ACH, multiple owner trucks/default-driver historical independence, custom dates/invalid periods, migration 1→2 and reopen/failure preservation, FK/CHECK integrity, immutable audit, stale versions, finalization/idempotence, linked payroll and rollback after mid-transaction failure. Run frontend typecheck/lint/tests/build, Rust fmt/check/tests/Clippy denied warnings and Tauri production build without publishing. Report actual results and stop for review.

## Phase 3 — SQLite-safe backup/recovery (accepted)

Must finish before real payroll records. Approve destination/cadence/retention/encryption. Implement Online Backup API snapshots, validation/manifests/completion, manual/automatic generations, honest status and guided restore with rollback. No blind active-file copies.

Acceptance: active WAL backup, offline clean-machine restore, interruption/disk-full/permissions/stale-sidecar handling, corrupt/newer schema rejection and last-good retention. Failed operations preserve existing data. Use synthetic fixtures; run applicable checks.

## Phase 4 — business UI (accepted)

Directory create/edit/archive, separate driver/owner payroll forms/history, settlement load/adjustment editor, typed narrow Rust commands, exact decimal string validation, simple audited edits and finalization. Rust owns financial formulas. USD display and explicit confirmation of signed negative payouts are approved. Completeness, duplicate/import and eventual payment conventions remain later review decisions.

Acceptance: offline create/edit/reopen/filter, accessible responsive/theme workflows, preserved input on errors, version-conflict handling, exact service/UI agreement and safe recovery. Run full frontend/native checks.

## Phase 5 — reports/exports/print (accepted)

Canonical snapshot datasets, PDF/print then CSV/Excel/DOCX, scoped destinations, exact historical values and safe spreadsheet text.

Acceptance: rendered report comparison, pagination/fonts, offline operation, exact totals and no partial artifacts on failure.

## Phase 6 — optional Sheets/email outputs (accepted; provider validation pending)

Reuse saved canonical Phase 5 reports. Customer-owned Google Desktop OAuth uses PKCE/state and a short-lived loopback callback. Separate Sheets and Gmail send scopes plus email identity; DPAPI current-user encrypted credentials remain outside SQLite/reports/backups. Rust fixed HTTPS endpoints, timeouts, no automatic retries, durable non-secret operation markers and one-use send confirmations. Sheets creates a new spreadsheet; Gmail sends generated PDF/XLSX/DOCX (CSV also supported). Settings connects/clears accounts; report actions review before external submission. Local workflows remain offline and do not require integration setup. No migration or financial audit write.

Acceptance: fake transport/vault tests for disconnected/corrupt/revoked credentials, PKCE/state/scope contracts, historical values/draft/owner no-ACH, exact attachment equality, cancellation, non-idempotent failure ambiguity, rate limits, restart duplicate prevention, metadata write failure and unchanged business/audit data. Run all strict frontend/native checks, isolated native UI and an interactive synthetic DPAPI probe. Live Google/email smoke is pending dedicated explicitly configured test accounts; never fabricate success or use production credentials without approval. Stop for review.

## Phase 7 — Windows release readiness (authorized)

Installer/WebView2/signing/update ownership, physical DPI hardware matrix, offline install/recovery/upgrade/uninstall preservation. Import requires separately reviewed mapping/reconciliation. Acceptance includes clean-machine offline drills and business sign-off. No automatic publishing.

## Phase 8 — optional routing/maps (requires approval)

Provider-neutral customer-owned routing adapter and reviewable draft candidates with offline manual fallback. No Google/email authorization implies routing, cloud synchronization or multi-user support. Imports require separate mapping/reconciliation review.

Stop after each approved phase. Phase 2 does not implement backups, exports, complete UI, integration or distribution and must not be used for real business records.


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

## Phase 5 approved implementation boundary

Implement Weekly Settlement Detail, separate Driver Payroll and Owner Payroll, Payroll Summary and Settlement History. Date ranges overlap saved periods; driver/owner filters apply to their respective sections and truck filters apply to both. One typed Rust report model reads saved SQLite inputs or finalized snapshots. Render CSV, XLSX, PDF, DOCX and local PDF print preview from that model; never calculate old totals again or read current catalogs to replace finalized labels. Drafts are visibly marked and unknown totals remain unavailable. Preserve current integer scales and provisional policy.

Rust owns native save selection, sanitized default names, same-directory staging, synced complete writes and no-clobber publication. New files only; cancellation/error publishes no partial report. No export metadata is required, so schema remains 4 and migrations 001–004 remain immutable. Add only preview_report, export_report and preview_report_print for the local main window. Local print opens a generated PDF in the configured PDF viewer; physical printing remains a deliberate viewer action.

Acceptance: historical/current-catalog independence, exact negative/large values, draft behavior, owner no-ACH, CSV quoting/injection protection, real XLSX/DOCX packages, Unicode PDF extraction/rendering, repeated PDF headings/pagination, no-clobber and failed-stage cleanup, cross-format saved totals, real Windows picker/cancellation/collision/print/restart checks with isolated fixtures. Run full frontend and locked native verification. Record unavailable viewer/rendering checks explicitly. Stop for review; no integrations, release installer, signing or distribution.


## Phase 7 release candidate boundary

The user explicitly authorizes per-user Windows packaging and local release validation, export-folder settings and an offline User Manual. Historical exclusions above remain evidence of earlier scopes. No routing/maps, new financial rules, cloud synchronization, real data or public publishing. Production identity remains com.twx.payroll; version 0.2.0; schema stays 4 with immutable migrations 001–004. An installed release still needs independent-machine upgrade/recovery/uninstall verification; this developer host cannot supply that evidence.

Export Settings is Rust-owned machine-local configuration (export-directory.json), outside SQLite so restoring business data does not resurrect an obsolete machine path. Absence means the current OS/Tauri-resolved Documents directory. Only native selection can set a custom path. React accepts/display status and invokes named get/select/reset/open commands; it supplies no path. Ordinary export stages/syncs completed bytes and atomically publishes without clobbering, trying -2, -3 suffixes on collisions. Save As remains a distinct named command. Missing/unwritable destinations fail explicitly with no fallback; backup settings and financial/audit rows are untouched.

Print cache cleanup is seven-day, app-owned UUID PDF files only, with Windows exclusive DELETE access and DELETE_ON_CLOSE to prevent a probe/delete race. Open/locked files remain. A serialized cache operation enforces a 100-file/100-MiB quota and refuses new previews when full; no potentially open file is forcibly removed. Ordinary exports remain available.

Operation journal policy retains all entries, capped at 8 MB; SUCCESS/UNCERTAIN/SENDING digests are never automatically pruned or reset. At capacity new external actions fail before a provider send. Users verify Sheets/Sent Mail and use local exports; no uncertain operation automatically repeats and no reconciliation override is shipped. A lost journal/new PC requires remote review before new outputs.

The User Manual is bundled JSON rendered through simple React headings, local section links and paragraphs. USER_GUIDE.md is generated from that same content and checked in automated verification. No network/documentation service is involved. USER_GUIDE, DATA_RECOVERY, RELEASE_NOTES, RELEASE_CHECKLIST and SOURCE_HANDOFF describe the candidate and actual remaining validation gates. NSIS currentUser with offline WebView2 packaging was chosen for this known-user application; signing remains optional and the candidate is unsigned. No automatic updater/deployment is added.

Acceptance: strict frontend/native suites, export settings persistence/collision/failure/independent-backup tests, manual keyboard/offline/theme/axe checks, production and installer builds, isolated native report/recovery checks. Separately record independent installed app identity/path, runtime-only prerequisites, upgrade and uninstall preservation when a separate environment exists. Do not claim those blocked checks passed. Stop for review after Phase 7.

## Final release-validation scope

Phase 7 implementation accepted; no next feature phase authorized. Correct the accidental-loss uninstall option using a pinned NSIS template, rebuild and rerun all applicable suites, regenerate/audit handoff hashes, and keep independent install/upgrade/uninstall/recovery acceptance pending until an independent environment exists. Do not reinterpret historical local diagnostic runs as independent release acceptance. Current evidence is in RELEASE_VALIDATION.md.
