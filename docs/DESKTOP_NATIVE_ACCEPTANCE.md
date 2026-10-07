# Desktop Phase 1 — native acceptance evidence

> Historical evidence. Phase 1 was accepted October 3, 2026. Current Phase 2 rules and implementation results are in PRODUCT_SPEC.md and DESKTOP_PHASE_2_REVIEW.md.

September 29, 2026. Frontend review is approved. Native compiler/build checks and the runtime smoke tests below now pass. Phase 1 is returned for review; Phase 2 has not begun.

## Prerequisites confirmed usable

Visual Studio Build Tools 2026 (18.10.12224.181), MSVC tools 14.51.36231 / compiler 19.51.36260 for x64, Windows SDK 10.0.26100.0, WebView2 154.0.4258.37, and pinned Rust/Cargo 1.98.1. Tauri detects MSVC successfully. cl.exe runs and kernel32.lib exists in the SDK's x64 libraries. Successful native compilation/linking further verifies usability.

## Commands and results

Cargo commands used the existing workspace-local Rust installation and temporary HTTPS relay/cache configuration to avoid the environment's earlier Schannel problem. Registry checksums and HTTPS validation remained enabled. Neither lockfile contains the temporary localhost registry. No application dependencies or versions changed.

| Command / verification | Result |
|---|---|
| vswhere.exe -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -format json | PASS — complete, launchable Build Tools installation; no reboot required. |
| MSVC cl.exe; SDK kernel32.lib existence check | PASS — x64 compiler runs; SDK library present. |
| npm run tauri -- info | PASS — MSVC, Rust and WebView2 detected. Optional crates.io latest-version metadata timed out; this did not block locked compilation. |
| cargo check --manifest-path src-tauri/Cargo.toml --locked | Initially FAILED E0493 in migration constant; fixed. Final PASS, including the script run and an offline cached repeat. |
| cargo fmt --manifest-path src-tauri/Cargo.toml | PASS — formatted the corrected declaration. |
| cargo fmt --manifest-path src-tauri/Cargo.toml -- --check | PASS — script run and final verification. |
| powershell.exe -NoProfile -File scripts/verify-native.ps1 | FAILED before script execution: legacy Windows PowerShell execution policy. No system policy was changed. |
| pwsh -NoProfile -File scripts/verify-native.ps1 (via node work/cargo-relay.mjs --native) | PASS — entire script with PowerShell 7.6.5, including every Cargo check and production build. |
| cargo test --manifest-path src-tauri/Cargo.toml --locked | PASS — four Rust tests, zero failures: settings/reopen/idempotency, future-schema refusal, migration rollback, settings constraints/path. Binary/doc test targets also completed with zero failures. |
| cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings | PASS — no suppressed warnings. |
| npm run tauri -- build --no-bundle -- --locked | PASS — production frontend build and optimized Windows executable produced. No installer bundling or publishing. |
| cargo build --manifest-path src-tauri/Cargo.toml --locked --offline | PASS — debug build; final rebuild removed temporary diagnostic browser arguments from the debug executable. |
| Tauri dev --no-watch -- --locked | Compiled and ran; initial fresh-data launch in the restricted tool process FAILED with Access denied during setup. Normal interactive Windows launch initialized storage successfully. |
| Tauri dev --no-watch --config work/native-smoke-config.json -- --locked | First retry FAILED because the initial Vite child still held port 1420. Retried with the temporary config reusing that existing verified Vite server: PASS — compiled and ran a native dev window. Interactive desktop launch of that same debug binary displayed the React app and restored the saved theme. Tool-process and interactive desktop contexts are distinct; the normal desktop window was exercised through Computer Use. |
| node work/read-live-settings.cjs | PASS — inspected the actual app-created database read-only; schema 1, only app_settings, saved Dark and later restored System, integrity_check=ok. Node's SQLite ExperimentalWarning is test-tooling only; its own SQLite version was not used as the app runtime version. |
| node work/probe-native.mjs | PASS — real Tauri IPC allowed reads, invalid-theme rejection, ungranted core command denial, absent fs/shell/sql plugin denial, and unapproved-origin denial. Initial test-server cleanup waited on a retained WebView HTTP connection; fixed the harness to close its own connections, then passed. |
| node work/native-metrics.mjs | PASS — actual native WebView metrics/theme/application information, including restored 1280x850 and maximized 2157x1191 CSS viewports with no horizontal overflow. |
| node work/native-scale-smoke.mjs | PASS — actual Windows Tauri WebView at CDP-emulated 125%, 150%, 200% scale and 1280x850, 1024x768, 800x600, compact approximately 375x700 viewport. No horizontal overflow or application alerts. |
| Native second-instance launch | PASS — original PID 27756 and window ID 7604470 remained; one process/window after second launch. Minimized first window restored; no second independent workspace. |
| Normal production executable launch and restart through Windows session | PASS — local bundled UI, usable settings, schema/runtime information. Restart after restoring original System preference read System from SQLite. Vite was stopped and unreachable; production app still opened at tauri.localhost. |
| Debug port cleanup | PASS — port 9223 no longer reachable after closing the diagnostic dev app. Production config has no diagnostic browser arguments. |
| npm ls --depth=0 | PASS — root dependencies match pinned desktop manifest; no Next.js/Supabase dependency participation. |
| node scripts/verify-foundation.mjs | PASS — build/capability/identifier isolation and settings migration constraints. |
| npm run format:check | PASS — includes new reusable desktop IPC/scale verification scripts. |
| git add package-lock.json src-tauri/Cargo.lock | FAILED — .git/index.lock permission denied. Both lockfiles are present and consistent with successful locked native build and npm dependency inspection, but remain uncommitted. |

The temporary read-only Win32 metrics helper compiled successfully but could not enumerate the interactive user's window from the tool desktop; it is not used as DPI evidence. An attempted mouse resize selected page text rather than moving the native border; no app code was changed for that tooling limitation. Native maximize/restore and independently measured WebView viewport/scale checks provide the recorded resize evidence. Repeated full native compilation after passing checks was limited to the ordinary debug rebuild needed to remove diagnostic configuration.

## Actual database and persistence

Observed live path, with username redacted:

```
C:\Users\[redacted]\AppData\Local\com.twx.payroll\data\twx-payroll.sqlite3
```

The real application created this file beneath per-user Windows LocalAppData, not Documents, the workspace, or installation directory. Rust IPC and the native Settings view both reported **SQLite 3.53.2**, application 0.1.0 and migration/schema version 1. Read-only inspection confirmed the sole table is app_settings and integrity_check returned ok.

Saved Dark through the production native UI, observed the committed SQLite row, closed the app, launched the development native build, and read Dark through real IPC and the rendered dark UI. The invalid-theme test did not change it. Restored the initial System preference through the narrow command; a fresh standalone production restart read System and displayed it in Settings. No browser preview persistence was substituted.

## IPC and security results

Generated compiled capabilities contain only allow-get-app-info, allow-get-preferences, allow-set-theme-preference for local window main. The generated application ACL has no default permission grant. Native calls to window.set_title, window.close and path.resolve returned not allowed. fs.read_text_file, shell.execute and sql.select returned not allowed / Plugin not found. Theme=invalid was rejected by Rust enum deserialization.

Navigated the actual debug WebView to a separate test-only localhost origin (port 4174) and invoked get_preferences. Tauri rejected it with allowed-on local-window/capability information. Returned to the authorized dev origin successfully. Thus even a localhost page outside the authorized local app origin cannot use these commands. No broad filesystem, shell, SQL, window manipulation or remote-origin permissions were added for tests. The remote-debug port was confined to a temporary dev config under ignored work/, closed after testing, and is absent from the product config and final debug rebuild.

## Window/scaling limits

Native restored and maximized window layouts were inspected. The unmodified WebView reported devicePixelRatio 1.1875 in this Windows session. Additional 125/150/200% checks used CDP device-scale/viewport emulation in the actual Tauri WebView; these are clearly **emulated DPI rendering checks**, not claims that Windows global display scaling was changed. Physical multi-monitor/per-monitor-DPI transitions were not tested. OS display settings were left intact. That broader hardware matrix remains a release validation task, and the distinction is available for Phase 1 review.

## Code changes and repository readiness

- src-tauri/src/storage.rs: replaced the invalid temporary-array const initializer with static MIGRATION_STEPS and static MIGRATIONS. Same migration SQL/table/version; no metadata table or business schema added.
- scripts/verify-native.ps1: production build now explicitly uses --locked.
- scripts/verify-desktop-ipc.mjs and scripts/verify-desktop-scale.mjs: reusable versions of the successful native diagnostic probes. Require a deliberately enabled localhost CDP dev session, and TWX_EXPECTED_THEME (default dark). No production commands or permissions added.
- README/architecture/requirements/plan/dependency status updated; earlier blocked report retained with a superseded-status banner.

package-lock.json and src-tauri/Cargo.lock exist, pin the desktop graphs and contain ordinary registry sources. Successful --locked Cargo checks/builds and npm ls confirm consistency. They are repository-ready files, but the environment still prohibits staging/committing them. No commit is claimed. legacy/web is excluded from the root dependency graph, TypeScript/Vite entry and Tailwind source scanning; the successful desktop production build used src and src-tauri only.

Production executable: src-tauri/target/release/twx-payroll.exe, 10,484,224 bytes. It is an unsigned local review artifact; installer signing/bundling/distribution were not performed and do not block Phase 1 development. Source and review binary are copied to outputs/desktop-phase-1.

## Remaining review items

No compiler, Rust test, Clippy, formatting, SQLite initialization/persistence, single-instance or IPC denial failure remains. Review the emulated-versus-physical DPI coverage above and stage/commit lockfiles when Git index access is available. No real driver/owner/company/payroll data was entered. Business rules and precision remain unresolved and untouched. No backups, exports, integrations or other Phase 2 features were added.

STOP for review. Phase 2 has not begun.

