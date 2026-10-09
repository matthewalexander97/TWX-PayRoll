# Settings and navigation UI review — 2026-10-08

## Changes

- Reference Records is a dedicated navigation destination immediately below Reports. It contains the existing TWX Codes, Plants, Jobs and Adjustment Types editors.
- Settings ends with Optional integrations. Backup/recovery and exports use consistent panel padding, readable text spacing and grouped actions.
- Automatic backup timing and retention are presented as separate bullets. Restore confirmation and safeguards remain intact.
- Google Sheets and Gmail have separate responsive cards; Google setup instructions are expandable. Connection status, errors and disconnect guidance remain visible.
- Shared typography, card treatment and short-window sidebar scrolling are refined for light/dark layouts.
- README and the shared offline/in-app user guide describe the new Reference Records location.
- Financial rules, migrations, Rust commands and capabilities are unchanged. No dependencies were added.

## Verification

- `npm run verify:frontend`: typecheck, lint, 45 tests, foundation guard, manual alignment, installer source guard and production frontend build passed. The aggregate command stopped at `test:ui` because Edge could not launch in the current Windows sandbox (named-pipe/process error).
- `npm run format:check`: passed separately.
- Initial Vitest temporary-directory failures were resolved using `work/ui-temp` for TEMP/TMP. An invalid Testing Library option in the new test was corrected; all 45 tests then passed.
- `scripts/verify-native.ps1`, through the existing offline toolchain relay: Cargo check and Clippy with warnings denied passed. Cargo fmt and Tauri production build failed to canonicalize paths with Windows Access denied. Cargo tests: 51 passed, 17 failed at filesystem/path access. A retry after a scoped directory permission grant produced the same blockers.
- Native invocation: `. ./work/phase2-env.ps1`, set `CARGO_NET_OFFLINE=true`, set TEMP/TMP to the workspace's `work/ui-temp`, then `node work/cargo-relay.mjs --native`.
- Detailed latest native output: `work/ui-native-check.log` (local diagnostic, not a release artifact).
- Browser visual/accessibility verification and native runtime smoke verification remain pending. No new desktop executable or installer was produced for this UI update. Existing release outputs have not been refreshed or published.
- No independent Windows PC/VM is available.

## User Manual presentation update — 2026-10-08

The in-app manual now has a numbered table of contents with grouped heading/introduction, desktop sticky positioning, hover/focus treatments and responsive one/two-column links. Section cards have consistent padding, numbered headings, paragraph spacing and a constrained reading width. Existing section anchors and keyboard return links are preserved. The app's "Offline first" badge and its unused CSS are removed; the manual page has a specific introduction. Bundled manual text is unchanged.

Typecheck, lint, all 45 frontend tests (including manual keyboard tests), foundation/manual/installer guards, formatting and production frontend build passed. `npm run verify:frontend` stopped when Edge failed to launch for visual/accessibility checks. The alternate in-app browser preview timed out reaching the local Vite server. The Tauri production build again failed to canonicalize `../dist` with Access denied. Visual/native runtime checks remain pending and no refreshed executable or installer was produced. Rust sources and native configuration were not edited for this presentation update; the prior native results above still apply.

Additional manual checkpoint file: `src/features/manual/UserManual.tsx`.

## Manual checkpoint commands

Git status is blocked here by an ownership mismatch. Do not add a global safe.directory exception or weaken .git permissions. In a trusted checkout owned by your Windows account, review the changes and run:

```powershell
git status --short
git diff --check
git add README.md USER_GUIDE.md src/app/App.tsx src/app/App.test.tsx src/components/Navigation.tsx src/features/settings/Settings.tsx src/features/settings/BackupPanel.tsx src/features/settings/ExportSettings.tsx src/features/settings/IntegrationSettings.tsx src/features/manual/sections.json src/features/manual/UserManual.tsx src/styles.css scripts/verify-ui.mjs docs/SETTINGS_UI_REVIEW.md
git diff --cached
git commit -m "Refine settings layout and separate reference records"
```

Run the remaining browser/native checks from that trusted development environment before rebuilding release artifacts. This review does not mark native or release acceptance complete.
