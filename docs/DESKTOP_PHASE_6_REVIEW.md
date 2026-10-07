# Desktop Phase 6 review

October 6, 2026. Desktop Phases 1–5 are accepted. Phase 6 optional Google Sheets and Gmail report outputs are implemented for review. Automated/fake integration tests, native unconfigured UI, interactive Windows DPAPI and Windows production build/startup passed. **Live Google authorization, Sheets creation and email delivery were not run** because dedicated test credentials were not supplied/configured. No routing, installer/signing, distribution, deployment, cloud synchronization or next phase was started.

## Implemented architecture and behavior

Rust integrations/{oauth,transport,vault,output} provides optional output adapters around the existing Phase 5 canonical report model. Saved SQLite inputs and finalized historical snapshots remain authoritative; no calculation, rate/rounding, old label or financial schema change was made. Preparation reads a coherent saved report under the storage guard, releases SQLite, and generates a held snapshot/attachment. A ten-minute one-use confirmation displays the report/period/state/account and email destination/subject/attachment before any provider request. Cancel submits nothing. A later database edit does not change that prepared revision.

Google Sheets supports Settlement Detail, Driver Payroll, Owner Payroll, Payroll Summary and Settlement History. Confirm always creates a **new spreadsheet**, with numbered section tabs, title/period/context, draft/finalized marking, saved load/payout/adjustment/payroll values, wrapped literal strings and frozen headers. No formulas, financial floating-point conversion, existing spreadsheet overwrite, arbitrary file upload or live synchronization. Owner sections contain no ACH. Successful creation returns only a fixed Google spreadsheet URL/destination ID.

Email uses a customer-owned **Gmail API** connection rather than a permanently hosted backend or developer-owned account. Default-mail-client options were evaluated: mailto attachment preparation is not a reliable common denominator; Simple MAPI requires a compatible installed/configured provider/profile. Gmail shares the installed OAuth infrastructure with Sheets, supports HTTPS MIME attachments and needs only send permission plus identity scopes, without a provider SDK. Other providers are not silently inferred. INTEGRATIONS_GUIDE.md documents this choice and primary references.

Email Report generates a new PDF (default), XLSX, DOCX or CSV attachment from the held saved dataset. One syntactically validated plain ASCII mailbox, subject and optional Unicode message are supported; deliverability is not asserted. Rust rejects MIME header injection, folds UTF-8 encoded subject words, base64-encodes body/attachments and sanitizes attachment filenames. No SQLite database, arbitrary path/file, automatically sent message, persistent recipient list or background send queue. Gmail acceptance is reported as acceptance, not recipient delivery.

## Authentication, credentials and permissions

Each connection privately reads a **customer-owned Google OAuth Desktop client JSON** selected by the main-window-owned native picker. No developer project/client is embedded. Endpoint URLs inside the selected file are ignored. Browser authorization uses code + PKCE S256 and unpredictable state, then a short-lived listener on 127.0.0.1 with an ephemeral port and fixed callback path. It closes on success/error, cancellation or three-minute timeout. All off-machine requests use HTTPS; Google's installed-app HTTP loopback redirect is the documented local exception. There is no permanent or hosted web server.

Separate service grants request `openid email` to display/verify account identity, plus `spreadsheets` or `gmail.send`. No Drive, Gmail read/list or broad mail scope. Identity is retrieved directly from the fixed HTTPS userinfo endpoint, not from unverified frontend claims. Only Desktop client ID/secret, refresh token and verified email identity persist; access tokens are refreshed and held in Rust memory for the current action. Google scope/project/consent/test-user setup and revocation are documented in INTEGRATIONS_GUIDE.md.

The selected vault uses **current-user Windows DPAPI** with UI forbidden, per-application/service entropy and no machine-wide flag. Ciphertext is staged/synced/atomically replaced in `integration-credentials/{sheets,email}.dpapi` under app-local data; plaintext is never written as a fallback. It is outside SQLite, reports, audit JSON and ordinary SQLite backups. Same-Windows-account access remains the security boundary. New-machine/account recovery reconnects integrations.

Connect/reconnect saves only after successful scope/verified-identity exchange. An unsuccessful account change retains the previous connection; success invalidates its prepared confirmations. Expired/revoked/corrupt credentials produce reconnect guidance without financial changes. Disconnect clears the local blob and pending confirmations first, then attempts Google revocation; offline/unconfirmed revocation directs the user to Google Account permissions. Revoking a Google project/account grant can affect both service connections. No token or full OAuth/provider response is logged or returned to React.

Seven named commands were added: get_integration_status, connect_integration, cancel_integration_connection, disconnect_integration, prepare_external_report, confirm_external_report and cancel_external_report. There are **31 named local-main permissions**. Frontend DTOs accept only closed service/format enums, saved identifiers/filters, email composition fields and opaque confirmation tokens. They reject paths/arbitrary attachment bytes/URLs. Remote-origin IPC, filesystem/SQL/shell/HTTP plugins and broad defaults remain unavailable. Frontend CSP has no Internet connection permission. Rust owns fixed token/revoke/identity/Sheets-create/Gmail-send HTTPS requests with normal Windows certificate verification, no redirects, no automatic HTTP retries, ten-second connect and thirty-second per-request timeouts and bounded responses.

## Offline, retry and persistence decisions

No network request occurs at startup or to use bookkeeping/local reports/recovery. Integration locks are separate from the SQLite access guard. Unconfigured providers report configuration/network limitations; provider errors leave local workflows available. External success/failure never writes financial audit or backup-change data.

Before each non-idempotent create/send, an operation marker is staged, synced and published. `integration-operations.json` stores only operational version, ID, timestamp, service, report kind, exact-action digest, state and returned destination ID. It contains no report content, plaintext account/recipient addresses or credentials. Successful/uncertain exact actions are blocked, including after restart; SENDING after interruption becomes UNCERTAIN. Explicit provider rejection/rate limiting permits a new deliberate preparation, with no automatic retry. Missing success identifiers, network ambiguity/server errors or inability to record a result produce honest uncertainty and preserve duplicate protection.

This is deliberately non-secret OS operational metadata outside SQLite, so restoring an older business snapshot cannot roll back the knowledge of an external side effect. It is not a second business database or a financial audit ledger. Corrupt/unavailable metadata disables external actions only. **No migration** was needed; schema remains 4 and migrations 001–004 are byte-for-byte unchanged. Journal loss/new-machine recovery cannot preserve exact duplicate knowledge; users must inspect remote destinations before resending. Identical successful/uncertain actions currently have no repeat override or automatic journal pruning; review reconciliation/retention before distribution. Size bounds and recovery behavior are in INTEGRATIONS_GUIDE.md.

## UI and materially changed files

Settings adds Optional integrations with separate Google Sheets and Gmail account/status, native client-file Connect, reconnect/account change, cancel authorization waiting and disconnect/clear actions. Reports and existing individual/filtered report ExportActions add Send to Google Sheets and Email Report. Two-step confirmations distinguish create/send from preparation. Recipient/subject/message/format are local React state only; there is no recent-recipient persistence. Light/dark, keyboard labels, responsive widths and existing local export controls remain usable.

Added:

- src-tauri/src/integrations/{mod,oauth,transport,vault,output,tests}.rs
- src-tauri/examples/verify_windows_vault.rs — strict synthetic platform probe, no network or real account
- src/contracts/integrations.ts
- src/features/settings/IntegrationSettings.tsx
- src/features/reports/{ExternalActions.tsx,ExternalActions.test.tsx}
- scripts/verify-integrations-native.mjs — refuses normal database; no real transmissions
- docs/INTEGRATIONS_GUIDE.md and this review
- Generated permission TOML files for the seven commands under src-tauri/permissions/autogenerated

Changed:

- src-tauri/src/{commands,dialogs,lib}.rs; build.rs; capabilities/main.json
- src-tauri/Cargo.toml and Cargo.lock
- reports/mod.rs and reports/tests.rs only expose the existing synthetic fixture to test code; report runtime/domain rules remain unchanged
- src/features/settings/Settings.tsx; reports/ExportActions.tsx; src/styles.css
- src/app/App.test.tsx mocks the new integration boundary for foundation tests
- scripts/verify-foundation.mjs checks the exact expanded permission list
- AGENTS.md, ARCHITECTURE.md, README.md, PRODUCT_SPEC.md, IMPLEMENTATION_PLAN.md and DEPENDENCIES.md

Ignored work/outputs contain temporary diagnostic configs, synthetic fixtures, logs and screenshots. No legacy/web script/source participates in desktop build or verification. package-lock.json remains present and unchanged. Both lockfiles are repository-ready; Git checkpoint status is reported separately after the final repository step.

The final Git metadata write permission request was granted for this turn, but `git add` still failed creating `.git/index.lock` with Permission denied. The local checkpoint could not be created; all source and both lockfiles remain uncommitted. No GitHub push is claimed. This is an outstanding repository/environment issue, not a passed verification.

## Exact dependency choices

reqwest **0.13.5**, default features disabled, blocking/json/form/native-tls: fixed HTTPS/OAuth transport with explicit retry/redirect/timeouts and Windows trust. base64 **0.22.1**: PKCE/MIME/Gmail encoding. windows remains **0.62.2**, adding Win32_Security_Cryptography. Existing sha2 **0.10.9**, uuid **1.26.1**, tempfile **3.27.0**, Tauri **2.12.0**, Rust **1.98.1**, rusqlite **0.40.2**, rusqlite_migration **2.6.0**, React **19.3.0**, TypeScript **6.0.3** and Tailwind **4.3.3** remain pinned. No JavaScript dependency or provider SDK was added. DEPENDENCIES.md and lockfiles provide exact transitive versions.

## Verification commands and results

Commands ran with the existing workspace-local Rust/Cargo environment `work/phase2-env.ps1`. The ignored `work/cargo-relay.mjs` wrapper selects the cached development registry and retains HTTPS certificate validation for acquisition/Cargo checksum verification; Cargo.lock contains standard crates.io sources. After acquisition, native checks used CARGO_NET_OFFLINE=true. The wrapper is not an application dependency. Standard environments run scripts/verify-native.ps1 directly.

| Command / check | Result |
|---|---|
| node work/cargo-relay.mjs check --manifest-path src-tauri/Cargo.toml | Initial rustls/AWS-LC C build failed; switched to Windows native TLS. Follow-up exposed two Win32 optional-argument type errors; corrected. Final locked check below passed. |
| cargo fmt --manifest-path src-tauri/Cargo.toml | Ran after changes; initial run before adding tests.rs could not resolve that module; tests added and all later formatting passed. |
| node work/cargo-relay.mjs test --manifest-path src-tauri/Cargo.toml --locked | Initial fixture Debug/move errors fixed. Then 62 passed / one platform-vault assertion failed under restricted shell; corrected the execution-context verification split described below. Final full suite 63 passed. |
| filtered Windows vault diagnostic tests | Credential Manager failed HRESULT -2147023584 (no logon session). DPAPI shell probe failed HRESULT -2147024894 (file/profile unavailable). These failures are retained, not called successful round trips. |
| node work/cargo-relay.mjs build --manifest-path src-tauri/Cargo.toml --locked --example verify_windows_vault | PASS strict platform-probe executable. |
| Interactive Windows launch of debug/examples/verify_windows_vault.exe | PASS encrypted round trip, ciphertext not plaintext, tamper rejection and removal; result.json passed=true, networkRequests=0. The UI helper reports no targetable window because this probe intentionally has no window; the process completed and its strict result was inspected. |
| node work/cargo-relay.mjs clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings | PASS; repeated in full native script. |
| npm.cmd exec prettier -- --write changed frontend/config/script files | PASS formatting; no lint suppression. |
| npm.cmd run verify:frontend | Initial foundation test saw an extra integration alert from an unmocked boundary; added the dedicated integration mock and tests. Final PASS all steps below. |
| npm.cmd run typecheck | PASS strict TypeScript. |
| npm.cmd run lint | PASS ESLint. |
| npm.cmd test | PASS **38 tests across five files** plus foundation tests. |
| npm.cmd run test:foundation | PASS exact 31 capabilities, local CSP/build boundary and real in-memory settings SQL. Node SQLite experimental warning is tooling-only. |
| npm.cmd run build | PASS strict TS and production Vite build. |
| npm.cmd run test:ui | PASS Edge navigation/layout/theme/keyboard/axe at 1440,1024,800,375 widths. |
| npm.cmd run format:check | PASS final after native-script addition. |
| node work/cargo-relay.mjs --native → scripts/verify-native.ps1 | PASS all locked native checks and production build. |
| cargo fmt --manifest-path src-tauri/Cargo.toml -- --check | PASS native script. |
| cargo check --manifest-path src-tauri/Cargo.toml --locked | PASS native script. |
| cargo test --manifest-path src-tauri/Cargo.toml --locked | PASS **63 tests, zero failures/ignored tests**. |
| cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings | PASS native script, no warning suppression. |
| npm.cmd run tauri -- build --no-bundle -- --locked | PASS Windows optimized executable via native script. |
| node work/cargo-relay.mjs --tauri build --debug --no-bundle --config work/phase6-native-config.json -- --locked | PASS isolated native diagnostic build. |
| node work/phase6-fixtures.mjs | PASS 14 existing-workflow fixture checks in phase6-smoke only, including exact linked driver/owner amounts, theme and denied broad IPC. |
| node scripts/verify-integrations-native.mjs | PASS disconnected services, required canonical report types, PDF/XLSX/DOCX unconfigured refusal, owner no-ACH, DTO/path/plugin denial, native email compose/cancel and light/dark axe/no-overflow at 1440/800/375. Financial/audit/backup fingerprint unchanged. |
| Actual native close/relaunch + node scripts/verify-integrations-native.mjs --restart | PASS stored financial/audit/backup fingerprint matches before restart; repeated UI/capability checks passed. |
| work/phase6-observe-db.py | PASS read-only SQLite integrity/schema/theme/counts and release SHA-256. No live database copied. |
| Get-FileHash release executable / migrations 001–004 | PASS inspection; migration hashes match Phase 5 exactly. |
| git diff --check, lockfile and registry-source inspection | PASS available inspection; originally all source untracked, so this is not claimed as a full tracked diff validation. |
| git add desktop source/manifests/docs and both lockfiles | FAILED: Permission denied creating .git/index.lock, even after scoped Git write permission was granted. git commit could not be run. |
| Ordinary optimized Windows app launch | PASS healthy native startup and existing empty business workspace. |
| Live Google OAuth / new Sheet / real email / recipient delivery | **NOT RUN**, no dedicated configured test account; no real external report or email sent. |
| Tauri dev-server launch, installer/physical DPI/clean-machine tests | NOT RUN in Phase 6; isolated native packaged debug/release apps were launched. Distribution/hardware validation remains gated. |

The Windows unit test explicitly verifies either native protection or the known restricted-host failure **with no plaintext file**. It does not call that unavailable branch a successful vault round trip. The independent strict interactive probe does not accept unavailable storage and passed. This models the real fail-closed product behavior and verifies the success path in the execution context used by the app; no test is ignored or warning denied flag removed.

Twelve new Rust tests cover credentials/config/scopes/state/PKCE, exact historical Sheets cells/draft/owner no-ACH, attachment byte equality for PDF/XLSX/DOCX, syntactic validation/injection, preparation/cancel/expiry/explicit confirmation, successful/uncertain duplicate protection across restart, provider/network/rate limits, revoked refresh/disconnect offline, attachment/journal failure before send, interruption/corrupt journal and OS-vault behavior. Fake transports require explicit queued replies and cannot fall through to the network. Nine new frontend tests cover confirmation, cancellation, PDF default/composition, disconnected/uncertain behavior, disabled actions, connection/status/account changes and disconnect. Existing 51 Rust financial/recovery/report tests also passed, including cross-format report consistency.

## Observed runtime and remaining review

SQLite runtime **3.53.2**, schema **4**. The isolated logical path is `C:\Users\[redacted]\AppData\Local\com.twx.payroll.phase6-smoke\data\twx-payroll.sqlite3`. This packaged host also redirects physical storage under `C:\Users\[redacted]\AppData\Local\Packages\OpenAI.Codex_2p2nqsd0c76g0\LocalCache\Local\com.twx.payroll.phase6-smoke\data\twx-payroll.sqlite3`. It holds one synthetic finalized settlement and two payroll rows; theme dark. The normal com.twx.payroll database still has zero settlements/payroll rows and system theme. Both read-only integrity checks returned ok. Diagnostic app was closed after restart verification; its loopback debugging configuration is absent from ordinary production source/config.

Optimized artifact `src-tauri/target/release/twx-payroll.exe`, SHA-256 `BAB845AA596442FD1CB07A20C3BC21FDE834366B41C3E2343B04FA64467B1F15`, launched successfully. No installer, signing, publication or deployment. Informational npm update notices and Node's SQLite experimental warning remain tooling notices, not suppressed product build warnings.

Before release or additional phases: configure a dedicated customer-owned test project/account and deliberately perform the documented live OAuth/Sheets/Gmail smoke; review new-sheet readability in Google and attachment reception; confirm Gmail provider suitability and Google consent/verification requirements; approve operation-journal retention/reconciliation and repeat-identical-send policy; verify offline new-machine recovery/reconnection and vault behavior outside the packaged development host. Prior Office visual pagination, print cache retention, physical printer/DPI, WebView2 provisioning, clean-machine upgrade/uninstall preservation, installer/signing ownership and provisional financial business sign-off remain review items. Routing needs its own authorization.

**STOP for review. No Phase 7/8, routing, installer/signing, distribution or deployment has begun.**

