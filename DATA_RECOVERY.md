# TWX Payroll — keep access to your records

You do not need the developer, a domain, server or online account to recover a supported TWX backup.

## Make a portable backup

Open Settings → Backup & recovery, choose a protected destination and select Backup Now. Wait for SUCCESS. Keep the complete .sqlite3 snapshot and its matching .sqlite3.json manifest together. Copy those completed files to external storage or a folder already synced by your own software. Confirm the external copy/upload yourself. Backups are not encrypted; protect them appropriately.

The live database remains in Windows LocalAppData under com.twx.payroll/data/twx-payroll.sqlite3. Settings shows the actual path. Do not move it into Documents or sync its live files. Never blindly copy an active WAL database: use Backup Now.

Automatic backup runs only while TWX is open and a destination is configured. It makes a baseline, then protects changed business data when the last successful copy is at least 24 hours old. Check Settings for failure notices; run Backup Now after important work. Ten automatic generations are retained in the current destination. Manual and pre-restore copies are retained for deliberate management.

## Restore or undo

1. Open a compatible TWX version and select Restore Backup in Settings.
2. Choose the .sqlite3 file with its manifest alongside. Review the selected snapshot and replacement warning.
3. Confirm deliberately. Current records and appearance will be replaced; TWX first keeps a PRE RESTORE safety snapshot. It preserves the current backup folder and automatic setting.
4. Review records, history and totals. Make a fresh backup after verifying recovery.
5. To undo, restore the PRE RESTORE snapshot whose location appears in history.

Do not interrupt restore. If interrupted, reopen the app; startup uses its preserved recovery marker and safety copy. Do not delete recovery markers, safety snapshots, WAL or SHM files to make an error disappear.

## Move to another PC

Transfer a successful backup/manifest pair, compatible installer, hashes and guides. Install under the replacement user's Windows account, restore through Settings, review historical records, choose that PC's backup/export folders and make a new backup. Retain the original until verified.

SQLite backups include business records, historical snapshots, audit history and theme. Export-folder preference, print PDFs, operation journal and Google/Gmail credentials are machine-local, outside these backups. DPAPI credentials do not move with the database: reconnect optional accounts using your own Google configuration. Check Sheets/Sent Mail before any repeated output because duplicate-operation knowledge does not migrate with a business backup.

## When storage cannot open

Close every TWX instance. Preserve the entire com.twx.payroll folder, including recovery metadata and all sidecars. Following the detailed offline RECOVERY_GUIDE.md, rename the whole closed folder to a distinct preserved name, then reopen a compatible app and use guided restore from a known-good snapshot. Never selectively combine files from different copies. Keep the preserved workspace and original backup until recovery is independently checked. If no backup validates, preserve everything and seek qualified recovery help.

Reports (Excel, PDF, CSV and Word) are derived copies. They are useful for reading records without TWX but cannot restore the database. Keep both readable exports and validated backups. A future-schema backup requires a compatible newer app; never force a downgrade by deleting records.

The release candidate still needs an independent installed-app recovery drill; see RELEASE_CHECKLIST.md for actual evidence rather than assuming this procedure has been certified on another machine.

Uninstall preservation: the corrected 0.2.0 installer has no app-data deletion option or app-data deletion code. It retains the complete workspace. Older installers may still show Delete app data; leave that unchecked. Runtime uninstall/reinstall validation remains pending. Keep a validated external backup before any acceptance drill.
