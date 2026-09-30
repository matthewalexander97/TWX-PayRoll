# TWX Payroll — Phase 1 review

> Historical review of the superseded web scaffold. These results do not verify or complete desktop Phase 1. The current local Tauri/React/SQLite architecture is documentation-only and awaits review in ARCHITECTURE.md and IMPLEMENTATION_PLAN.md.

Phase 1 implements the application foundation only. Phase 2 is not started or authorized. No financial/business migrations, business records, financial calculations, exports, integrations, email delivery, or deployment were created.

## Created

- Next.js App Router application with strict TypeScript, Tailwind/PostCSS, design tokens, reusable small UI components, and exact dependency lockfile.
- Sign-in through Supabase email/password Auth; server actions, cookie-backed SSR clients, proxy refresh, verified-user checks on every workspace page, safe login destinations, logout, recoverable auth errors, and missing-configuration guidance.
- Responsive authenticated navigation and separate driver/owner payroll empty sections; Weekly Load / Settlement Detail and directory empty pages; loading/error/not-found states.
- System-aware light/dark theme initialized before hydration, manual toggle, remembered local preference, and usable controls when localStorage is unavailable.
- Unit/browser/accessibility checks, a test-only Auth HTTP fixture, environment template, setup instructions, dependency record, and updated planning documents.

## Architectural choices

Authentication uses real Supabase SSR client calls; test behavior is supplied by an isolated local HTTP fixture, never an application login bypass. Server-side getUser verifies the current account beyond proxy token refresh and rejects anonymous users. No service-role key is used. Business membership/RLS authorization is intentionally deferred: authenticated users currently see only an empty shell, not financial data.

Server Components render private pages dynamically. Server Actions validate credentials/destinations and provide error/pending states. Private routes are marked private/no-store. Theme preference is the only application state stored locally; theme has no financial effect. The interface uses no fake totals or historical records.

The build requires no remote fonts/images. Next.js 16 uses proxy.ts. Supported TypeScript 6 and ESLint 10 with compatible individual lint plugins avoid the incompatible TypeScript 7/bundled Next lint dependency combination. Clean-install verification caught and fixed an incomplete optional-dependency lock entry. The browser harness uses explicit IPC teardown to avoid restricted Windows taskkill cleanup.

## Recorded versions

Next.js 16.3.7; React/React DOM 19.3.0; Tailwind/PostCSS plugin 4.3.3; PostCSS 8.5.28; TypeScript 6.0.3; ESLint 10.11.0; Supabase JS 2.117.2; Supabase SSR 0.12.7; Playwright 1.63.0; axe Playwright 4.13.0. Full direct versions are in DEPENDENCIES.md and package.json; transitive versions/integrity hashes are in package-lock.json.

## Validation

The final combined command is npm run check, which runs typecheck, lint, unit tests, production build, and browser tests in sequence. All must exit successfully before completion is reported.

| Check | Result |
|---|---|
| npm ci | Passed with regenerated lockfile |
| npm run typecheck | Passed |
| npm run lint | Passed with zero warnings |
| npm run test:unit | 4 passed |
| npm run build | Passed; private pages render dynamically |
| npm test | 13 browser tests passed against the production build |
| Automated accessibility | No tested WCAG AA violations in light/dark modes; login and 375/768/1280px workspace checks |
| Client asset scan | No fixture passwords, test signing secrets, or server-secret markers in emitted client JavaScript |
| npm audit --omit=dev | Zero reported vulnerabilities at validation time |

Browser coverage includes login/logout/session continuity, unauthorized routes, invalid credentials/provider errors, forged/revoked sessions, external redirect rejection, missing project configuration, keyboard skip navigation, responsive overflow, touch targets, theme persistence/OS preference/storage failure, and light/dark accessibility. Desktop/mobile screenshots were visually inspected. Automated accessibility is scoped to tested views/states and does not replace future full manual accessibility review.

No live Supabase project values or test account were supplied. The real project login check remains a setup follow-up; the isolated fixture verifies the application Auth contract, not an external project's configuration. Copy .env.example to .env.local, supply the development URL/public key, provision a test account, and check login/logout before using that project. No private credentials were requested or saved.

## Files added or materially changed

Foundation/configuration: package.json, package-lock.json, tsconfig.json, next-env.d.ts, next.config.ts, postcss.config.mjs, eslint.config.mjs, playwright.config.ts, .gitignore, .env.example, README.md.

Application: src/app/layout.tsx, globals.css, page.tsx, not-found.tsx; src/app/(auth)/login/page.tsx and actions.ts; src/app/(app)/layout.tsx, payroll/page.tsx, settlements/page.tsx, directory/page.tsx, loading.tsx, error.tsx.

Boundaries/components: src/server/auth.ts; src/lib/auth/input.ts; src/lib/supabase/config.ts, server.ts, proxy.ts; src/proxy.ts; src/lib/theme.ts; src/components/theme-toggle.tsx, login-form.tsx, sign-out-button.tsx, navigation.tsx, empty-state.tsx, page-heading.tsx.

Verification: tests/unit/auth.test.ts; tests/e2e/foundation.spec.ts; tests/support/auth-server.ts, app-server.ts, global-setup.ts.

Planning: AGENTS.md, ARCHITECTURE.md, docs/PRODUCT_SPEC.md, docs/IMPLEMENTATION_PLAN.md; new docs/DEPENDENCIES.md and this review. The planning documents now reflect Phase 1-only authorization.

## Review before Phase 2

Confirmed design decisions: separate period_start/period_end DATE fields with ISO exchange/display-only range strings; finalized revision snapshots preserve TWX code and labels/explanations plus driver/owner/truck/plant/job statement labels; PostgreSQL payroll payee CHECK enforces exactly one driver or owner; no generic payee hierarchy.

Unresolved: actual business terminology and meaning of the rate field, ACH treatment, percentage base/sign/order, weight/ton units and authority, final load/settlement formulas, payroll-settlement linkage, driver/owner/truck assignments, calendar rules, per-calculation precision and rounding points, correction/payment behavior, identifier uniqueness, TWX semantics/required fields, and permissions. Consult the question matrix and rule-approval records in PRODUCT_SPEC.md.

Stop here. Phase 2 requires explicit instructions and resolution of the business/schema decisions that affect it; Phase 1 completion does not authorize subsequent phases.
