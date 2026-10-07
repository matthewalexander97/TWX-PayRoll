# TWX Payroll — offline backup and recovery

Keep this guide with your backups and a compatible application copy. No developer account, server, domain or cloud API is needed. Phases 1–6 are accepted; the Phase 7 release candidate still needs independent Windows release acceptance. See DATA_RECOVERY.md and RELEASE_CHECKLIST.md.

## Preserve your workspace

In Settings → Backup & recovery, choose a backup folder and select Backup Now. Wait for “Backup created and validated” and a SUCCESS history entry. Keep each `.sqlite3` file with its adjacent `.sqlite3.json` completion manifest. The snapshot includes the entire database: records, historical labels, audit events and settings. Backups are not encrypted; protect their storage using your Windows account and appropriate drive protection.

The live database always remains beneath your Windows LocalAppData folder at `com.twx.payroll\data\twx-payroll.sqlite3`. Never move it into a synchronized folder. A selected backup destination may be an external drive or an already synchronized folder. A successful local save does not confirm a cloud upload. A copy on the same physical drive cannot protect against that drive failing.

Automatic backup is enabled initially, but needs a destination. While the app is open it checks at startup and every minute. Changed business data is backed up when the last successful backup is at least 24 hours old, or no successful baseline exists. Failed attempts retry no more often than hourly. Appearance changes do not trigger repeated automatic copies. After important work, use Backup Now if you need an immediate copy. Keep 10 successful automatic generations in the selected folder; manual and pre-restore copies remain until you deliberately manage them. History shows the latest 100 operations; older metadata remains in SQLite.

## Restore a saved copy

1. In Settings, choose Restore Backup and select the desired `.sqlite3` file. Leave its manifest beside it. The app verifies integrity, foreign keys and the exact supported TWX schema (versions 1–4). Newer unsupported versions are refused. Known older versions are migrated only in a private staged copy; the original backup is unchanged.
2. Review the name and saved time. Restore replaces the active business records and appearance. Records entered since that backup will no longer be active, but the app first keeps a complete pre-restore safety copy.
3. Select the acknowledgment and Confirm Restore. Keep the app open. On success, the workspace refreshes. Your current backup destination and automatic setting remain configured, and operation history is retained.
4. Review the result in history. To undo a successful restore, select its PRE RESTORE `.sqlite3` safety copy from the path shown in history, using the same restore flow. These copies live beneath `com.twx.payroll\recovery` in LocalAppData.

If candidate validation fails before replacement, current records stay active. If verification fails after replacement, the app restores the safety snapshot and records ROLLED BACK. Failed candidates and safety files are preserved. If the app closes during restore, reopen it: the pending recovery marker makes startup conservatively recover the safety snapshot before allowing writes. A reported recovery failure disables storage writes; keep all files and use the procedure below rather than deleting a marker or WAL file.

## If storage cannot open

This procedure preserves the entire damaged/interrupted workspace and lets the normal guided restore validate your chosen backup. Use File Explorer; no database editor or developer tools are required.

1. Close every TWX Payroll window. Do not perform file operations while the app runs. Keep your original backup files untouched.
2. Open your Windows LocalAppData folder in File Explorer (the address `%LOCALAPPDATA%`), then locate `com.twx.payroll`. Use the actual location previously shown in Settings/history if your development host redirects LocalAppData. In the Codex-hosted development smoke test Windows resolved it beneath `LocalAppData\Packages\OpenAI.Codex_2p2nqsd0c76g0\LocalCache\Local`; this is an environment observation, not an application dependency. Do not rename a different workspace by guessing its path.
3. Rename the **whole** `com.twx.payroll` folder to an unused name such as `com.twx.payroll-preserved-2026-10-04`. Keep everything together, including data, recovery markers, safety copies, WAL and SHM files. Do not delete or selectively copy an active database or its sidecars. If Windows says files are in use, stop and close the remaining app process before proceeding.
4. Reopen a compatible TWX Payroll application. With its original folder absent, it creates a fresh empty workspace at the normal location. In Settings choose Restore Backup and select a known successful backup from your separate destination. If necessary, select a PRE RESTORE safety snapshot in the preserved folder's recovery directory.
5. Complete the normal validation and confirmation flow. Confirm your expected saved state, then select a suitable backup destination and make a new validated backup. Keep the preserved folder and the original backups until recovery has been independently checked.

Do not place old WAL/SHM files next to a restored database. Do not overwrite or delete the preserved folder to resolve an error. If no backup passes validation, retain all copies and use a qualified recovery specialist; the application cannot promise recovery from physical media failure. A future-schema backup requires a compatible newer application, not a forced downgrade. Installer delivery and clean-machine recovery drills remain a later release gate.

## Reports and print copies

CSV, Excel, PDF and Word files are derived copies of saved records. They are not backups and cannot restore the workspace. Keep using the SQLite-safe backup/recovery workflow above. Reports retain the historical saved labels and totals; edited report files do not change the database. Print preview creates local PDF copies under application data/report-previews, excluded from the SQLite backup. These copies may contain statement data and use the same Windows-account access boundary as the workspace. Normal exports use Settings → Export Settings (Windows Documents by default) with numbered collision names. Save As is a separate one-off destination. Optional Sheets/Gmail actions send only after explicit confirmation; local reports are never automatically uploaded. Export preference, print cache, operation journal and DPAPI credentials are outside SQLite backups; choose new machine paths and reconnect on replacement-PC recovery.
