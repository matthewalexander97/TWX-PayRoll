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
| tempfile (tests) | 3.27.0 |
| Bundled SQLite source | 3.53.2 (libsqlite3-sys 0.38.2) |

Verified frontend environment: Node 24.13.0, npm 11.6.2, Edge/WebView2 154.0.4258.37. jsdom 29.1.1 supports this Node version; latest jsdom 30.1.1 requires a newer Node 24 patch and was replaced. Native runtime SQLite 3.53.2 has been verified in the running desktop application.

npm install reported zero vulnerabilities. Lockfiles are generated; Git index write restrictions prevent committing them in this environment.
