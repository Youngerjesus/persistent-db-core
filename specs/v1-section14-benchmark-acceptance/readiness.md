# Final Readiness: v1-section14-benchmark-acceptance

## Verdict
GO

Implementation may begin from the frozen approved package.

## Ready Inputs
- Canonical spec: `spec.md`
- Canonical contract: `contracts.md`
- Research: `research.md`
- Plan: `plan.md`
- Design: `design.md`
- Tasks: `tasks.md`
- Analysis: `analysis_report.md`

## Required Implementation Evidence
- `db bench` exits `0` and prints `DB_BENCH: PASS evidence=target/bench_acceptance/section14-benchmark-acceptance.json`.
- `scripts/verify_bench_acceptance` exits `0` and prints `BENCH_ACCEPTANCE: PASS evidence=target/bench_acceptance/section14-benchmark-acceptance.json`.
- Repo-outside absolute invocation of `scripts/verify_bench_acceptance` exits `0` and writes the same repo-relative evidence path.
- `scripts/verify` exits `0`.
- Evidence JSON satisfies the full schema, fixed fixture, speedup, recovery, index-use, and hard-fail contracts.
- Final evidence lifecycle is respected: `db bench` writes truthful pre-verifier evidence, and `scripts/verify_bench_acceptance` owns the post-validation verifier pass entry in the same JSON file.
- `INTEGER` alias support is implemented narrowly enough for the contracted benchmark schema and covered by focused SQL/docs updates.
- Full-scan hard-fail behavior is covered by negative regression tests for eligible equality and range evidence.
- Docs trace `METRIC-14-1`, `METRIC-14-2`, `METRIC-14-3`, `METRIC-14-4`, `FAIL-14-5`, `EVID-15`, and `EVID-16-7`.

## Blockers
None.

## Guardrails For Next Phase
- Do not edit `spec.md` or `contracts.md`.
- Do not edit `ssot/` or `policies/` unless a later phase explicitly escalates.
- Do not replace public `db bench` with script-only benchmark evidence.
- Do not mark non-targeted V1 obligations as complete through this Section 14 artifact.
