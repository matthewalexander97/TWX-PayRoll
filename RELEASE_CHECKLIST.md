# TWX Payroll release acceptance

Candidate: 0.2.0, Windows x64, production identity com.twx.payroll. Do not treat an unchecked step as passed. The developer machine is not a clean machine. The user confirmed no separate Windows PC/VM is available for this phase.

## Before handing over for real bookkeeping

- [ ] On an independent supported Windows account/PC, verify supplied SHA-256 and install the unsigned NSIS setup.
- [ ] Launch the installed Start Menu shortcut. Confirm version and schema 4 in Settings; record the actual normal LocalAppData path (redact username in shared evidence).
- [ ] Verify no Node, npm, Rust, Cargo, MSVC, source repository, Codex or development scripts are needed.
- [ ] Test the embedded offline WebView2 provisioning on a machine where Runtime is absent; do not uninstall a shared runtime from the development PC as a test.
- [ ] Use synthetic references, a custom period, two loads, ADD/DEDUCT and a configured driver percentage. Finalize, check linked Driver Payroll and ACH. Verify negative-payout confirmation separately.
- [ ] Verify light/dark/system, close/reopen and persistence.
- [ ] Export PDF/XLSX/CSV/DOCX to resolved Documents and a custom folder; check Save As and -2 collision naming. Test missing, read-only and disconnected folders without fallback.
- [ ] Create a manual backup and automatic baseline to a temporary test destination. Restore, inspect PRE RESTORE safety copy, restart, verify records/history/totals/integrity/foreign keys.
- [ ] Install an earlier compatible build, create synthetic data, close, install 0.2.0 over it and confirm data/theme/audits/backup destination and financial values unchanged. Existing migration tests are not a substitute for installer upgrade testing.
- [ ] Uninstall/reinstall with synthetic records. Confirm the complete LocalAppData workspace remains and is reopened unchanged. Never delete data to make the upgrade succeed.
- [ ] If possible, restore a copied valid snapshot on a replacement Windows account/PC and reconnect optional integrations. DPAPI secrets must not be copied as portable credentials.
- [ ] Review XLSX and DOCX in intended Microsoft Office applications, including long notes and multi-page content. PDF is primary print output.
- [ ] Obtain representative historical TWX examples and compare the documented provisional rounding; record sign-off or a separately approved versioned correction.

## Optional provider acceptance

Dedicated Google credentials were not supplied. OAuth connect, actual new Sheet, Gmail test delivery/attachment and disconnect/revoke remain unrun. This does not block local operation. Never use production credentials merely for test acceptance. Mock contracts and local failure behavior are recorded in the phase review.

## Release engineering

Run npm ci, npm run verify:frontend, scripts/verify-native.ps1, npm run build:installer and npm run handoff. Both lockfiles must remain included. Record hashes and command results in docs/DESKTOP_PHASE_7_REVIEW.md. Check that no smoke identifier/debug browser argument, credential, database, report fixture or build cache enters the handoff. Preserve bundled dependency/font notices.

The installer is per-user NSIS, selected for a single known Windows user without routine administrator access. MSI/WiX adds no needed enterprise deployment behavior here. Tauri's offlineInstaller WebView2 option increases package size but avoids a first-install network dependency. See [Tauri Windows packaging](https://v2.tauri.app/distribute/windows-installer/) and [NSIS configuration](https://v2.tauri.app/reference/config/#nsisconfig).

Unsigned artifacts may trigger Windows reputation warnings. Do not automate bypass of security warnings. Confirm source and checksum with the release owner. Future signed delivery needs owner-controlled Authenticode signing and timestamping; no signing service or public publishing is part of this candidate.

## Phase 7 developer-host evidence (2026-10-07)

Frontend verification passed (44 tests); native verification passed (68 tests); final production executable and NSIS installer built. Isolated native export preference/restart/reset/missing-folder, all four formats, Save As, collision, IPC denial, manual accessibility and backup/restore checks passed. These results do not check the independent-machine boxes above.

Uninstall launch was blocked by the computer-use tool's product policy. Uninstall/reinstall remains unrun. The generated uninstaller offers an optional Delete app data checkbox; leave it unchecked to preserve the workspace. Final resource-complete installer reinstallation, physical read-only/disconnected-device exports and independent upgrade/recovery remain pending. Full evidence: docs/DESKTOP_PHASE_7_REVIEW.md.
