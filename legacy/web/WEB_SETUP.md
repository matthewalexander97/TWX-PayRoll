# TWX Payroll

> Superseded web-scaffold documentation. The current product is a local Tauri 2/React/SQLite desktop application awaiting architecture review. Do not run the cloud setup instructions below as the new Phase 1. Read AGENTS.md, ARCHITECTURE.md, docs/PRODUCT_SPEC.md, and docs/IMPLEMENTATION_PLAN.md for the current direction. The scaffold is preserved as historical work; no desktop implementation has started.

Phase 1 application foundation: an authenticated Next.js shell with responsive navigation, accessible empty payroll/settlement/directory pages, and remembered light/dark themes. Financial tables, business persistence, calculations, reports, integrations, and deployment have not been implemented.

## Local setup

Use Node.js 22.18+ (Node 24 LTS recommended) and npm. The verified environment used Node 24.13.0 and npm 11.6.2. Dependency versions are pinned in package.json/package-lock.json; use npm ci for reproducible installation. In PowerShell, npm.cmd avoids execution-policy issues with npm.ps1.

```powershell
npm.cmd ci
Copy-Item .env.example .env.local
# Edit .env.local with your existing development project's public values.
npm.cmd run dev
```

Open http://127.0.0.1:3000. Without valid Supabase configuration, the login page displays setup guidance and private routes redirect to login. There is no preview account, bypass, fake financial data, or browser-only business storage.

## Supabase Auth configuration

Configure an existing development Supabase project and an administratively provisioned test account. Do not enable public signup or anonymous sign-in for this private application. Supply these values locally; never paste a password or service-role secret into tracked files:

| Variable | Purpose |
|---|---|
| NEXT_PUBLIC_SUPABASE_URL | Development project URL, normally https://PROJECT.supabase.co |
| NEXT_PUBLIC_SUPABASE_PUBLISHABLE_KEY | Project publishable key; a legacy anon key is accepted, service-role keys are rejected |

No service-role key, APP_BASE_URL, or integration credentials are needed by the implemented Phase 1 flow. APP_BASE_URL remains a possible future reporting/email configuration. Auth uses administrator-provisioned email/password accounts, server actions, SSR session cookies, proxy refresh, and a server-side getUser check. Passwords never enter application storage or logs. The Auth provider's rate limits remain active; no custom auth database has been created. See [Supabase SSR setup](https://supabase.com/docs/guides/auth/server-side/creating-a-client?framework=nextjs).

Authenticated access currently exposes only the empty application shell. Organization memberships, RBAC business permissions, and RLS require the separately approved database phase. Do not treat Phase 1 login as sufficient authorization for future financial data.

## Checks

```powershell
npm.cmd run typecheck
npm.cmd run lint
npm.cmd run test:unit
npm.cmd run build
npm.cmd test
# Or run the complete sequence:
npm.cmd run check
```

The browser suite runs against the production build with an isolated test-only Supabase Auth HTTP fixture and the real Supabase SSR client. It uses ports 3100, 3101, and 54321 and shuts down its own processes through IPC, without Windows taskkill. The fixture is not a real Supabase project and is never imported by application code. It verifies application behavior against the Auth contract; it does not replace a later live project login check.

Tests use locally installed Microsoft Edge by default. On systems without Edge, install Playwright Chromium and select it:

```powershell
npx.cmd playwright install chromium
$env:PLAYWRIGHT_CHANNEL = 'chromium'
npm.cmd test
```

The suite covers protected direct navigation, login/logout and session continuity, invalid/provider-failure feedback, revoked/forged sessions, safe redirects, missing configuration, system/manual theme behavior, unavailable browser storage, keyboard navigation, 375/768/1280px layouts, and automated WCAG AA checks in both themes. It uses synthetic credentials only. Generated traces/reports stay ignored; screenshots are in outputs/.

## Important boundaries

- Server-only auth functions and Supabase client creation stay outside client UI modules. Proxy refresh does not replace each page's verified-user check. Private pages are dynamically rendered and responses are marked private/no-store.
- Theme is a presentation preference in localStorage, initialized before hydration. It follows the operating system until a manual preference is saved. Business data does not use localStorage.
- Next.js App Router uses its version 16 proxy convention. Tailwind 4 is installed through its PostCSS plugin; custom design tokens and responsive CSS keep the shell small. No external fonts or image services are required for builds.
- ESLint 10 uses supported Next.js, TypeScript, and React Hooks plugins directly. The bundled eslint-config-next dependency tree still declares older React/import plugin peers; TypeScript 7 is not supported by the installed typescript-eslint parser. TypeScript 6.0.3 and ESLint 10.11.0 avoid those incompatible combinations without disabling lint rules.
- Dates will be separate DATE/ISO period_start and period_end values; displayed week strings will not be stored. Finalized revisions will snapshot statement labels/TWX codes. Payroll will enforce driver XOR owner in PostgreSQL. These are planning decisions, not Phase 1 migrations.

Read AGENTS.md, ARCHITECTURE.md, docs/PRODUCT_SPEC.md, docs/IMPLEMENTATION_PLAN.md, and docs/PHASE_1_REVIEW.md before the next phase. Phase 2 requires explicit authorization and resolution of the relevant business/schema decisions, including actual rate terminology and financial rounding.
