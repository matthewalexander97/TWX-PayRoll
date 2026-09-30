# TWX Payroll — desktop product specification

Status: Desktop Phase 1 foundation is authorized and implemented for review. Native compiler/build checks and runtime smoke tests have passed; see docs/DESKTOP_NATIVE_ACCEPTANCE.md. No later phase is authorized. The earlier web scaffold is isolated under legacy/web.

## Purpose and boundaries

One bookkeeper uses a normal installed Windows application to replace an Excel-based trucking workflow. SQLite stores payroll, drivers/owners/trucks, loads, weekly settlements, adjustments, references, and history. Ordinary work, existing records, local backups/recovery, and reports need no hosted backend/domain/subscription or developer account. Windows account access is the initial boundary; no app login is required.

PC-first window resizing matters. Compact components may be reused later, but the initial database does not synchronize to another PC, phone, or tablet. Application PIN/auth/encryption, cloud sync, Sheets, email, and routing are separately scoped options.

## Views and fields

| View | Behavior / components |
|---|---|
| TWX Payroll | Distinct Driver Payroll and Owner Payroll sections; PayrollEntryForm, PayrollTable/Cards, WeekFilter, AmountDisplay; save/edit/filter/prior-week history |
| Weekly settlements | Date/driver/truck/status filters, saved settlement list, new/open draft |
| Weekly Load / Settlement Detail | SettlementHeader, LoadEditor/Table/Cards, AdjustmentEditor, SettlementTotals, ActionBar, History/ConflictNotice |
| Directory | Drivers, owners, trucks, TWX codes, plants/jobs, flexible adjustment labels; create/edit/archive |
| Backup & recovery | Destination picker, manual/automatic backups, generations, last success/failure/overdue status, guided restore |
| Settings/About | Theme, read-only data location, application/schema version, offline recovery help |
| Reports (later) | Record/revision selection; local PDF/CSV/XLSX/DOCX, print, optional external actions |

Payroll fields in both sections: Driver Name or Owner Name, TWX Code, ACH Charge, Truck #, Amount, Week Ending, Notes. Propose payee/truck/amount/week required; code/ACH/notes optional pending review. Database CHECK enforces exactly one driver/owner consistent with kind. NULL ACH is not zero and no net-pay rule is inferred. Multiple same-payee/week entries remain allowed until uniqueness is defined.

Settlement header: inclusive week range, Driver ID, Truck ID. Store period_start/period_end separately as validated ISO TEXT dates, e.g. 2026-02-28 and 2026-03-06; display 02/28/2026 – 03/06/2026, never store that combined display string.

Load fields: Load ID, BOL Number, PO Number, Plant, Job Name, Truck #, Driver Name, Net Weight, Tons, Mileage, rate field, Total. Completion Rate is supplied terminology, not yet verified; inspect paperwork for Completion Rate, Compensation Rate, Commission Rate, or Rate before implementing terminology or math.

Adjustment rows: TWX Code | Explanation | Dollar Amount. Seed %, Insurance, Hopper, Reimbursement, Apportioned Plates as lookup rows. New types need no migration; labels imply neither sign nor formula. Drafts preserve entered amounts; final TOTAL remains pending until approved formula/manual-total policy.

Action area evolves from Save to local Export/Print and separately optional Email. Omit/clearly disable unavailable actions; do not simulate success. Provide loading/empty/saved/error/conflict/draft/finalized states. Saved means SQLite transaction committed; disk/lock errors preserve input.

## Resizing and theme

Desktop tables/forms have clear typography/aligned amounts and two distinct payroll sections. Settlement layout: header → loads → adjustments → totals/actions. Narrow windows use labeled cards/stacked forms with every field accessible and no page-wide horizontal scrolling. Review 800×600 minimum, test 1024×768/1440×900 and Windows 125–200% scaling. 375px component testing is reuse work, not phone delivery.

Keyboard/focus/labels/errors, WCAG AA contrast, status text independent of color, and practical 44px touch targets remain required. Default to Windows light/dark; manual preference persists in local nonfinancial settings and overrides system changes. Resolve theme before showing the window where practical; use safe fallback on settings failure. Print/PDF theme is independent.

## History, ownership, and backup requirements

Drafts editable with expected versions; finalized facts/labels/results/rule version remain frozen. Revision snapshots preserve literal TWX code/label, explanation, driver/owner names, truck number, plant/job labels, dates, units, currency/scale, inputs/totals. Catalog changes cannot rewrite statements. Proposed Correct record creates linked draft with reason/original retained; payment effects await approval. Internals stay largely invisible except History.

Ship portable SQLite snapshots, schema/encoding documentation, compatible installer and offline recovery steps. Data recovery cannot require developer accounts. Excel import needs actual workbook review/mapping/duplicate and decimal/date reconciliation; it is not automatically included.

Backup is required before a released app accepts real records: manual backup/full database export, ordinary destination folder selection, restore, automatic in-app schedule, generations, honest last success/failure/overdue display. Proposed cadence/retention for approval: daily when running/data changed; catch-up on next launch; seven daily/four weekly automatic generations; manual/pre-restore retained until deliberate deletion. Closed-app backup requires separately approved OS scheduling.

Folders may be removable or already synced; no provider/API is required, and local completion does not prove remote upload. Keep live data in app-local storage. Validate backups; preserve last good generation; failures/interruptions must not destroy active data. Restore replaces, never merges/synchronizes. Clarify encryption/retention/destination/recovery requirements before release.

## Unresolved rules and decisions

| ID | Question | Safe proposal / gate |
|---|---|---|
| Q1 | Actual rate label/meaning: completion/compensation/commission/rate; per ton/mile/load/percentage? | Provisional rate_decimal/unit; no computation |
| Q2 | % explanation or calculation; base/sign/order/compounding/cap? | Category only |
| Q3 | Exact load Total/settlement TOTAL formulas, signed adjustments, missing-value policy? | Entered values separate; final outputs pending |
| Q4 | ACH deducted/included/added/informational; driver/owner differences? | Store independently |
| Q5 | Driver/owner/truck cardinalities/history; row/header equality; dual driver/owner identity? | Independent reference entities; no generic payee or fixed assignment |
| Q6 | Weight/ton units, conversion/authority/overrides? | Preserve both explicit inputs; no conversion |
| Q7 | Week endings/start/inclusion/timezone, partial/overlapping periods? | Explicit saved start/end dates |
| Q8 | Settlement → payroll relationship, split/aggregated payments? | No automatic linkage |
| Q9 | Currency/minor-unit scale; input/intermediate/output precision; tie/negative rounding and every rounding point? | Exact storage proposal; require per-calculation approval |
| Q10 | Manual totals allowed; required completeness/finalization gates? | Draft only until approved |
| Q11 | Correction replacement/reversal/delta/next-week policy; rule version/report/payment effects? | Preserve originals, no inferred correction accounting |
| Q12 | Identifier case/uniqueness, duplicate week entries, shared loads, revision numbering? | UUIDs internally; no invented global/week uniqueness |
| Q13 | TWX meaning/shared catalog/required fields? | Reference IDs plus finalized code/label snapshots |
| Q14 | Is OS-only protection sufficient; future PIN/encryption/multiple local users? | No app Auth/user/role tables initially |
| Q15 | Report samples/volume/selection, customer email/Google/routing accounts, credential/attachment constraints? | Local report boundary; optional integrations |
| Q16 | Backup destination/cadence/retention/encryption, closed-app scheduling, recovery expectations? | Proposed in-app schedule; explicit review needed |
| Q17 | Windows/CPU/min window/DPI, WebView2 offline provisioning, signing/installer/update ownership? | Review desktop prerequisites/distribution |
| Q18 | Which existing Excel records require import and reconciliation? | Review actual workbook/mapping first |

Schema decisions Q5–Q7/Q9/Q12/Q13 gate relevant business migrations. Relevant Q1–Q11/Q13 answers gate authoritative calculations. Q16 governs backups; Q17 desktop targets; Q18 import. Planning completion never authorizes implementation.

## Per-calculation approval records — pending

Do not assume rounding behavior. Define input/intermediate/output precision and every rounding point before implementing an approved calculation. Storage capacity, display formatting, and rounding policy are different.

| Calculation | Required details |
|---|---|
| Weight → Tons (if used) | Units/factor, authoritative input/override, precision and conversion rounding |
| Rate → Load Total (if used) | Actual meaning/base/formula, precision, per-load versus retained intermediates |
| Percentage adjustment (if used) | Base/sign/order, independent/compound, precision and rounding step |
| ACH / payroll payable | Amount meaning, driver/owner treatment, scale, intermediate/final rounding |
| Final settlement TOTAL | Included items/sign/order/missing-input policy, sum of rounded lines versus rounded sum |
| Correction financial effect | Replacement/reversal/delta, rule version, rounding and report inclusion |

Each approval includes ID/version, exact formula, units/signs, precision/mode/rounding points, effective date, approver, and worked examples covering ties/zero/negatives where allowed plus divergent line-versus-total rounding.

## Acceptance criteria

- Close/reopen offline and retrieve identical saved records/history from SQLite; no browser-only business storage.
- Exact financial string/integer round-trip, checked overflow/precision, NULL distinct from zero, XOR payee CHECK, approved calculation examples.
- Transactions/version conflicts never partly save/finalize; catalog/rule changes cannot alter old statements.
- Restore a full validated backup on a clean offline installation without developer accounts; invalid/interrupted restore preserves original/rollback; status is honest.
- Restricted IPC/path/remote access is denied; approved dialogs work without broad filesystem rights.
- Windows resizing/DPI/keyboard/light/dark workflows work; input survives recoverable disk/lock errors.
- Local reports/print reproduce saved exact values offline; optional API failures preserve manual operation.

Tax filing, ACH payment execution, accounting ledger posting, routing optimization, and cloud sync are not specified MVP capabilities.

Phase 1 selects no business quantity precision. The only persisted preference is theme (system/light/dark). Windows signing is a later distribution requirement; native development needs C++ Build Tools/Windows SDK and WebView2, not a signing certificate.
