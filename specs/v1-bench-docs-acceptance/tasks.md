# Tasks: v1-bench-docs-acceptance

## T1. Revalidate Implementation Context [x]
- Files: read-only.
- Steps:
  - Check `git status --short --branch`.
  - Re-read `scripts/verify`, `docs/cli_contract.md`, `docs/v1_spec.md`, `work_queue/progress.md`, and current tests related to CLI dispatch.
  - Confirm `db bench` remains reserved and unsupported.
- Evidence: implementation notes or final report command transcript.

## T2. Add Benchmark Acceptance Script [x]
- Files: `scripts/verify_bench_acceptance`.
- Subtasks:
  - Add executable Bash script with `set -euo pipefail`.
  - Resolve repo root from script path and `cd` there.
  - Create `target/bench_acceptance/`.
  - Generate deterministic 1,000-row SQL for exactly `bench_items(id INT, value TEXT)`.
  - Implement `bench_insert_1k` warmup plus three measured fresh-DB iterations.
  - Implement `bench_reopen_select_1k` warmup plus three measured fresh-DB iterations.
  - Validate exit code and empty stderr for mutation commands.
  - Validate select stdout header, 1,000 rows, first row, and last row.
  - Compute rows/sec per measured iteration.
  - Fail when any measured iteration is below threshold.
  - Emit JSON at `target/bench_acceptance/v1-bench-docs-acceptance.json` with all required fields.
- Acceptance:
  - Script does not call `db bench`.
  - Script works from non-root caller cwd.
  - Script records `evidence_id` suitable for `evidence-v1-benchmark-lower-bounds`.

## T3. Add Benchmark Documentation [x]
- Files: `docs/benchmarks.md`.
- Subtasks:
  - Document command: `scripts/verify_bench_acceptance`.
  - Document artifact path: `target/bench_acceptance/v1-bench-docs-acceptance.json`.
  - Document workload schema and deterministic row values.
  - Document warmup and measured iteration policy.
  - Document thresholds and minimum-iteration pass/fail rule.
  - Document JSON schema minimum.
  - Document environment assumptions and non-guarantees.
- Acceptance:
  - No unsupported performance claims.
  - No claim that `db bench` is available.

## T4. Add V1 Acceptance Guide [x]
- Files: `docs/v1_acceptance.md`.
- Subtasks:
  - Include `evidence-v1-acceptance-docs`.
  - Name `autopilot/ssot/current-artifact.md` as the gate source at handoff.
  - Add rows for all required gate/requirement pairs:
    - `gate-v1-cli-smoke`: `req-v1-cli-help-smoke`, `req-v1-cli-dispatch-tests`
    - `gate-v1-disk-page-storage`: `req-v1-page-storage-restart`, `req-v1-record-format-doc`
    - `gate-v1-sql-schema-exec`: `req-v1-sql-exec-examples`
    - `gate-v1-indexes`: `req-v1-primary-index-proof`, `req-v1-secondary-index-proof`
    - `gate-v1-transactions-wal-recovery`: `req-v1-wal-recovery-proof`
    - `gate-v1-crash-testing`: `req-v1-crash-matrix-output`
    - `gate-v1-differential-property-tests`: `req-v1-differential-property-proof`
    - `gate-v1-db-check-invariants`: `req-v1-db-check-proof`
    - `gate-v1-bench-docs-acceptance`: `req-v1-benchmark-lower-bounds`, `req-v1-acceptance-docs`
  - For each row include evidence path, verification command or manual review evidence, and current status.
  - Mark missing evidence explicitly instead of treating progress notes as completion.
- Acceptance:
  - Every required row exists.
  - `req-v1-secondary-index-proof` is not falsely completed unless new evidence exists.

## T5. Minimal Reference Updates [x]
- Files: `docs/v1_spec.md`, `docs/cli_contract.md` only if needed.
- Subtasks:
  - Clarify script-local benchmark acceptance evidence only if durable docs would otherwise conflict.
  - Preserve `bench <path>` under reserved future commands.
- Acceptance:
  - Public CLI surface unchanged.

## T6. Focused Tests If Needed [x]
- Files: `tests/`.
- Subtasks:
  - Add or update tests only for behavior touched by this task.
  - Prefer preserving existing CLI reserved-command tests over adding broad new coverage.
- Acceptance:
  - No unrelated feature tests or engine behavior changes.

## T7. Verification And Evidence Capture [x]
- Commands:
  - `scripts/verify`
  - `scripts/verify_bench_acceptance`
- Evidence:
  - `target/bench_acceptance/v1-bench-docs-acceptance.json`
  - command output summary
  - docs diff for `docs/benchmarks.md` and `docs/v1_acceptance.md`
- Final report:
  - Include `evidence-v1-benchmark-lower-bounds`.
  - Include `evidence-v1-acceptance-docs`.
