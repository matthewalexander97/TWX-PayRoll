# Final release validation — 0.2.0

2026-10-07. Phase 7 implementation is accepted. This task corrects release-specific uninstall behavior and reruns verification; no product feature, financial policy, migration or dependency version changed. **Independent release acceptance remains blocked:** the user reconfirmed that no independent Windows PC or VM is available. Nothing was published, committed or pushed.

## Completion report

| Requested item | Result |
| --- | --- |
| 1. Clean install | NOT RUN independently. The corrected installer compiled successfully but was not installed in a clean environment. Earlier developer-host installation is historical evidence only. No new SmartScreen/security-warning result is claimed. Unsigned setup may produce warnings; do not automate a bypass. |
| 2. Independent physical LocalAppData | NOT OBSERVED. Expected logical production location is `C:\Users\[redacted]\AppData\Local\com.twx.payroll\data\twx-payroll.sqlite3`. This host still redirects physically to `C:\Users\[redacted]\AppData\Local\Packages\OpenAI.Codex_2p2nqsd0c76g0\LocalCache\Local\com.twx.payroll\data\twx-payroll.sqlite3`. Fresh read-only inspection: schema 4, integrity OK, no FK errors, dark theme, no business records. That is not an independent path result. |
| 3. WebView2 | Existing developer-host Runtime allows local startup. NSIS still embeds the Evergreen offline installer. Runtime-absent provisioning remains UNVERIFIED; no shared Runtime was removed for testing. Previously observed application SQLite runtime remains 3.53.2; no SQLite dependency change. |
| 4. Business workflow | Fresh Rust suite PASS, including exact calculation, draft/finalize/linkage/ACH/reopen/history cases. Earlier isolated Phase 7 native fixture verified loads $50/$80, $130 gross, 85% payout $110.50, -$10+$2 adjustments, settlement $102.50, ACH $1.25 and payroll $103.75. This task did not repeat that workflow in an independent installed release. |
| 5. Export folders | Fresh automated tests PASS: Documents resolver/default, custom persistence, reset, four formats, Save As separation, collisions, missing/resolver failure and financial/backup independence. Earlier local native picker/export/restart/reset evidence is retained in the Phase 7 review. Independent packaged Documents/custom/Save As/reset workflow remains UNRUN. |
| 6. Backup/restore | Fresh automated recovery suite PASS, including business data restoration, PRE_RESTORE preservation, migration, rollback and interrupted restart. Previous isolated native snapshot/restore/restart passed. The requested modify/restore/restart drill on an independent installed release remains UNRUN. |
| 7. Upgrade | Fresh migration tests PASS. Earlier-release installer → corrected 0.2.0 upgrade on populated independent Windows remains UNRUN. No database was deleted or reset to obtain a pass. |
| 8. Uninstall/reinstall | UNRUN. No independent environment. The earlier computer-use attempt to launch the installed uninstaller was blocked by product policy; no alternate method was used to bypass that restriction. Static checks and successful compilation are not runtime data-preservation acceptance. |
| 9. Destructive uninstall option | FIXED. Pinned custom NSIS template removes the checkbox, show/leave callbacks and complete conditional app-data deletion block. The confirmation explains retention. Interactive, passive, silent and upgrade uninstall have no app-data deletion code. Program files, shortcut and uninstall-registration removal remain. Source and generated-script policy checks PASS. |
| 10. Office visual review | UNRUN. Excel/Word executable and registered App Paths checks found neither; no Office installation was attempted. Previous four-format content verification is not a visual Office pass. |
| 11. Unavailable/read-only exports | Prior local missing-directory simulation passed without fallback/financial mutation. Fresh failure/no-clobber tests PASS. Genuine removable-device disconnect and Windows ACL read-only tests remain UNVERIFIED; no permissions/security settings were altered. |
| 12. Historical rounding | No historical TWX statements supplied. Existing provisional exact-integer half-up policy retained unchanged. Arithmetic tests passing do not establish comparison to actual TWX paperwork. |
| 13. Google/Gmail | Provider validation pending; no dedicated credentials supplied. Fresh fake/contract tests PASS. No customer credential, Sheet or email transmission used. |
| 14. Final installer SHA-256 | `db835c5e35b0e020e14d7602c9fbd08984296204e6bb2c0bfefe30853877de8f` |
| 15. Final executable SHA-256 | `6d912c218bbbd557cbdf342014813eb774591bbdb0c4f203ec50d8d32ace30d4` |
| 16. Full verification | 44 frontend tests and 68 Rust tests pass, plus typecheck/lint/format, foundation/manual/installer guards, browser UI/axe, strict Clippy, optimized build, NSIS compilation and handoff checks. Detailed ledger below. |
| 17. Remaining release blockers | Independent clean install/path/runtime independence; missing-WebView2 path; populated upgrade and uninstall/reinstall preservation; independent recovery/business/export workflow. Physical failure/ACL, Office visual and historical rounding sign-off remain pending. Optional provider validation is separate. Do not treat this report as full release acceptance. |

## Release-specific change and provenance

Tauri supports a custom NSIS template through `bundle.windows.nsis.template`; installer hooks alone do not replace its confirmation-page checkbox code. We use the [documented template mechanism](https://v2.tauri.app/distribute/windows-installer/#installer-template). `src-tauri/installer/installer.nsi` derives from the official `tauri-cli-v2.12.0` template; the adjacent README records source URL and upstream SHA-256. Only the data-deletion UI/code was removed and preservation wording added. The existing per-user mode, production identifier, offline WebView2, downgrade prohibition, resource/shortcut handling and update behavior remain.

The removed block also contained cleanup of remembered installer-location/language metadata; that metadata is retained with the workspace. Windows Add/Remove Programs uninstall registration is still removed by the ordinary uninstall section. No new app-data destruction command or cleanup feature was added.

`scripts/verify-installer.mjs` checks both the source and (with `--generated`) Tauri-rendered script. It refuses the old checkbox, recursive directory deletion, application-data Delete/RmDir statements, missing preservation message or missing ordinary executable/uninstaller/registration cleanup. This guards the release policy but cannot prove operating-system behavior. Review/rebase this pinned template deliberately when upgrading Tauri.

Material files: installer template/README; Tauri configuration; new installer verification script; package scripts; handoff guard/allowlist; bundled manual JSON and generated USER_GUIDE; AGENTS/ARCHITECTURE/implementation-plan status; DATA_RECOVERY, RELEASE_NOTES, RELEASE_CHECKLIST, SOURCE_HANDOFF; historical Phase 7 review banner; this report. Both lockfiles and migrations 001–004 remain unchanged. No financial/backend workflow changes were made.

## Fresh verification ledger

| Command/check | Result |
| --- | --- |
| `npm.cmd run verify:frontend` | PASS: strict TypeScript, ESLint, 44 Vitest tests in 7 files, foundation, manual alignment, installer-source policy, production frontend, Edge layout/navigation/keyboard/themes/offline manual/axe at 1440/1024/800/375, Prettier. Log `work/release-validation-frontend.log`. |
| `node work/cargo-relay.mjs --native` → `scripts/verify-native.ps1` | PASS with existing pinned host environment; all five commands below. Log `work/release-validation-native.log`. |
| `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` | PASS. |
| `cargo check --manifest-path src-tauri/Cargo.toml --locked` | PASS. |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked` | PASS: 68, zero failed/ignored; empty binary/doc targets also pass. Includes financial, migration, recovery, report/export, integrations, cache and settings cases. |
| `cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings` | PASS, no suppressions. |
| `npm.cmd run tauri -- build --no-bundle -- --locked` | PASS: optimized Windows executable. |
| `node work/cargo-relay.mjs --tauri build --bundles nsis -- --locked` | PASS: corrected per-user NSIS setup, 209.62 MiB. Log `work/release-validation-installer.log`. |
| `node scripts/verify-installer.mjs --generated` | PASS for template and generated NSIS script. Standard `npm run build:installer` now includes this check; handoff repeats it. |
| `npm.cmd run format:check`; `node scripts/manual-guide.mjs --check` | PASS after final script/document-source edits. |
| Native local optimized executable launch | PASS: native payroll shell opened using preserved dark theme and existing workspace. The uniquely named test copy matched the optimized pre-bundle executable. Tauri subsequently patched bundle-type metadata into the final executable; final bundled setup was not installed/launched here. No independent-runtime claim. User input was detected during Settings navigation, so no further input was applied. |
| `node work/phase7-observe.mjs release-validation` | PASS read-only schema/integrity/FK/count/theme checks; ordinary business tables remained empty, isolated earlier fixture retained 1 settlement/2 payroll/6 audits. No live DB copy. |
| Office App Paths and executable discovery | No Excel/Word/LibreOffice found; visual check unrun. |
| `Get-FileHash` / `Get-AuthenticodeSignature` | Final hashes above; both application and installer `NotSigned`. |
| `npm.cmd run handoff` and independent `Get-FileHash` manifest loop | PASS: 20 allowlisted files plus SHA256SUMS.txt; all 20 copied-file hashes verified. Generated-installer policy guard also passed. |

Environment: Node 24.13.0/npm 11.6.2, Rust 1.98.1, installed MSVC/Windows SDK. `. ./work/phase2-env.ps1`, `CARGO_NET_OFFLINE=true` and the existing ignored registry relay select this host's cached toolchain. These tools are not shipped or runtime dependencies. One approved network grant retrieved the official pinned template with TLS validation. Standard maintainers use the commands in SOURCE_HANDOFF.

Initial new installer guard failed because its section locator expected quoted `Section "Uninstall"`; the actual upstream syntax is `Section Uninstall`. Corrected the locator with an explicit existence assertion and reran the full frontend suite successfully. Initial template fetch was denied by network permissions; the scoped grant allowed the official download. No product failure was suppressed. npm update notices and Node experimental SQLite warnings are tooling-only.

## Handoff and next acceptance action

`outputs/release/0.2.0` contains the corrected setup/executable, guides, current and historical reviews, dependency notices/corresponding sources and SHA256SUMS.txt. Current hashes supersede the earlier 0.2.0 candidate despite the unchanged requested version. Keep the hashes with the selected release; do not confuse the earlier setup with this correction. The package excludes source credentials, test databases/reports, debug configuration and build/development caches.

On an independently supplied Windows environment, follow RELEASE_CHECKLIST using only this handoff. Capture actual installation/security observations, physical path, runtime availability and the synthetic workflow; then complete upgrade and preservation/recovery drills. No next feature phase or real-data acceptance is implied. Manual Git commands remain in the Phase 7 review; no Git changes were attempted.
