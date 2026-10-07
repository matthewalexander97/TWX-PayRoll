# Desktop dependency versions

Exact direct versions are pinned in package.json and src-tauri/Cargo.toml; complete resolved graphs are in package-lock.json and src-tauri/Cargo.lock. This replaces the superseded web dependency document (archived under legacy/web).

| JavaScript dependency | Version |
|---|---|
| @tauri-apps/api | 2.12.0 |
| react | 19.3.0 |
| react-dom | 19.3.0 |
| @axe-core/playwright | 4.13.0 |
| @eslint/js | 10.0.1 |
| @playwright/test | 1.63.0 |
| @tailwindcss/vite | 4.3.3 |
| @tauri-apps/cli | 2.12.0 |
| @testing-library/react | 16.3.3 |
| @testing-library/user-event | 14.6.7 |
| @types/node | 24.19.0 |
| @types/react | 19.3.0 |
| @types/react-dom | 19.3.0 |
| @vitejs/plugin-react | 6.1.1 |
| eslint | 10.11.0 |
| eslint-plugin-react-hooks | 7.1.1 |
| jsdom | 29.1.1 |
| prettier | 3.9.9 |
| tailwindcss | 4.3.3 |
| typescript | 6.0.3 |
| typescript-eslint | 8.71.0 |
| vite | 8.3.1 |
| vitest | 5.0.2 |

| Native dependency/toolchain | Version |
|---|---|
| Rust / Cargo | 1.98.1 |
| Tauri | 2.12.0 |
| tauri-build | 2.7.0 |
| tauri-plugin-single-instance | 2.5.1 |
| rusqlite | 0.40.2 |
| rusqlite_migration | 2.6.0 |
| serde | 1.0.229 |
| tempfile | 3.27.0 |
| serde_json | 1.0.151 |
| sha2 | 0.10.9 |
| uuid | 1.26.1 |
| windows | 0.62.2 |
| Bundled SQLite source | 3.53.2 (libsqlite3-sys 0.38.2) |

Verified frontend environment: Node 24.13.0, npm 11.6.2, Edge/WebView2 154.0.4258.37. jsdom 29.1.1 supports this Node version; latest jsdom 30.1.1 requires a newer Node 24 patch and was replaced. Native runtime SQLite 3.53.2 has been verified in the running desktop application.

npm install reported zero vulnerabilities. Lockfiles are generated; Git index write restrictions prevent committing them in this environment.

## Phase 2 dependency change

serde_json 1.0.151 is now an exact-pinned direct Rust dependency for historical snapshots/audit payloads. This same version was already in Cargo.lock transitively; no new arithmetic dependency or dependency upgrades were introduced. Financial integers serialize as JSON strings. package-lock.json is unchanged. The current native checks are recorded in DESKTOP_PHASE_2_REVIEW.md.

## Phase 3 dependency changes

Enable rusqlite's backup feature. Promote tempfile to runtime use for private staging and no-clobber publication; add direct sha2, uuid (v4) and Windows-only windows dependencies at the exact versions above, already present in the lockfile graph. Windows COM supplies native file/folder selection without a frontend filesystem or dialog plugin. No JavaScript dependency changed. Cargo.lock records the direct dependency/features changes; package-lock.json remains present and unchanged. No dependency upgrades or network downloads were required. Phase 3 verification is recorded in DESKTOP_PHASE_3_REVIEW.md.

## Phase 4 dependencies

No direct dependencies, features or resolved versions changed. Existing UUID support supplies Rust-generated record IDs; existing Playwright/axe tooling verifies the native WebView. Both generated lockfiles are present and repository-ready, but remain uncommitted with the rest of this checkout. Runtime SQLite remains 3.53.2. Phase 4 verification is recorded in DESKTOP_PHASE_4_REVIEW.md.

## Phase 5 exact dependency additions

- csv = 1.4.0: deterministic CSV quoting/UTF-8/CRLF. https://docs.rs/csv/1.4.0/
- rust_xlsxwriter = 0.99.1: real offline XLSX packages, cell types/formats/sheets. https://docs.rs/rust_xlsxwriter/0.99.1/
- docx-rs = 0.4.22, default image features disabled: real local Word OOXML without an image framework. https://docs.rs/docx-rs/0.4.22/
- pdf-writer = 0.15.0: compact PDF object/content writer for controlled report layouts. https://docs.rs/pdf-writer/0.15.0/
- ttf-parser = 0.25.1: bundled font glyph/advance/Unicode mapping, no font service. https://docs.rs/ttf-parser/0.25.1/
- zip = 8.6.0 dev-only (deflate): inspect generated DOCX/XLSX packages in tests; same resolved version already used by the export libraries.
- windows stays 0.62.2; enable Win32_UI_WindowsAndMessaging for the private generated-PDF viewer adapter. No shell plugin is granted to React.

Cargo.lock records these additions while preserving existing resolved versions. JavaScript dependencies and package-lock.json remain unchanged. All report generation works offline after dependency acquisition. No report generation framework, office installation or online service is required in the application.

Embedded NotoSans-Regular.ttf is redistributed unchanged under SIL Open Font License 1.1, with OFL-NotoSans.txt adjacent. Source: https://github.com/notofonts/noto-fonts/blob/main/hinted/ttf/NotoSans/NotoSans-Regular.ttf . SHA-256 B85C38ECEA8A7CFB39C24E395A4007474FA5A4FC864F6EE33309EB4948D232D5. Font license source: https://github.com/google/fonts/blob/main/ofl/notosans/OFL.txt . PDF font embedding does not require a local installed font or network.

Read-only verification uses the bundled Python runtime with pypdf 6.13.2, openpyxl 3.1.5, python-docx 1.2.0 and bundled Poppler. These QA helpers are not shipped, added to application manifests or required by users. Word/LibreOffice rendering availability is reported separately.

## Phase 6 exact dependency additions

- reqwest = 0.13.5, default features disabled, blocking/json/form/native-tls: fixed-endpoint HTTPS and OAuth form transport, bounded timeouts, disabled redirects/retries. Windows native TLS uses the OS trust store; certificate verification stays enabled. No provider SDK or permanently hosted backend. [Documentation](https://docs.rs/reqwest/0.13.5/reqwest/).
- base64 = 0.22.1: PKCE S256 challenge, MIME UTF-8 header/body/attachment encoding and Gmail base64url payload. Previously resolved transitively; now exact-pinned as a direct dependency. [Documentation](https://docs.rs/base64/0.22.1/base64/).
- windows remains 0.62.2; add Win32_Security_Cryptography for current-user DPAPI. No frontend security/filesystem/network plugin. sha2 0.10.9, UUID 1.26.1 and tempfile 3.27.0 are reused for PKCE, operation IDs and staged encrypted/operational files. OAuth is an explicit installed-app flow with state/PKCE validation, not a provider framework.

Cargo.lock was regenerated through the validated development registry transport and uses standard crates.io sources. Existing pinned direct versions remain unchanged. Native TLS replaced an initial rustls/AWS-LC build attempt that failed on this environment's C compiler configuration; no compiler warning was suppressed. package-lock.json and all JavaScript dependency versions remain unchanged. App generation stays offline; only explicitly invoked provider outputs use network.

## Phase 7 packaging and dependency record

App version is 0.2.0; no direct application dependency versions changed. Tauri/API/CLI 2.12.0, tauri-build 2.7.0, React 19.3.0, TypeScript 6.0.3, Tailwind 4.3.3, Vite 8.3.1, rusqlite 0.40.2, rusqlite_migration 2.6.0 and SQLite 3.53.2 remain pinned. Node 24.13.0/npm 11.6.2 and Rust 1.98.1 were used. Both lockfiles are repository-ready and uncommitted.

Installer tooling: NSIS 3.11 and nsis-tauri-utils 0.5.3, acquired/verified by Tauri. Project-local installer tools avoid this host's inaccessible shared tools cache. Microsoft Evergreen offline WebView2 installer is embedded; its downloaded Authenticode signature was valid. The application/setup remain unsigned. No new runtime JS plugin, generic filesystem/shell/SQL plugin, or remote-page permission was added.

THIRD_PARTY_NOTICES.txt includes 284 resolved Windows runtime crates, production JavaScript notices, font and installer notices. Supplemental license files come from the exact crate source commit/tag. Five unmodified MPL corresponding-source archives are in third-party/source with hashes. The release allowlist includes these source obligations without copying development caches. See DESKTOP_PHASE_7_REVIEW.md for tests and outstanding acceptance gates.
