# Desktop Phase 7 — release candidate review

Historical implementation review: Phase 7 implementation is now accepted. The uninstall option and artifact hashes below describe the original candidate. See [final release validation](RELEASE_VALIDATION.md) for the corrected installer, current hashes and remaining acceptance gates.

Date: 2026-10-07. Version 0.2.0, Windows x64. Implementation and available automated verification are complete. Release acceptance is **pending** the checks explicitly identified below. No subsequent phase, public publication, Git commit or push was performed.

## Requested acceptance report

| Item | Result and evidence |
| --- | --- |
| 1. Installer | NSIS currentUser; stable production identifier `com.twx.payroll`; Start Menu shortcut; no routine administrator requirement. Downgrades disabled. Initial candidate installed and launched on this developer host. Final installer built with complete notices/source resources. |
| 2. Signing | Application and setup are unsigned (`NotSigned`). Future owner-controlled Authenticode signing and timestamping documented; no security warning bypass. |
| 3. WebView2 | Microsoft Evergreen offline installer embedded; downloaded runtime installer signature validated. Provisioning on a runtime-absent independent PC remains unrun. |
| 4. Clean machine | Unrun: user confirmed no separate Windows PC/VM. Developer-host installation is not clean-machine acceptance or proof of operation without development tools. |
| 5. Independent data path | Not observed. Logical path is `C:\Users\[redacted]\AppData\Local\com.twx.payroll\data\twx-payroll.sqlite3`. This host actually redirects to `C:\Users\[redacted]\AppData\Local\Packages\OpenAI.Codex_2p2nqsd0c76g0\LocalCache\Local\com.twx.payroll\data\twx-payroll.sqlite3`. Diagnostic uses the separate `.phase7-smoke` identifier. SQLite runtime 3.53.2; schema 4; integrity OK; no foreign-key violations. |
| 6. Upgrade | Migration tests pass; earlier compatible installer-to-installer upgrade with populated synthetic records remains unrun. A preserved 0.1.0 executable alone is not installer upgrade evidence. |
| 7. Uninstall/reinstall | Unrun. Computer-use product policy blocked launching the installed `uninstall.exe`; no alternate launch bypass attempted. Baseline was captured first. Generated NSIS script preserves LocalAppData by default, but offers an optional **Delete app data** checkbox: leave it unchecked. Script inspection does not establish runtime preservation. |
| 8. Recovery | Isolated native fixture: manual validated snapshot, native backup selection, explicit restore, PRE_RESTORE safety copy, automatic baseline and actual close/relaunch passed. Saved financial history unchanged: one settlement, two payroll records, six audit events. Replacement-PC restore remains unrun. |
| 9. Google/Gmail | Dedicated credentials unavailable; live OAuth, Sheet creation, Gmail delivery/attachment and disconnect/revoke remain unrun. Fake/contract tests pass. No provider transmission made. |
| 10. Office | Current native CSV/XLSX/DOCX/PDF parsed against canonical saved report: all 19 rows agree, PDF two pages. Intended Microsoft Office visual review remains pending; structural/content checks are not Office rendering acceptance. |
| 11. Export folder | Rust resolves Documents from Windows/Tauri at use time. This host resolves `C:\Users\[redacted]\Documents`. Custom folder selected in native picker, persisted in independent machine-local `export-directory.json`, retained across an actual process restart. Reset to Documents passed. Backup destination and financial/audit data remained independent. |
| 12. Save As | Separate native picker saved a PDF to a different test directory without changing the export preference. Ordinary exports use the saved preference; collisions publish `-2`, etc., without overwrite. |
| 13. Export failures | Missing/disconnected simulated by temporarily moving only the workspace test directory: explicit error, no fallback and no financial changes. Invalid configuration, unavailable Documents resolver, failure cleanup and no-clobber tests pass. Physical Windows read-only-directory ACL and genuine disconnected-device tests remain pending; not claimed passed. |
| 14. Manual | Fourteen bundled sections, top-level User Manual navigation, local anchors and return link. USER_GUIDE generated from the same source. Native keyboard/light-dark/axe at 1440/800/375 widths passed; browser offline check and 1024-width coverage passed. |
| 15. Preview cache | Seven-day cleanup restricted to app-owned UUID PDFs. Exclusive Windows DELETE access and DELETE_ON_CLOSE retain open files without a check/delete race. Serialized 100-file/100-MiB quota refuses new previews; ordinary PDF export remains available. Tests pass including open/closed aged files. |
| 16. Operation journal | Existing 8-MB cap retains all entries. No automatic pruning/retry of SUCCESS, UNCERTAIN or SENDING. At capacity, stop new provider operations before sending; preserve journal and verify remote outcomes. No reconciliation override introduced. |
| 17. Financial rounding | No formulas, scales, schema or migrations changed. Existing exact-integer provisional half-up rules and tests retained. Representative historical TWX samples/sign-off still unavailable. No claim of business validation against historical paperwork. |
| 18. Documentation | USER_GUIDE, DATA_RECOVERY, RELEASE_NOTES, RELEASE_CHECKLIST, SOURCE_HANDOFF, THIRD_PARTY_NOTICES; planning/recovery/integration/dependency documents updated. |
| 19. Artifacts | Final release executable and NSIS installer; allowlisted `outputs/release/0.2.0` handoff with guides, notices, required corresponding sources and SHA256SUMS. No database, private report, credentials, diagnostic executable or build cache included. |
| 20. Hashes | Final setup SHA-256 `e4a663ff088df8044288331c945c47c6005a66cbf3e83d95df92e298f5420034`; executable `f70257fcf2a58803523e23e97a06227defe23937d75f21f4d3f30087cab7ecd8`. All handoff files individually hashed in SHA256SUMS.txt. |
| 21. Verification | See command ledger below. 68 Rust tests and 44 frontend tests pass. |
| 22. Remaining gates | Independent install/path/runtime/upgrade/recovery; blocked uninstall/reinstall; final installer installed-runtime check; physical read-only/disconnected export destination; Office visual review; historical rounding sign-off. Live Google validation remains optional-provider pending. Unsigned status disclosed. |
| 23. Git | Both lockfiles present; source remains untracked in an unborn `master` checkout, no remote configured at inspection. Manual checkpoint commands below. No filesystem security changes. |

## Material implementation changes

- `src-tauri/src/export_settings.rs`, commands/dialogs/lib/build/capability declarations: bounded config reads, native selection, checked canonical paths, independent preference, atomic no-clobber numbered publishing. Exactly 36 named local-main permissions; no generic SQL, filesystem, shell or remote-page IPC.
- `src-tauri/src/preview_cache.rs`, report publication and integration journal messaging: bounded conservative retention and actionable capacity failures.
- `src/features/settings/ExportSettings.tsx`, export contract and report actions: configured folder, reset/open, explicit one-off Save As.
- `src/features/manual/`, navigation/App/layout: bundled responsive manual, keyboard navigation, matching generated guide and tests.
- `scripts/manual-guide.mjs`, `verify-release-native.mjs`, `collect-notices.mjs`, `prepare-handoff.mjs`, foundation/UI checks: alignment, native diagnostics and strict release allowlist.
- Tauri configuration/manifests/lockfile app version: 0.2.0 packaging. No direct dependency upgrade. `third-party/notices` and `third-party/source` contain redistribution material, not development caches.
- Planning and release documents listed above. `legacy/web` remains excluded from desktop scripts/builds.

## Command and verification ledger

The development host uses `. ./work/phase2-env.ps1`, `CARGO_NET_OFFLINE=true` and the ignored `work/cargo-relay.mjs` adapter for its pre-existing registry transport. Cargo.lock retains standard crates.io sources. Normal developer commands are in SOURCE_HANDOFF.

| Command/check | Final result |
| --- | --- |
| `npm.cmd run verify:frontend` | PASS: typecheck, ESLint, 44 Vitest tests in 7 files, foundation checks, manual alignment, production frontend, Edge UI/axe/offline manual, Prettier. Log `work/phase7-frontend-release.log`. |
| `node work/cargo-relay.mjs --native` / `scripts/verify-native.ps1` | PASS; runs the five native commands below. Log `work/phase7-native-release.log`. |
| `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` | PASS. |
| `cargo check --manifest-path src-tauri/Cargo.toml --locked` | PASS. |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked` | PASS: 68 tests, zero failed/ignored; financial, migration, backup/recovery, reporting, integration fakes, export/cache tests included. |
| `cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings` | PASS. |
| `npm.cmd run tauri -- build --no-bundle -- --locked` | PASS: production native executable. |
| `node work/cargo-relay.mjs --tauri build --bundles nsis -- --locked` | PASS: final 209.61-MiB setup; log `work/phase7-installer-final.log`. |
| `node work/cargo-relay.mjs --tauri build --debug --no-bundle --config work/phase7-native-config.json -- --locked` | PASS: isolated diagnostic build. Native feature checks used this diagnostic, not a claim that debug and release executables have the same hash. |
| `node work/phase7-fixtures.mjs` | PASS: 14 synthetic financial/reference/linkage/theme/denied-IPC checks. |
| `node scripts/verify-release-native.mjs folder`, `verify`, `backup-folder`, `backup`, `restore-select`, `restore`, `save-as`, `manual`, `open`, `restart`, `missing`, `reset` | PASS. Native picker operations assisted through Windows UI; actual process close/relaunch preceded restart assertion. Missing-folder simulation restored the workspace directory in finally. |
| `node work/phase7-observe.mjs after-restart` and `before-uninstall` | PASS read-only SQLite integrity/FK/schema/count/history observations. Node experimental SQLite warning is a QA-tool warning, not application runtime. |
| Bundled Python `scripts/verify-report-files.py work/phase7-exports` | PASS: 19 rows across all four native-generated formats. Initial custom PYTHONPATH failed access to typing_extensions; removing the unnecessary override used bundled libraries successfully. |
| `npm.cmd run format:check` | PASS after native harness formatting. |
| `npm.cmd run handoff` | PASS: 19 allowlisted files plus SHA256SUMS.txt. Independent PowerShell Get-FileHash loop verified all 19 copied files against the manifest. |
| `Get-FileHash` and `Get-AuthenticodeSignature` on final setup/executable | Hashes above; unsigned status verified. |
| Native installer launch | Initial candidate installed and opened the existing empty-business production workspace. Final resource-complete installer built, but final reinstall blocked as described above. |

Earlier failures were corrected, not suppressed: cache-test handle access, clippy needless borrow, locked running build executable, default NSIS cache access (supported project-local tools option selected), and formatting. Initial diagnostic launches resolved the installed app instead of the intended filename; a uniquely named isolated diagnostic corrected that before fixtures ran. No customer business data was entered. No fresh npm audit or external vulnerability certification is claimed.

## Manual Git checkpoint

Run from a normal developer PowerShell in this repository after reviewing the files. These commands have **not** been executed here. They checkpoint the complete currently untracked project, not merely a Phase 7 diff. Review legacy source before including it; caches and output data are excluded by .gitignore.

```powershell
git status --short
git switch -c codex/phase-7-release-candidate
git add .gitignore AGENTS.md ARCHITECTURE.md README.md DATA_RECOVERY.md RELEASE_CHECKLIST.md RELEASE_NOTES.md THIRD_PARTY_NOTICES.txt USER_GUIDE.md docs eslint.config.mjs index.html package.json package-lock.json rust-toolchain.toml scripts src-tauri src third-party tsconfig.json vite.config.ts legacy
git diff --cached --stat
git diff --cached --check
git diff --cached
git commit -m "Prepare TWX Payroll desktop Phase 7 release candidate"
git remote add origin https://github.com/matthewalexander97/TWX-PayRoll.git
git push -u origin codex/phase-7-release-candidate
```

If a remote or branch has since been created, inspect it and use that verified configuration rather than rerunning creation commands. Do not stage work, outputs, databases, private exports, credentials or build directories. Stop for Phase 7 review; no next phase is authorized by this report.
