# 072 Terminal Session Lint Debt Checklist

Date: 2026-02-13

## Scope

This dump captures the currently failing lint findings reported by `just check-all` in `src/lib/terminal/session.ts`.

## Resolution Status

Status: Resolved on 2026-02-13.

## Current Lint Findings

1. `biome lint` error: `lint/complexity/noExcessiveCognitiveComplexity`
   - File: `src/lib/terminal/session.ts:356`
   - Symbol: `syncRuntimeMode` (inside `useEffect`)
   - Detail: complexity score `23` exceeds max `15`.

2. `biome lint` error: `lint/correctness/noUnsafeFinally`
   - File: `src/lib/terminal/session.ts:392`
   - Detail: `return` inside `finally` can overwrite control flow from `try`/`catch`.

3. `oxlint` warning: `eslint(no-unsafe-finally)`
   - File: `src/lib/terminal/session.ts:392`
   - Detail: same unsafe `finally` control-flow issue as above.

## Why This Fails The Gate

- `just check-all` runs linting as a hard gate.
- Biome reports `noUnsafeFinally` as an error, so the lint step exits non-zero.
- `noExcessiveCognitiveComplexity` is also configured as an error.

## Remediation Checklist

### Unsafe `finally` (must-fix first)

- [x] Remove `return` from `finally` in `syncRuntimeMode`.
- [x] Preserve cancellation semantics without flow-control in `finally` (assign state only).
- [x] Keep error propagation behavior identical (do not swallow thrown errors).

### Complexity reduction (`23 -> <= 15`)

- [x] Extract inactive-path handling to a helper (set buffered mode and exit).
- [x] Extract active resume-sync path to a helper (buffered -> replay -> live -> replay queued).
- [x] Extract queued output flush ordering to a helper.
- [x] Keep current invariants explicit:
  - [x] replay cursor monotonicity.
  - [x] live mode restoration attempt on failure.
  - [x] queued output cleared on cancellation/finalization.

### Behavioral safety checks

- [x] Re-run existing terminal session tests.
- [x] Ensure no regression in:
  - [x] inactive buffering behavior.
  - [x] replay on reactivation.
  - [x] restore-to-live on replay failure.
  - [x] seq ordering and dedupe behavior.

### Verification commands

- [x] `just check-all`
- [x] `bun run test:run src/lib/terminal/session.test.ts`

## Definition of Done

- `just check-all` passes with zero lint errors.
- No changes to expected runtime behavior in terminal stream mode tests.

## Verification Log

- `bunx biome lint src/lib/terminal/session.ts`
- `bunx oxlint src/lib/terminal/session.ts`
- `bun run test:run src/lib/terminal/session.test.ts`
- `just check-all`
