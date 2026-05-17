# Plan Verification Review: v1-bench-docs-acceptance

## Verdict
resolved_for_reverification

## Findings

### Must Fix
1. [x] Align the benchmark workload schema with the canonical contract.
   - `spec.md` and `contracts.md` require the benchmark workload to use `bench_items(id INT, value TEXT)`.
   - Before retry fix, `design.md` specified `bench_items(id INT PRIMARY KEY, value TEXT)`.
   - Before retry fix, `tasks.md` instructed the worker to generate SQL for `bench_items(id INT PRIMARY KEY, value TEXT)`.
   - This would have changed the measured workload by enabling primary-key behavior and would have produced benchmark evidence for a different schema than the acceptance contract requires.

## Required Checklist Before Re-Verification
- [x] Update `specs/v1-bench-docs-acceptance/design.md` to use exactly `bench_items(id INT, value TEXT)` for the benchmark workload.
- [x] Update `specs/v1-bench-docs-acceptance/tasks.md` T2 to generate SQL for exactly `bench_items(id INT, value TEXT)`.
- [x] Re-scan all derived plan artifacts for `bench_items` and ensure no derived artifact reintroduces `PRIMARY KEY` in the benchmark workload.
- [x] Refresh `analysis_report.md`, `readiness.md`, and `spec-progress.md` if their PASS/GO conclusions depend on the corrected derived artifacts.

## Review Notes
- The remaining plan structure is mostly decision-complete: script boundary, JSON schema, lower-bound policy, verification commands, `db bench` exclusion, and acceptance-doc gate rows are all covered.
- Retry 1 correction updated the derived plan so implementation can start from the corrected benchmark workload schema after plan re-verification.

## Retry 1 Resolution Evidence
- `design.md` benchmark workload now uses exactly `bench_items(id INT, value TEXT)`.
- `tasks.md` T2 now instructs implementation to generate SQL for exactly `bench_items(id INT, value TEXT)`.
- Re-scan of derived artifacts found no benchmark workload instruction that reintroduces `PRIMARY KEY`.
- `analysis_report.md`, `readiness.md`, and `spec-progress.md` were refreshed to reflect the corrected schema.
