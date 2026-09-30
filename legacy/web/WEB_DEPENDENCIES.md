# Phase 1 dependency record

> Historical web-scaffold versions, not approved desktop dependency selections. Desktop versions will be selected when implementation is authorized. No dependencies are installed or changed by the current architecture revision.

Registry selections were checked during Phase 1. Exact versions are declared in package.json and resolved with integrity hashes in package-lock.json. The application uses stable releases, with TypeScript selected for current lint compatibility rather than assuming every newest major can be combined.

| Dependency | Version |
|---|---|
| Next.js / Next lint plugin | 16.3.7 |
| React / React DOM | 19.3.0 |
| Supabase JS | 2.117.2 |
| Supabase SSR | 0.12.7 |
| server-only | 0.0.1 |
| Tailwind CSS / Tailwind PostCSS | 4.3.3 |
| PostCSS | 8.5.28 |
| TypeScript | 6.0.3 |
| ESLint | 10.11.0 |
| @eslint/js | 10.0.1 |
| typescript-eslint | 8.71.0 |
| React Hooks lint plugin | 7.1.1 |
| Playwright Test | 1.63.0 |
| axe-core Playwright | 4.13.0 |
| Node types | 24.19.0 |
| React / React DOM types | 19.3.0 |

Runtime used: Node.js 24.13.0, npm 11.6.2. Project minimum Node 22.18.0 supports native TypeScript stripping for test helpers. No global package installations were performed.

The first attempt at TypeScript 7/ESLint 10 with eslint-config-next failed compatibility checks. The resolved tooling uses TypeScript 6 and ESLint 10 with compatible individual plugins. ESLint 9 was not retained because the registry marks it unsupported. This choice preserves supported tooling and strict lint checks.

Official implementation references: [Next.js 16 upgrade/runtime guidance](https://nextjs.org/docs/app/guides/upgrading/version-16), [Next.js Proxy convention](https://nextjs.org/docs/app/getting-started/proxy), [Supabase SSR](https://supabase.com/docs/guides/auth/server-side/creating-a-client?framework=nextjs), [Tailwind Next.js setup](https://tailwindcss.com/docs/installation/framework-guides/nextjs). Package-lock.json, not the documentation, is authoritative for exact transitive versions.
