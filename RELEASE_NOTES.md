# TWX Payroll 0.2.0 — release candidate

This is a local, single-user Windows x64 release candidate. Release acceptance remains pending the independent Windows installation/upgrade/recovery drill in RELEASE_CHECKLIST.md. No public distribution has been performed.

New in this candidate:

- A per-user NSIS setup executable with the Microsoft Evergreen WebView2 offline installer included.
- Settings → Export Settings: Windows-resolved Documents by default, native Change Folder, Reset to Documents and Open Export Folder.
- Normal exports write complete files to the configured folder. Collisions add -2, -3, etc. Save As… remains a separate one-off destination.
- A bundled, offline User Manual with fourteen sections, keyboard navigation and light/dark/system support.
- Bounded print-preview storage: seven-day cleanup only with exclusive Windows delete access; maximum 100 files or 100 MiB. Open files are retained; a full cache blocks new previews, not ordinary PDF exports.

The production identity remains com.twx.payroll. Schema remains 4; migrations 001–004 and approved financial formulas are unchanged. Exact integer arithmetic retains the provisional per-load/percentage half-up rule, including negative payouts and explicit confirmation. Historical TWX calculation samples were not supplied for comparison.

Optional Google Sheets/Gmail features remain provider-validation pending; no dedicated credentials were supplied. Local bookkeeping does not require them. Microsoft Office visual XLSX/DOCX review remains pending; PDF is the verified print representation.

The installer and executable are unsigned. Windows may show an unknown-publisher or SmartScreen warning. Verify the installer SHA-256 against a trusted copy of SHA256SUMS.txt and confirm the source; a hash from an untrusted package is not proof of publisher identity. Future signing requires an appropriate Windows code-signing certificate/service controlled by the release owner, secure keys and timestamped Authenticode signing of both application and installer. No certificate was purchased.

Installation is intended for currently supported Windows 11 x64. Windows 10 or other editions/architectures require a separately recorded support/validation decision. The app uses Evergreen WebView2; the installer contains Microsoft's runtime installer for offline provisioning when absent. The runtime may need Windows repair/support if blocked by policy. Offline installation of an absent runtime has not been independently tested here. Evergreen updates remain Microsoft's responsibility when the machine is online.

Upgrade by closing TWX, making a validated backup, then running the newer setup under the same Windows user. Do not delete the database to upgrade. The corrected installer removes program files/shortcuts while retaining the entire application-data workspace. The former Delete app data checkbox and its deletion code have been removed, including for silent/passive uninstall. Runtime uninstall/reinstall preservation still requires acceptance testing. Older installers may still offer a destructive checkbox; leave it unchecked.

See USER_GUIDE.md for workflows, DATA_RECOVERY.md for recovery, and RELEASE_CHECKLIST.md for acceptance evidence and outstanding checks.

Release validation correction (2026-10-07): Phase 7 implementation accepted. The 0.2.0 installer has been rebuilt with a pinned, data-preserving NSIS template. Use the newly generated SHA256SUMS.txt to distinguish it from the earlier 0.2.0 candidate. Independent-machine acceptance remains pending; no environment is available.
